from fastapi import APIRouter, UploadFile, File
from ..controllers.prediction_controller import PredictionController

router = APIRouter()

prediction_controller = PredictionController()

@router.post("/predict")
async def predict(file: UploadFile = File(...)):
    return await prediction_controller.handle_prediction(file)

@router.post("/detect")
async def detect_breast_cancer(file: UploadFile = File(...)):
    return await prediction_controller.handle_detect_breast_cancer_with_fastRCNN(file)

@router.post("/predict_tb")
async def predict_tb(file: UploadFile = File(...)):
    return await prediction_controller.handle_prediction_tuberculosis(file)

@router.post("/osteoporosis")
async def predict_osteoporosis(file: UploadFile = File(...)):
    return await prediction_controller.handle_prediction_osteoporosis(file)