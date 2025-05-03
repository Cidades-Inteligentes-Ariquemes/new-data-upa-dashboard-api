# UPA Dashboard API

API para o sistema de Dashboard UPA - Uma API Rust com recursos de processamento de dados, autenticação, visualização de gráficos e integração com modelos de aprendizado de máquina.

## Estrutura do Projeto

A API segue uma arquitetura em camadas baseada nos princípios de Clean Architecture:

```
src/
├── adapters/            # Implementações de interfaces (portas)
├── application/         # Serviços de aplicação (casos de uso)
├── domain/              # Modelos de domínio e interfaces de repositório
├── handlers/            # Manipuladores HTTP para tratamento de requisições 
├── infrastructure/      # Implementação de persistência e serviços externos
├── middleware/          # Middleware (autenticação, auditoria, logging)
├── routes/              # Definição de rotas da API
└── utils/               # Utilitários diversos

ml_models_api/           # Serviço Python para modelos de ML
```

## Principais Funcionalidades

### 1. Autenticação

- Login com email/senha (`/api/auth/login`)
- Login com sistema Pronto (`/api/auth/login-pronto`)
- Middleware JWT para proteção de rotas

### 2. Gestão de Usuários

- CRUD completo de usuários
- Gestão de permissões e aplicações
- Recuperação de senha com código de verificação
- Feedback para modelos de predição

### 3. Dados UPA

- Upload e processamento de dados
- Visualizações diversas para o dashboard
  - Consultas por mês/ano
  - Distribuição por faixa etária
  - Atendimentos por fluxo
  - Mapas de calor para doenças e atendimentos por bairro
  - Estatísticas de atendimento por profissional

### 4. Predição e ML

- Integração com serviço Python separado para ML
- Predição de doenças respiratórias
- Detecção de câncer de mama
- Predição de tuberculose

### 5. Informações e Auditoria

- Registro de ações dos usuários
- Métricas do sistema e da máquina

## Arquitetura Técnica

### Camadas Principais

1. **Handlers**: Recebem requisições HTTP, validam parâmetros, delegam para serviços e formatam respostas.
2. **Services (Application)**: Implementam a lógica de negócios, coordenam chamadas a repositories.
3. **Repositories**: Abstraem o acesso a dados, permitem interações com banco de dados.
4. **Domain Models**: Definem estruturas de dados e regras de negócio.
5. **Middleware**: Funções que interceptam e processam requisições (auth, auditoria, logging).

### Tecnologias Utilizadas

- **Backend Rust**:
  - Actix-web: Framework web
  - Tokio: Runtime assíncrono
  - sqlx: Biblioteca de banco de dados assíncrona
  - serde: Serialização/deserialização
  - jsonwebtoken: Autenticação JWT
  - chrono: Manipulação de datas e horas
  - uuid: Geração de identificadores únicos

- **Serviço ML (Python)**:
  - FastAPI: Framework web
  - Modelos de predição para doenças respiratórias e tuberculose
  - Modelo de detecção para câncer de mama

## Endpoints API

### Autenticação

- **POST** `/api/auth/login` - Autenticação por email/senha
- **POST** `/api/auth/login-pronto` - Autenticação via sistema Pronto

### Usuários

- **GET** `/api/users` - Listar todos os usuários
- **POST** `/api/users` - Criar novo usuário
- **GET** `/api/users/{id}` - Obter usuário específico
- **PUT** `/api/users/{id}` - Atualizar usuário
- **DELETE** `/api/users/{id}` - Remover usuário
- **POST** `/api/users/send-verification-code/{email}` - Enviar código de verificação para reset de senha
- **POST** `/api/users/resend-verification-code/{email}` - Reenviar código de verificação
- **PATCH** `/api/users/update-password-for-forgetting-user/{id}` - Atualizar senha para usuário que esqueceu
- **PATCH** `/api/users/update-password-by-user-common/{id}` - Atualizar senha pelo próprio usuário
- **POST** `/api/users/confirm-verification-code` - Confirmar código de verificação
- **POST** `/api/users/feedback-respiratory-diseases` - Enviar feedback sobre predição de doenças respiratórias
- **POST** `/api/users/feedback-tuberculosis` - Enviar feedback sobre predição de tuberculose
- **GET** `/api/users/feedbacks` - Obter feedbacks
- **PATCH** `/api/users/{id}/update-password-by-admin` - Atualizar senha por admin
- **DELETE** `/api/users/{id}/application/{application_name}` - Remover aplicação de usuário
- **POST** `/api/users/{id}/applications` - Adicionar aplicações a usuário
- **PATCH** `/api/users/{id}/update-enabled` - Ativar/desativar usuário

### Dados UPA

- **POST** `/api/data/add-file` - Adicionar arquivo de dados
- **GET** `/api/data/update-graph-data` - Atualizar dados de gráficos
- **GET** `/api/data/number-of-appointments-per-month` - Número de atendimentos por mês
- **GET** `/api/data/number-of-appointments-per-year/{year}` - Número de atendimentos por ano específico
- **GET** `/api/data/years-available-for-number-of-appointments-per-month` - Anos disponíveis para consulta
- **GET** `/api/data/number-of-appointments-per-flow` - Número de atendimentos por fluxo
- **GET** `/api/data/distribuition-of-patients-ages` - Distribuição por idade dos pacientes
- **GET** `/api/data/number-of-calls-per-day-of-the-week` - Número de chamadas por dia da semana
- **GET** `/api/data/distribution-of-services-by-hour-group` - Distribuição de serviços por grupo de horas
- **GET** `/api/data/number-of-visits-per-nurse` - Número de visitas por enfermeiro
- **GET** `/api/data/number-of-visits-per-doctor` - Número de visitas por médico
- **GET** `/api/data/average-time-in-minutes-per-doctor` - Tempo médio de atendimento por médico
- **GET** `/api/data/heat-map-with-disease-indication` - Mapa de calor com indicação de doenças
- **GET** `/api/data/heat-map-with-the-number-of-medical-appointments-by-neighborhood` - Mapa de calor com atendimentos por bairro

### Predição

- **POST** `/api/prediction/predict` - Predizer doença respiratória a partir de imagem
- **POST** `/api/prediction/detect` - Detectar câncer de mama em imagem
- **POST** `/api/prediction/predict_tb` - Predizer tuberculose a partir de imagem

### Informação

- **GET** `/api/information/audits/audit-filters` - Obter filtros disponíveis para auditoria
- **GET** `/api/information/audits/{page}` - Obter registros de auditoria paginados

### Informações da Máquina

- **GET** `/api/machine-information` - Obter informações do sistema e da máquina

## Segurança

- Middleware de autenticação via JWT
- Criptografia de senhas usando Argon2
- Registros de auditoria para todas as ações
- Validação de permissões baseada em perfil de usuário

## Como Executar

### Requisitos

- Rust (versão estável)
- PostgreSQL
- Python 3.8+ (para o serviço ML)
- Docker e Docker Compose (opcional)

### Configuração do Ambiente

1. Clone o repositório
2. Configure o arquivo `.env` baseado no `example.env`
3. Execute as migrações do banco de dados
4. Compile e execute a API Rust

```bash
# Configurar ambiente
cp example.env .env
# edite o arquivo .env com suas configurações

# Executar API
cargo run

# Para o serviço ML
cd ml_models_api
pip install -r requirements.txt
python -m app.main
```

### Usando Docker

```bash
docker-compose up -d
```

## Desenvolvimento

A API segue convenções de código Rust e está estruturada seguindo padrões de Clean Architecture. O sistema de rotas, handlers e serviços facilita a adição de novas funcionalidades.

Ao desenvolver novas funcionalidades:

1. Defina modelos no módulo `domain/models`
2. Crie interfaces de repositório em `domain/repositories`
3. Implemente repositórios em `infrastructure/repositories`
4. Implemente serviços em `application`
5. Crie handlers em `handlers`
6. Configure rotas em `routes`