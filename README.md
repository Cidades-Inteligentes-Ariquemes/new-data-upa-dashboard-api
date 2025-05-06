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

#### 1. Login
- **URL**: `/api/auth/login`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "email": "usuario@exemplo.com",
    "password": "senha123"
  }
  ```
- **Nível de acesso**: Público
- **Descrição**: Autentica usuário com email e senha, retornando token JWT

#### 2. Login via Pronto
- **URL**: `/api/auth/login-pronto`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "username": "usuario123",
    "password": "senha123"
  }
  ```
- **Nível de acesso**: Público
- **Descrição**: Autentica usuário via sistema Pronto, retornando token JWT e unidades de saúde permitidas

### Usuários

#### 1. Listar Usuários
- **URL**: `/api/users`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Retorna lista de todos os usuários cadastrados

#### 2. Criar Usuário
- **URL**: `/api/users`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "full_name": "João Silva",
    "email": "joao.silva@exemplo.com",
    "password": "senha123",
    "profile": "Administrador", // Valores: "Administrador" ou "Usuario Comum"
    "allowed_applications": ["xpredict", "upavision"],
    "allowed_health_units": [1, 2, 3]
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Cria novo usuário com perfil e aplicações permitidas

#### 3. Obter Usuário Específico
- **URL**: `/api/users/{id}`
- **Método**: GET
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Nível de acesso**: Administrador
- **Descrição**: Retorna dados de um usuário específico

#### 4. Atualizar Usuário
- **URL**: `/api/users/{id}`
- **Método**: PUT
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "full_name": "João Silva Atualizado",
    "email": "joao.silva@exemplo.com",
    "profile": "Usuario Comum",
    "allowed_applications": ["upavision"],
    "allowed_health_units": [1, 2]
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Atualiza dados de um usuário específico

#### 5. Remover Usuário
- **URL**: `/api/users/{id}`
- **Método**: DELETE
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Nível de acesso**: Administrador
- **Descrição**: Remove um usuário específico

#### 6. Enviar Código de Verificação
- **URL**: `/api/users/send-verification-code/{email}`
- **Método**: POST
- **Parâmetros de rota**: `email` (email do usuário)
- **Nível de acesso**: Público
- **Descrição**: Envia código de verificação para reset de senha

#### 7. Reenviar Código de Verificação
- **URL**: `/api/users/resend-verification-code/{email}`
- **Método**: POST
- **Parâmetros de rota**: `email` (email do usuário)
- **Corpo da requisição**:
  ```json
  {
    "id_verification": "123e4567-e89b-12d3-a456-426614174000"
  }
  ```
- **Nível de acesso**: Público
- **Descrição**: Reenvia código de verificação para email

#### 8. Confirmar Código de Verificação
- **URL**: `/api/users/confirm-verification-code`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "verification_code": 123456,
    "id_verification": "123e4567-e89b-12d3-a456-426614174000"
  }
  ```
- **Nível de acesso**: Público
- **Descrição**: Confirma código de verificação recebido por email

#### 9. Atualizar Senha (Usuário que Esqueceu)
- **URL**: `/api/users/update-password-for-forgetting-user/{id}`
- **Método**: PATCH
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "new_password": "novaSenha123",
    "id_verification": "123e4567-e89b-12d3-a456-426614174000"
  }
  ```
- **Nível de acesso**: Público
- **Descrição**: Atualiza senha para usuário que esqueceu, após verificação

#### 10. Atualizar Senha (Usuário Comum)
- **URL**: `/api/users/update-password-by-user-common/{id}`
- **Método**: PATCH
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "current_password": "senha123",
    "new_password": "novaSenha123"
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Permite usuário atualizar sua própria senha

#### 11. Feedback de Doenças Respiratórias
- **URL**: `/api/users/feedback-respiratory-diseases`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "user_name": "João Silva",
    "feedback": "sim", // Valores: "sim" ou "não"
    "prediction_made": "pneumonia viral", // Valores: "normal", "covid-19", "pneumonia viral", "pneumonia bacteriana"
    "correct_prediction": "pneumonia viral" // Valores: "normal", "covid-19", "pneumonia viral", "pneumonia bacteriana"
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Registra feedback sobre predição de doenças respiratórias

#### 12. Feedback de Tuberculose
- **URL**: `/api/users/feedback-tuberculosis`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "user_name": "João Silva",
    "feedback": "sim ou nao"
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Registra feedback sobre predição de tuberculose

#### 13. Listar Feedbacks
- **URL**: `/api/users/feedbacks`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Retorna todos os feedbacks registrados

#### 14. Atualizar Senha (Administrador)
- **URL**: `/api/users/{id}/update-password-by-admin`
- **Método**: PATCH
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "email": "joao.silva@exemplo.com",
    "new_password": "novaSenha123"
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Permite administrador atualizar senha de qualquer usuário

#### 15. Adicionar Aplicações
- **URL**: `/api/users/{id}/applications`
- **Método**: POST
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "applications_name": ["xpredict", "upavision"]
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Adiciona aplicações ao usuário especificado

#### 16. Remover Aplicação
- **URL**: `/api/users/{id}/application/{application_name}`
- **Método**: DELETE
- **Parâmetros de rota**: `id` (UUID do usuário), `application_name` (nome da aplicação)
- **Nível de acesso**: Administrador
- **Descrição**: Remove uma aplicação específica do usuário

#### 17. Ativar/Desativar Usuário
- **URL**: `/api/users/{id}/update-enabled`
- **Método**: PATCH
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "enabled": true
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Ativa ou desativa um usuário específico

#### 18. Adicionar Unidades de Saúde
- **URL**: `/api/users/{id}/health-units`
- **Método**: POST
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "health_units": [1, 2, 3]
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Adiciona unidades de saúde permitidas ao usuário

#### 19. Remover Unidade de Saúde
- **URL**: `/api/users/{id}/health-unit/{health_unit_id}`
- **Método**: DELETE
- **Parâmetros de rota**: `id` (UUID do usuário), `health_unit_id` (ID da unidade de saúde)
- **Nível de acesso**: Administrador
- **Descrição**: Remove uma unidade de saúde específica do usuário

### Dados UPA

#### 1. Adicionar Arquivo de Dados
- **URL**: `/api/data/add-file`
- **Método**: POST
- **Corpo da requisição**: Multipart form com arquivo CSV/Excel
- **Nível de acesso**: Administrador
- **Descrição**: Faz upload e processa arquivo de dados para o sistema

#### 2. Atualizar Dados de Gráficos
- **URL**: `/api/data/update-graph-data`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Processa os dados brutos para gerar visualizações em gráficos

#### 3. Listar Unidades de Saúde Disponíveis
- **URL**: `/api/data/available-health-units`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Retorna lista de unidades de saúde disponíveis no sistema

#### 4. Número de Atendimentos por Mês
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-appointments-per-month`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna quantidade de atendimentos agregados por mês

#### 5. Número de Atendimentos por Ano
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-appointments-per-year/{year}`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade), `year` (ano)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna atendimentos mensais para um ano específico

#### 6. Anos Disponíveis para Consulta
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/years-available-for-number-of-appointments-per-month`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna lista de anos disponíveis para consulta

#### 7. Número de Atendimentos por Fluxo
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-appointments-per-flow`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna quantidade de atendimentos por cada fluxo

#### 8. Distribuição por Idade dos Pacientes
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/distribuition-of-patients-ages`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna distribuição de pacientes por faixas etárias

#### 9. Número de Chamadas por Dia da Semana
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-calls-per-day-of-the-week`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna quantidade de chamadas agregadas por dia da semana

#### 10. Distribuição de Serviços por Grupo de Horas
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/distribution-of-services-by-hour-group`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna distribuição de serviços agrupados por horário

#### 11. Número de Visitas por Enfermeiro
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-visits-per-nurse`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade) 
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna quantidade de visitas realizadas por cada enfermeiro

#### 12. Número de Visitas por Médico
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-visits-per-doctor`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna quantidade de visitas realizadas por cada médico

#### 13. Tempo Médio por Médico
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/average-time-in-minutes-per-doctor`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna tempo médio de atendimento em minutos por médico

#### 14. Mapa de Calor com Indicação de Doenças
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/heat-map-with-disease-indication`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna dados para mapa de calor indicando doenças por região

#### 15. Mapa de Calor com Atendimentos por Bairro
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/heat-map-with-the-number-of-medical-appointments-by-neighborhood`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Retorna dados para mapa de calor com atendimentos por bairro

### Predição

#### 1. Predizer Doença Respiratória
- **URL**: `/api/prediction/predict`
- **Método**: POST
- **Corpo da requisição**: Multipart form com imagem de raio-X torácico
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e prediz possível doença respiratória

#### 2. Detectar Câncer de Mama
- **URL**: `/api/prediction/detect`
- **Método**: POST
- **Corpo da requisição**: Multipart form com imagem de mamografia
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e detecta possível câncer de mama

#### 3. Predizer Tuberculose
- **URL**: `/api/prediction/predict_tb`
- **Método**: POST
- **Corpo da requisição**: Multipart form com imagem de raio-X torácico
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e prediz possível tuberculose

### Informação e Auditoria

#### 1. Obter Registros de Auditoria
- **URL**: `/api/information/audits/{page}`
- **Método**: GET
- **Parâmetros de rota**: `page` (número da página)
- **Parâmetros de query**:
  - `email` (opcional): filtrar por email do usuário
  - `path` (opcional): filtrar por caminho da requisição
  - `date_of_request` (opcional): filtrar por data da requisição
- **Nível de acesso**: Administrador
- **Descrição**: Retorna registros de auditoria paginados

#### 2. Obter Filtros de Auditoria
- **URL**: `/api/information/audits/audit-filters`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Retorna opções de filtros disponíveis para auditoria

### Informações da Máquina

#### 1. Obter Métricas do Sistema
- **URL**: `/api/machine-information`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Retorna informações do sistema e métricas da máquina

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

# Se não estiver usando Docker Compose para a aplicação completa,
# é necessário iniciar o banco de dados PostgreSQL:
docker compose up db -d

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

## Autor

https://github.com/DiogoBrazil