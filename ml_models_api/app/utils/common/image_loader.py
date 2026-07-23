import io

import numpy as np
import pydicom
from fastapi import HTTPException
from PIL import Image
from pydicom.errors import InvalidDicomError
from pydicom.pixels import apply_modality_lut, apply_voi_lut

# Assinatura DICOM ("DICM") localizada logo após o preâmbulo de 128 bytes.
_DICOM_PREAMBLE_SIZE = 128
_DICOM_MAGIC = b"DICM"


def _is_dicom(image_data: bytes) -> bool:
    return (
        len(image_data) > _DICOM_PREAMBLE_SIZE + len(_DICOM_MAGIC)
        and image_data[_DICOM_PREAMBLE_SIZE:_DICOM_PREAMBLE_SIZE + len(_DICOM_MAGIC)]
        == _DICOM_MAGIC
    )


def _dicom_to_image(image_data: bytes) -> Image.Image:
    try:
        ds = pydicom.dcmread(io.BytesIO(image_data))
        arr = ds.pixel_array
    except InvalidDicomError as exc:
        raise HTTPException(
            status_code=400,
            detail="Arquivo DICOM inválido ou corrompido.",
        ) from exc
    except Exception as exc:
        # Ex.: transfer syntax comprimida sem plugin de decodificação disponível.
        raise HTTPException(
            status_code=400,
            detail="Não foi possível decodificar o pixel data do DICOM.",
        ) from exc

    # Se for multiframe/3D (e não uma imagem colorida com canais), usa o 1º frame.
    if arr.ndim == 3 and ds.get("SamplesPerPixel", 1) == 1:
        arr = arr[0]

    # Rescale Slope/Intercept e windowing (retornam o array inalterado se ausentes).
    arr = apply_modality_lut(arr, ds)
    arr = apply_voi_lut(arr, ds)

    # MONOCHROME1 é grayscale invertido (0 = branco): inverte para exibição normal.
    if ds.get("PhotometricInterpretation", "") == "MONOCHROME1":
        arr = arr.max() - arr

    # Normaliza para uint8 (0-255), com guarda contra imagem chapada.
    arr = arr.astype(np.float64)
    arr = arr - arr.min()
    max_value = arr.max()
    if max_value > 0:
        arr = arr / max_value
    arr = (arr * 255).astype(np.uint8)

    return Image.fromarray(arr)


def load_image_from_bytes(image_data: bytes) -> Image.Image:
    if _is_dicom(image_data):
        return _dicom_to_image(image_data)

    return Image.open(io.BytesIO(image_data))
