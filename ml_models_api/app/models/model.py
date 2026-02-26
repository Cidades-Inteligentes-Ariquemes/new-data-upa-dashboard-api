from ultralytics import YOLO
import torch
import torchvision
import torch.nn as nn
from torchvision import models, transforms
from torchvision.models import resnet50, ResNet50_Weights
from torchvision.models.detection.faster_rcnn import FastRCNNPredictor

def load_model():
    model = YOLO('app/models/best.pt')
    return model

def load_model_breast_cancer_with_fatRCNN(device):
    num_classes = 3  

    model = torchvision.models.detection.fasterrcnn_resnet50_fpn(weights=None)

    # Obtem o número de características de entrada
    in_features = model.roi_heads.box_predictor.cls_score.in_features

    # Substitui o preditor por um novo com o número de classes desejado
    model.roi_heads.box_predictor = FastRCNNPredictor(in_features, num_classes)

    model.load_state_dict(torch.load('app/models/faster_rcnn_model.pth', map_location=device))


    model.to(device)
    model.eval()

    return model

class TuberculosisModel(nn.Module):
    def __init__(self):
        super().__init__()
        # Carrega ResNet50 com pesos do ImageNet
        self.resnet = resnet50(weights=ResNet50_Weights.IMAGENET1K_V1)

        # Congelar as camadas iniciais
        ct = 0
        for child in self.resnet.children():
            ct += 1
            if ct < 7:
                for param in child.parameters():
                    param.requires_grad = False

        # Altera a última FC (igual ao treino)
        num_ftrs = self.resnet.fc.in_features
        self.resnet.fc = nn.Sequential(
            nn.Linear(num_ftrs, 128),
            nn.BatchNorm1d(128),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(128, 64),
            nn.BatchNorm1d(64),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(64, 32),
            nn.BatchNorm1d(32),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(32, 2)
        )

    def forward(self, x):
        # Retorna logits crus
        return self.resnet(x)


def load_model_tuberculosis(device):
    model = TuberculosisModel().to(device)
    state_dict = torch.load("app/models/best_model-18-01-2025.pth", map_location=device)
    model.load_state_dict(state_dict)
    model.eval()
    return model

class OsteoporosisModel(nn.Module):
    def __init__(self):
        super().__init__()
        # Carrega ResNet50 com pesos da ImageNet
        self.resnet = resnet50(weights=ResNet50_Weights.IMAGENET1K_V1)

        # Congela as camadas iniciais
        ct = 0
        for child in self.resnet.children():
            ct += 1
            if ct < 7:
                for param in child.parameters():
                    param.requires_grad = False

        # Modifica a última camada FC para classificação de 3 classes
        num_ftrs = self.resnet.fc.in_features
        self.resnet.fc = nn.Sequential(
            nn.Linear(num_ftrs, 128),
            nn.BatchNorm1d(128),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(128, 64),
            nn.BatchNorm1d(64),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(64, 32),
            nn.BatchNorm1d(32),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(32, 3)
        )

    def forward(self, x):
        return self.resnet(x)


def load_model_osteoporosis(device):
    try:
        
        model = OsteoporosisModel().to(device)
        
        state_dict = torch.load("app/models/best_model_osteoporosis.pth", map_location=device)
        model.load_state_dict(state_dict)
        
        model.eval()
        
        return model
    except Exception as e:
        raise RuntimeError(f"Failed to load osteoporosis model: {str(e)}")
