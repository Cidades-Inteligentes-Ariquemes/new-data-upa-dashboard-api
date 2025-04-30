from fastapi import UploadFile
from ..services.prediction_service import PredictionService
from fastapi.responses import JSONResponse
import base64
import io


class PredictionController:
    def __init__(self):
        self.prediction_service = PredictionService()

    async def handle_prediction(self, file: UploadFile):
        image = await file.read()
        
        prediction = await self.prediction_service.predict_image(image)
        
        return {"prediction": prediction}

    async def handle_detect_breast_cancer_with_fastRCNN(self, file: UploadFile):
        image_data = await file.read()
        
        result = await self.prediction_service.detect_breast_cancer_with_fastRCNN(image_data)
        
        image_base64 = base64.b64encode(result["image"]).decode('utf-8')
        
        return JSONResponse(content={
            "detections": result["detections"],
            "image": image_base64
        })

    async def handle_prediction_tuberculosis(self, file: UploadFile):
        image_data = await file.read()
        
        result = await self.prediction_service.predict_tuberculosis_image(image_data)
        
        return {
            "prediction_tb": result
        }