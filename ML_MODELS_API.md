# ML Models API — Guia Completo de Fluxo e Arquitetura

Este documento descreve em detalhes o serviço Python `ml_models_api`, responsável por receber imagens, executar predições com redes neurais (YOLOv8, Faster R-CNN e classificadores baseados em ResNet50) e retornar os resultados para a API principal do Dashboard UPA.

- Stack principal: FastAPI, PyTorch, Torchvision, Ultralytics YOLO, OpenCV e Pillow.
- Localização: `ml_models_api/`
- Entrypoint: `ml_models_api/app/main.py`


## Sumário

- Visão geral do fluxo
- Arquitetura e pastas
- Endpoints e contratos (request/response)
- Camadas: routes → controllers → services → utils → models
  - O que cada função faz (com referências de arquivo)
  - Detalhe do carregamento dos modelos e pesos (`models/model.py`)
- Erros e validações
- Execução (Docker) e como testar via curl
- Notas de desempenho e GPU/CPU

---

## Visão geral do fluxo (da imagem à predição)

1) Cliente envia uma imagem via HTTP multipart/form-data usando a chave `file` para um dos endpoints:
   - `POST /predict` → classificador YOLOv8 (salva resultados em TXT e converte em dicionário)
   - `POST /detect` → detecção com Faster R-CNN, retorna caixas + imagem anotada (base64)
   - `POST /predict_tb` → classificação binária (tuberculose: negative/positive)
   - `POST /osteoporosis` → classificação em 3 classes (normal/osteopenia/osteoporosis)

2) A rota delega para o controller, que:
   - lê os bytes da imagem (`UploadFile.read()`),
   - chama o service correspondente.

3) O service realiza o pré-processamento (quando necessário), chama o modelo carregado em memória (instanciado no `__init__` do service) e pós-processa a saída (decodificação, scores, NMS, formatação, etc.).

4) O controller formata a resposta HTTP (por exemplo, converte a imagem anotada em base64) e retorna JSON ao cliente.

5) Em caso de erros (imagem inválida, problemas de inferência), exceções HTTP são lançadas com mensagens e status adequados.


## Arquitetura e pastas

```
ml_models_api/
  Dockerfile
  requirements.txt
  app/
    main.py                 # Cria a FastAPI, CORS e inclui o router de predição
    routes/
      prediction_route.py   # Endpoints públicos
    controllers/
      prediction_controller.py
    services/
      prediction_service.py # Lógica de negócio e inferência
    models/
      model.py              # Definição e carregamento dos modelos/pesos
      best.pt
      faster_rcnn_model.pth
      best_model-18-01-2025.pth
      best_model_osteoporosis.pth
    utils/
      common/
        load_file.py        # Conversão de arquivo de resultados YOLO em dicionário
```

- O `docker-compose.yml` na raiz sobe este serviço como `ml_api` (porta 8000) e expõe a URL interna `http://ml_api:8000` para a aplicação Rust.


## Endpoints e contratos

Base URL (Docker): `http://localhost:8000`

### 1) Predição YOLO — `POST /predict`
- Corpo: multipart/form-data com `file=@/caminho/para/imagem.jpg`
- Retorno: JSON com um dicionário de rótulos e probabilidades (%) lidas do arquivo `results.txt` gerado pelo YOLO.
- Exemplo de resposta:
```json
{
  "prediction": {
    "pneumonia": 87.12,
    "covid": 5.34,
    "normal": 7.54
  }
}
```

### 2) Detecção Câncer de Mama (Faster R-CNN) — `POST /detect`
- Corpo: multipart/form-data com `file=@/caminho/para/imagem.jpg`
- Retorno: JSON com:
  - `detections`: lista de detecções (class_id, confidence, bbox [xmin, ymin, xmax, ymax])
  - `image`: imagem anotada em base64 (formato JPEG)
- Exemplo de resposta:
```json
{
  "detections": [
    { "class_id": 1, "confidence": 0.92, "bbox": [34, 52, 150, 200] }
  ],
  "image": "...base64..."
}
```

### 3) Predição Tuberculose — `POST /predict_tb`
- Corpo: multipart/form-data com `file=@/caminho/para/imagem.jpg`
- Retorno: JSON com classe predita e probabilidades em %:
```json
{
  "prediction_tb": {
    "class_pred": "negative",
    "probabilities": { "negative": 97.12, "positive": 2.88 }
  }
}
```

### 4) Predição Osteoporose — `POST /osteoporosis`
- Corpo: multipart/form-data com `file=@/caminho/para/imagem.jpg`
- Retorno: JSON com classe predita e probabilidades em %:
```json
{
  "prediction_osteoporosis": {
    "class_pred": "osteopenia",
    "probabilities": {
      "normal": 12.34,
      "osteopenia": 80.11,
      "osteoporosis": 7.55
    }
  }
}
```


## Camadas e funções (routes → controllers → services → utils → models)

### app/main.py
- Cria a aplicação FastAPI e configura CORS.
- Inclui o router de predição com a tag "Predição e Detecção - Dashboard UPA API".

Referência: `ml_models_api/app/main.py`

```python
app = FastAPI()
app.add_middleware(CORSMiddleware, allow_origins=["*"], ...)
app.include_router(prediction_router, tags=["Predição e Detecção - Dashboard UPA API"])
```

---

### routes/prediction_route.py
- Define as rotas públicas:
  - `POST /predict` → `PredictionController.handle_prediction`
  - `POST /detect` → `PredictionController.handle_detect_breast_cancer_with_fastRCNN`
  - `POST /predict_tb` → `PredictionController.handle_prediction_tuberculosis`
  - `POST /osteoporosis` → `PredictionController.handle_prediction_osteoporosis`

Referência: `ml_models_api/app/routes/prediction_route.py`

---

### controllers/prediction_controller.py
- Constrói um `PredictionService` na inicialização.
- Funções:
  - `handle_prediction(file)`
    - Lê os bytes da imagem e chama `prediction_service.predict_image(image_bytes)`.
    - Retorna `{"prediction": <dict>}`.
  - `handle_detect_breast_cancer_with_fastRCNN(file)`
    - Lê bytes, chama `prediction_service.detect_breast_cancer_with_fastRCNN(...)`.
    - Converte os bytes da imagem anotada para base64 e retorna JSON com `detections` + `image` (base64).
  - `handle_prediction_tuberculosis(file)`
    - Lê bytes, chama `prediction_service.predict_tuberculosis_image(...)` e retorna `{"prediction_tb": {...}}`.
  - `handle_prediction_osteoporosis(file)`
    - Lê bytes, chama `prediction_service.predict_osteoporosis(...)` e retorna `{"prediction_osteoporosis": {...}}`.

Referência: `ml_models_api/app/controllers/prediction_controller.py`

---

### services/prediction_service.py
- Carrega todos os modelos uma única vez no `__init__` do service, definindo automaticamente o `device` (GPU se disponível):
  - `self.model = load_model()` → YOLOv8 (Ultralytics)
  - `self.model_breast_cancer_faster_rcnn = load_model_breast_cancer_with_fatRCNN(device)` → Faster R-CNN
  - `self.model_tb = load_model_tuberculosis(device)` → classificador tuberculose
  - `self.model_osteoporosis = load_model_osteoporosis(device)` → classificador osteoporose

- Principais métodos:

  1) `predict_image(image_data: bytes)` — YOLOv8
     - Abre a imagem com Pillow.
     - Roda `self.model(image)` (Ultralytics) e salva `prediction[0]` em TXT (`results.txt`) via `save_txt`.
     - Converte o TXT para dicionário (`utils.common.load_file_to_dictionary`), remove o arquivo e retorna o dicionário.
     - Erros: lança `HTTPException 400` se a imagem for inválida ou não houver predições válidas.

  2) `detect_breast_cancer_with_fastRCNN(image_data: bytes)` — Faster R-CNN
     - Abre e converte para RGB.
     - Redimensiona mantendo proporção com máximo de 1024 px no maior lado para estabilidade e performance.
     - Converte para tensor (`ToTensor`) e envia para `device`.
     - Faz inferência (`model([img_tensor])`).
     - Filtra por `score_threshold=0.7` e aplica NMS (`nms_threshold=0.7`).
     - Desenha bounding boxes e scores sobre a imagem (Pillow `ImageDraw`, com espessura e fonte proporcionais ao tamanho da imagem; fallback de fonte para DejaVuSans se `arial.ttf` não estiver presente).
     - Retorna: bytes JPEG da imagem anotada + lista `detections` (class_id, confidence, bbox).

  3) `predict_tuberculosis_image(image_data: bytes)` — Classificador ResNet50
     - Pré-processamento: `Resize(224x224)`, `ToTensor`, normalização ImageNet.
     - Inferência em `self.model_tb` (logits) e `softmax` para probabilidades.
     - Retorna `class_pred` em `{negative, positive}` e `probabilities` em %.

  4) `predict_osteoporosis(image_data: bytes)` — Classificador ResNet50
     - Pré-processamento igual ao TB.
     - Inferência em `self.model_osteoporosis` (3 classes) e `softmax`.
     - Retorna `class_pred` em `{normal, osteopenia, osteoporosis}` e `probabilities` em %.

- Tratamento de erros: converte exceções em `HTTPException` com mensagens claras (400 para imagem inválida; 500 para erros internos).

Referência: `ml_models_api/app/services/prediction_service.py`

---

### utils/common/load_file.py
- `load_file_to_dictionary(file_path)`
  - Lê arquivo linha a linha assumindo o formato "<valor> <rótulo>".
  - Converte `valor` para float, multiplica por 100 (probabilidade em %) e monta `dict[rótulo] = valor_em_%`.

Referência: `ml_models_api/app/utils/common/load_file.py`

---

### models/model.py (carregamento dos modelos e pesos) — Detalhado

Este módulo centraliza a definição das arquiteturas e o carregamento de pesos treinados.

- YOLOv8 (Ultralytics)
  - `load_model()`
    - `model = YOLO('app/models/best.pt')`
    - Retorna a instância do modelo já pronta para `__call__` em imagens Pillow/NumPy.

- Faster R-CNN (Detecção de massas em mamografia)
  - `load_model_breast_cancer_with_fatRCNN(device)`
    - Cria `fasterrcnn_resnet50_fpn(weights=None)`.
    - Substitui o `box_predictor` por `FastRCNNPredictor(in_features, num_classes=3)` (inclui background implícito → índices de classe a partir de 1 para alvo; o mapeamento usado no service é `{1: 'Mass'}`).
    - Carrega pesos: `model.load_state_dict(torch.load('app/models/faster_rcnn_model.pth', map_location=device))`.
    - Move para `device` e `eval()`.

- Classificador Tuberculose (ResNet50)
  - Classe `TuberculosisModel(nn.Module)`
    - Base: `resnet50(weights=ResNet50_Weights.IMAGENET1K_V1)`.
    - Congele as camadas iniciais (até o 6º bloco) para aproveitar transferência de aprendizado.
    - Cabeça totalmente conectada substituída por: FC → BN → ReLU → Dropout com progressão 128→64→32→2 (2 classes).
  - `load_model_tuberculosis(device)`
    - Instancia `TuberculosisModel`, carrega pesos de `app/models/best_model-18-01-2025.pth` (map_location=device), `eval()`.

- Classificador Osteoporose (ResNet50)
  - Classe `OsteoporosisModel(nn.Module)`
    - Semelhante ao TB, mas a última camada tem 3 saídas (Normal, Osteopenia, Osteoporosis).
  - `load_model_osteoporosis(device)`
    - Instancia `OsteoporosisModel`, carrega pesos de `app/models/best_model_osteoporosis.pth`, `eval()`.

Observações importantes:
- Seleção de dispositivo:
  - O `PredictionService` define `self.device = torch.device('cuda') if torch.cuda.is_available() else torch.device('cpu')`.
  - Todos os modelos são colocados neste `device` (GPU quando disponível), melhorando a performance.
- Modo avaliação:
  - Todos os modelos são colocados em `model.eval()` para desabilitar camadas como dropout e normalização em modo treino.
- Consistência de pré-processamento:
  - Classificadores TB e Osteoporose usam normalização padrão ImageNet (mesma do treino, por coerência).

Referência: `ml_models_api/app/models/model.py`


## Erros e validações (resumo)

- Imagem inválida/corrompida → `HTTP 400` com mensagem explicativa.
- Falha de codificação de imagem anotada (cv2.imencode) → `HTTP 500`.
- Qualquer exceção não tratada explícitamente → `HTTP 500` com mensagem genérica.
- Limpeza de artefato: em `/predict`, o arquivo temporário `results.txt` é removido após leitura.


## Execução com Docker

O serviço é construído e iniciado pelo `docker-compose.yml` da raiz do projeto:
- Serviço: `ml_api`
- Porta exposta: `8000`
- A aplicação Rust (`rust_app`) utiliza `ML_API_URL=http://ml_api:8000` para se comunicar internamente.

Dockerfile relevante (resumo): instala dependências de imagem/visão computacional, instala `requirements.txt`, copia o código e sobe o `uvicorn` em `0.0.0.0:8000`.


## Como testar (curl)

Substitua `/path/to/image.jpg` pelo caminho da imagem no seu ambiente Linux.

- Predição YOLO:
```bash
curl -s -X POST http://localhost:8000/predict \
  -F "file=@/path/to/image.jpg" | jq
```

- Detecção com Faster R-CNN (imagem anotada base64):
```bash
curl -s -X POST http://localhost:8000/detect \
  -F "file=@/path/to/image.jpg" | jq -r '.image' > annotated_base64.txt
# opcional: decodificar base64 em arquivo JPEG
base64 -d annotated_base64.txt > annotated.jpg
```

- Predição Tuberculose:
```bash
curl -s -X POST http://localhost:8000/predict_tb \
  -F "file=@/path/to/image.jpg" | jq
```

- Predição Osteoporose:
```bash
curl -s -X POST http://localhost:8000/osteoporosis \
  -F "file=@/path/to/image.jpg" | jq
```


## Notas de desempenho e GPU/CPU

- Dispositivo: o serviço usa automaticamente GPU se `torch.cuda.is_available()` for verdadeiro; caso contrário, roda em CPU.
- Redimensionamento controlado em detecção: imagens muito grandes são redimensionadas para no máximo 1024px no maior lado para equilibrar performance e qualidade das detecções.
- NMS e thresholds: `score_threshold=0.7` e `nms_threshold=0.7` foram definidos para reduzir falsos positivos — ajuste conforme necessário para seu dataset/uso.
- Fontes: caso `arial.ttf` não esteja disponível, o serviço tenta `DejaVuSans`. Em último caso usa fonte padrão do Pillow.


## Referências rápidas de código

- Entrypoint: `app/main.py`
- Rotas: `app/routes/prediction_route.py`
- Controller: `app/controllers/prediction_controller.py`
- Service: `app/services/prediction_service.py`
- Utils: `app/utils/common/load_file.py`
- Modelos e pesos: `app/models/model.py` e arquivos `.pt/.pth` no mesmo diretório


## Conclusão

Este documento cobre o fluxo ponta a ponta do serviço `ml_models_api`, detalhando como cada camada participa desde a recepção da imagem até a geração e retorno da predição. Para ajustes finos (thresholds, tamanhos de entrada, normalizações, mapeamentos de classes e caminhos dos pesos), consulte e edite diretamente os arquivos nas seções indicadas acima.