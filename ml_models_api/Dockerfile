FROM python:3.11-slim as base

# Instalar dependências do sistema necessárias para processamento de imagens, 
# Computer Vision e modelos de ML
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    libffi-dev \
    libxml2-dev \
    libxslt1-dev \
    zlib1g-dev \
    libjpeg-dev \
    libpng-dev \
    curl \
    libgl1-mesa-glx \
    ffmpeg \
    libsm6 \
    libxext6 \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Instalar Rust (necessário para algumas dependências)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

FROM base as deps

# Definir o diretório de trabalho
WORKDIR /app

# Copiar requirements.txt primeiro para aproveitar o cache do Docker
COPY ./requirements.txt /app/requirements.txt

# Instalar dependências Python
RUN pip install --no-cache-dir -r requirements.txt

FROM deps as runner

# Copiar o código da aplicação
COPY ./app /app/app

# Copiar os modelos (se estiverem no diretório local)
COPY ./app/models /app/app/models

# Definir variáveis de ambiente
ENV PYTHONUNBUFFERED=1

# Expor a porta
EXPOSE 8000

# Iniciar a aplicação
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
