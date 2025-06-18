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
- **Resposta em caso de sucesso**:
  ```json
  {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "user_id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
    "full_name": "Diogo Ribeiro",
    "email": "diogoifroads@gmail.com",
    "profile": "Administrador",
    "allowed_applications": [
        "upavision"
    ],
    "allowed_health_units": [
        2,
        3
    ]
  }
  ```
- **Resposta em caso de credenciais inválidas**:
  ```json
  {
    "error": "Unauthorized",
    "message": "Unauthorized: Invalid credentials",
    "status_code": 401
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "data": {
        "allowed_applications": [
            "xpredict"
        ],
        "allowed_health_units": [
            2
        ],
        "full_name": "DIOGO RIBEIRO",
        "profile": "Usuario Comum",
        "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
        "user_id": "4347000200000024"
    },
    "message": "Operation successful",
    "status": 200
  }
  ```
- **Resposta em caso de credenciais inválidas**:
  ```json
  {
    "error": "Unauthorized",
    "message": "Unauthorized: Incorrect password",
    "status_code": 401
  }
  ``` 
- **Resposta em caso do usuário PRONTO não ser médico**:
  ```json
  {
    "error": "Forbidden",
    "message": "Forbidden: User does not have the required profile",
    "status_code": 403
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "data": [
        {
            "allowed_applications": [
                "upavision",
                "xpredict"
            ],
            "allowed_health_units": [
                2
            ],
            "email": "fulano@gmail.com",
            "enabled": true,
            "full_name": "fulando de tal",
            "id": "8c15eb84-6e6e-455e-9c86-a8f4864b9e78",
            "profile": "Administrador"
        },
        {
            "allowed_applications": [
                "upavision"
            ],
            "allowed_health_units": [
                2
            ],
            "email": "beltrano@hotmail.com.br",
            "enabled": true,
            "full_name": "baltrano de tal",
            "id": "ccba2604-02ff-40cc-afa8-c1a0bbb7bb10",
            "profile": "Usuario Comum"
        }
    ],
    "message": "Operation successful",
    "status": 200
  }
  ```

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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "full_name": "João Silva",
      "email": "joao.silva@exemplo.com",
      "profile": "Administrador",
      "allowed_applications": ["xpredict", "upavision"],
      "allowed_health_units": [1, 2, 3],
      "enabled": true
    }
  }
  ```
- **Resposta em caso de email já existente**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding user: email 'joao.silva@exemplo.com' already exists",
    "status_code": 400
  }
  ```
- **Resposta em caso de email inválido**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding user: 'email-invalido' is not a valid email",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Cria novo usuário com perfil e aplicações permitidas

#### 3. Obter Usuário Específico
- **URL**: `/api/users/{id}`
- **Método**: GET
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "full_name": "João Silva",
      "email": "joao.silva@exemplo.com",
      "profile": "Administrador",
      "allowed_applications": ["xpredict", "upavision"],
      "allowed_health_units": [1, 2, 3],
      "enabled": true
    }
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "message": "User not found",
    "status": 404,
    "data": null
  }
  ```
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource updated successfully",
    "status": 200,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "full_name": "João Silva Atualizado",
      "email": "joao.silva@exemplo.com",
      "profile": "Usuario Comum",
      "allowed_applications": ["upavision"],
      "allowed_health_units": [1, 2],
      "enabled": true
    }
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error updating user: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Atualiza dados de um usuário específico

#### 5. Remover Usuário
- **URL**: `/api/users/{id}`
- **Método**: DELETE
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource deleted successfully",
    "status": 200,
    "data": null
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error deleting user: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Remove um usuário específico

#### 6. Enviar Código de Verificação
- **URL**: `/api/users/send-verification-code/{email}`
- **Método**: POST
- **Parâmetros de rota**: `email` (email do usuário)
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "id_verification": "123e4567-e89b-12d3-a456-426614174000",
      "user_id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "email": "joao.silva@exemplo.com",
      "verification_code": 123456,
      "used": false,
      "created_at": "2024-03-15T14:30:25Z",
      "expiration_at": "2024-03-15T14:40:25Z"
    }
  }
  ```
- **Resposta em caso de email inválido**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding user: 'email-invalido' is not a valid email",
    "status_code": 400
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: User not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de usuário desabilitado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: User disabled",
    "status_code": 400
  }
  ```
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
- **Resposta em caso de código ainda válido (reenvia o mesmo)**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "id_verification": "123e4567-e89b-12d3-a456-426614174000",
      "user_id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "email": "joao.silva@exemplo.com",
      "verification_code": 123456,
      "used": false,
      "created_at": "2024-03-15T14:30:25Z",
      "expiration_at": "2024-03-15T14:40:25Z"
    }
  }
  ```
- **Resposta em caso de código expirado (gera novo)**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "id_verification": "123e4567-e89b-12d3-a456-426614174000",
      "user_id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "email": "joao.silva@exemplo.com",
      "verification_code": 654321,
      "used": false,
      "created_at": "2024-03-15T15:00:25Z",
      "expiration_at": "2024-03-15T15:10:25Z"
    }
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: User not found",
    "status_code": 400
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "id_verification": "123e4567-e89b-12d3-a456-426614174000",
      "used": true,
      "updated_at": "2024-03-15T14:35:25Z"
    }
  }
  ```
- **Resposta em caso de ID de verificação não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Id verification not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de código incorreto**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Verification code not matched",
    "status_code": 400
  }
  ```
- **Resposta em caso de código já utilizado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Code already used",
    "status_code": 400
  }
  ```
- **Resposta em caso de código expirado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Code expired",
    "status_code": 400
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "user_id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "password_updated": true,
      "updated_at": "2024-03-15T14:40:25Z"
    }
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: User not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de ID de verificação não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Id verification not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de código não verificado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Code not verified",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Público
- **Descrição**: Atualiza senha para usuário que esqueceu, após verificação (requer código confirmado)

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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource updated successfully",
    "status": 200,
    "data": null
  }
  ```
- **Resposta em caso de campo vazio**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error updating password: current_password cannot be empty",
    "status_code": 400
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error updating password: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de senha atual incorreta**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error updating password: current password is incorrect",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Permite usuário atualizar sua própria senha (requer conhecimento da senha atual)

### Fluxo de Recuperação de Senha

Para recuperar a senha, o usuário deve seguir este fluxo sequencial:

1. **Solicitar código de verificação** via `send-verification-code/{email}`
   - Retorna `id_verification` que será usado nas próximas etapas
   - Código expira em 10 minutos
   - Email é enviado com código de 6 dígitos

2. **Confirmar código recebido** via `confirm-verification-code`
   - Utiliza o `verification_code` recebido por email
   - Utiliza o `id_verification` da etapa anterior
   - Marca o código como "usado" (validação para próxima etapa)

3. **Atualizar senha** via `update-password-for-forgetting-user/{id}`
   - Utiliza o mesmo `id_verification` (deve estar marcado como "usado")
   - Define a nova senha

**Observações importantes:**
- Cada etapa valida a anterior
- O código só pode ser usado uma vez
- Se o código expirar, use `resend-verification-code` para gerar um novo
- O fluxo de "usuário comum" (`update-password-by-user-common`) é independente e requer a senha atual

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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "user_name": "João Silva",
      "feedback": "sim",
      "prediction_made": "pneumonia viral",
      "correct_prediction": "pneumonia viral"
    }
  }
  ```
- **Resposta em caso de campo vazio**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error creating feedback: user_name cannot be empty",
    "status_code": 400
  }
  ```
- **Resposta em caso de feedback inválido**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error: 'talvez' is not a valid feedback. Allowed values are: sim, não",
    "status_code": 400
  }
  ```
- **Resposta em caso de doença inválida**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error: 'gripe' is not a respiratory diseases. Allowed values are: normal, covid-19, pneumonia viral, pneumonia bacteriana",
    "status_code": 400
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
    "feedback": "sim" // Valores: "sim" ou "não"
  }
  ```
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "user_name": "João Silva",
      "feedback": "sim"
    }
  }
  ```
- **Resposta em caso de campo vazio**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error creating feedback: user_name cannot be empty",
    "status_code": 400
  }
  ```
- **Resposta em caso de feedback inválido**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error: 'talvez' is not a valid feedback. Allowed values are: sim, não",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Registra feedback sobre predição de tuberculose

#### 13. Feedback de Osteoporose
- **URL**: `/api/users/feedback-osteoporosis`
- **Método**: POST
- **Corpo da requisição**:
  ```json
  {
    "user_name": "João Silva",
    "feedback": "sim", // Valores: "sim" ou "não"
    "prediction_made": "osteopenia", // Valores: "normal", "osteopenia", "osteoporosis"
    "correct_prediction": "osteopenia" // Valores: "normal", "osteopenia", "osteoporosis"
  }
  ```
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "user_name": "João Silva",
      "feedback": "sim",
      "prediction_made": "osteopenia",
      "correct_prediction": "osteopenia"
    }
  }
  ```
- **Resposta em caso de campo vazio**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error creating feedback: user_name cannot be empty",
    "status_code": 400
  }
  ```
- **Resposta em caso de predição inválida**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error: 'artrose' is not a valid feedback for osteoporosis. Allowed values are: osteopenia, osteoporosis, normal",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Registra feedback sobre predição de osteoporose


#### 14. Listar Feedbacks
- **URL**: `/api/users/feedbacks`
- **Método**: GET
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "feedbacks_respiratory_diseases": {
        "normal": {
          "total_quantity": 15,
          "total_quantity_correct": 12
        },
        "covid-19": {
          "total_quantity": 8,
          "total_quantity_correct": 6
        },
        "pneumonia viral": {
          "total_quantity": 10,
          "total_quantity_correct": 8
        },
        "pneumonia bacteriana": {
          "total_quantity": 5,
          "total_quantity_correct": 4
        }
      },
      "feedbacks_tuberculosis": {
        "total_quantity": 20,
        "total_quantity_correct": 18
      },
      "feedbacks_osteoporosis": {
        "osteopenia": {
          "total_quantity": 12,
          "total_quantity_correct": 10
        },
        "osteoporosis": {
          "total_quantity": 8,
          "total_quantity_correct": 7
        },
        "normal": {
          "total_quantity": 25,
          "total_quantity_correct": 23
        }
      }
    }
  }
  ```
- **Resposta em caso de feedbacks não encontrados**:
  ```json
  {
    "message": "Feedbacks not found",
    "status": 404,
    "data": null
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Retorna estatísticas processadas de todos os feedbacks registrados

#### 15. Atualizar Senha (Administrador)
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource updated successfully",
    "status": 200,
    "data": null
  }
  ```
- **Resposta em caso de campo vazio**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error updating password: email cannot be empty",
    "status_code": 400
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "message": "User not found",
    "status": 404,
    "data": null
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Permite administrador atualizar senha de qualquer usuário

#### 16. Adicionar Aplicações
- **URL**: `/api/users/{id}/applications`
- **Método**: POST
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "applications_name": ["xpredict", "upavision"]
  }
  ```
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "full_name": "João Silva",
      "email": "joao.silva@exemplo.com",
      "profile": "Administrador",
      "allowed_applications": ["xpredict", "upavision"],
      "allowed_health_units": [1, 2, 3],
      "enabled": true
    }
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding application: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de aplicação inválida**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error: 'app_invalida' is not a valid application. Allowed values are: xpredict, upavision",
    "status_code": 400
  }
  ```
- **Resposta em caso de aplicação já existente**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding application: application 'xpredict' already exists",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Adiciona aplicações ao usuário especificado

#### 17. Remover Aplicação
- **URL**: `/api/users/{id}/application/{application_name}`
- **Método**: DELETE
- **Parâmetros de rota**: `id` (UUID do usuário), `application_name` (nome da aplicação)
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource deleted successfully",
    "status": 200,
    "data": null
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error deleting application: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de aplicação não encontrada no usuário**:
  ```json
  {
    "message": "Application not found",
    "status": 404,
    "data": null
  }
  ```
- **Resposta em caso de última aplicação**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error deleting application: user must have at least one allowed application",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Remove uma aplicação específica do usuário

#### 18. Ativar/Desativar Usuário
- **URL**: `/api/users/{id}/update-enabled`
- **Método**: PATCH
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "enabled": true
  }
  ```
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource updated successfully",
    "status": 200,
    "data": null
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "message": "User not found",
    "status": 404,
    "data": null
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Ativa ou desativa um usuário específico

#### 19. Adicionar Unidades de Saúde
- **URL**: `/api/users/{id}/health-units`
- **Método**: POST
- **Parâmetros de rota**: `id` (UUID do usuário)
- **Corpo da requisição**:
  ```json
  {
    "health_units": [1, 2, 3]
  }
  ```
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
      "full_name": "João Silva",
      "email": "joao.silva@exemplo.com",
      "profile": "Administrador",
      "allowed_applications": ["xpredict", "upavision"],
      "allowed_health_units": [1, 2, 3],
      "enabled": true
    }
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding health units: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de lista vazia**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding health units: health_units list cannot be empty",
    "status_code": 400
  }
  ```
- **Resposta em caso de unidade inexistente**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding health units: health unit with id '999' does not exist",
    "status_code": 400
  }
  ```
- **Resposta em caso de unidade já existente**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error adding health units: health unit with id '2' already exists",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Adiciona unidades de saúde permitidas ao usuário

#### 20. Remover Unidade de Saúde
- **URL**: `/api/users/{id}/health-unit/{health_unit_id}`
- **Método**: DELETE
- **Parâmetros de rota**: `id` (UUID do usuário), `health_unit_id` (ID da unidade de saúde)
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource deleted successfully",
    "status": 200,
    "data": null
  }
  ```
- **Resposta em caso de usuário não encontrado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error deleting health unit: user with id 'abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648' not found",
    "status_code": 400
  }
  ```
- **Resposta em caso de última unidade**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error deleting health unit: user must have at least one health unit",
    "status_code": 400
  }
  ```
- **Resposta em caso de usuário sem acesso à unidade**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Error deleting health unit: user does not have access to health unit with id '3'",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Remove uma unidade de saúde específica do usuário

### Dados UPA

#### 1. Adicionar Arquivo de Dados
- **URL**: `/api/data/add-file`
- **Método**: POST
- **Corpo da requisição**: Multipart form com arquivo CSV/Excel
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Resource created successfully",
    "status": 201,
    "data": {
      "message": "Dados processados e importados com sucesso",
      "rows_processed": 1500,
      "columns_processed": 15,
      "competencia_values": ["2024-01", "2024-02", "2024-03"]
    }
  }
  ```
- **Resposta em caso de nenhum arquivo enviado**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Nenhum arquivo foi enviado",
    "status_code": 400
  }
  ```
- **Resposta em caso de dados duplicados**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Dados do período 2024-01 já existem no banco",
    "status_code": 400
  }
  ```
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": [
      {
        "id": 2,
        "name": "UPA ARIQUEMES"
      },
      {
        "id": 3,
        "name": "UPA JARU"
      }
    ]
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Retorna lista de unidades de saúde disponíveis no sistema

#### 4. Número de Atendimentos por Mês
- **URL**: `/api/data/user/{user_id}/unit/{unidade_id}/number-of-appointments-per-month`
- **Método**: GET
- **Parâmetros de rota**: `user_id` (ID do usuário), `unidade_id` (ID da unidade)
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "2024-01": 1250,
      "2024-02": 1180,
      "2024-03": 1320,
      "2024-04": 1095
    }
  }
  ```
- **Resposta em caso de usuário sem acesso à unidade**:
  ```json
  {
    "error": "Forbidden",
    "message": "Forbidden: User does not have access to unit 2",
    "status_code": 403
  }
  ```
- **Resposta em caso de dados não encontrados**:
  ```json
  {
    "error": "Not Found",
    "message": "Not Found: Não há dados processados para a unidade 2 em visualization_data_graph",
    "status_code": 404
  }
  ```
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
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "prediction": {
        "covid-19": 0.15,
        "normal": 0.65,
        "pneumonia bacteriana": 0.05,
        "pneumonia viral": 0.15
      }
    }
  }
  ```
- **Resposta em caso de nenhuma imagem enviada**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Nenhuma imagem enviada",
    "status_code": 400
  }
  ```
- **Resposta em caso de erro na API ML**:
  ```json
  {
    "error": "Internal Server Error",
    "message": "Internal Server Error",
    "status_code": 500
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e prediz possível doença respiratória

#### 2. Detectar Câncer de Mama
- **URL**: `/api/prediction/detect`
- **Método**: POST
- **Corpo da requisição**: Multipart form com imagem de mamografia
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "detections": [
        {
          "class_id": 1,
          "confidence": 0.92,
          "bbox": [150, 200, 250, 350]
        }
      ],
      "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA..."
    }
  }
  ```
- **Resposta em caso de nenhuma imagem enviada**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Nenhuma imagem enviada",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e detecta possível câncer de mama

#### 3. Predizer Tuberculose
- **URL**: `/api/prediction/predict_tb`
- **Método**: POST
- **Corpo da requisição**: Multipart form com imagem de raio-X torácico
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "class_pred": "positive",
      "probabilities": {
        "negative": 0.25,
        "positive": 0.75
      }
    }
  }
  ```
- **Resposta em caso de nenhuma imagem enviada**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Nenhuma imagem enviada",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e prediz possível tuberculose

#### 4. Predizer Osteoporose
- **URL**: `/api/prediction/predict_osteoporosis`
- **Método**: POST
- **Corpo da requisição**: Multipart form com imagem de raio-X da canela ou joelho
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "class_pred": "osteopenia",
      "probabilities": {
        "normal": 0.15,
        "osteopenia": 0.65,
        "osteoporosis": 0.20
      }
    }
  }
  ```
- **Resposta em caso de nenhuma imagem enviada**:
  ```json
  {
    "error": "Bad Request",
    "message": "Bad Request: Nenhuma imagem enviada",
    "status_code": 400
  }
  ```
- **Nível de acesso**: Usuário Comum ou Administrador
- **Descrição**: Analisa imagem e prediz possível osteoporose

### Informação e Auditoria

#### 1. Obter Registros de Auditoria por página
- **URL**: `/api/information/audits/{page}`
- **Método**: GET
- **Parâmetros de rota**: `page` (número da página)
- **Parâmetros de query**:
  - `email` (opcional): filtrar por email do usuário
  - `path` (opcional): filtrar por caminho da requisição
  - `date_of_request` (opcional): filtrar por data da requisição
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "audits": [
        {
          "id": "abb63cbe-e8ba-4ca8-9e9d-ede6f674eb648",
          "user_email": "admin@exemplo.com",
          "user_profile": "Administrador",
          "method": "GET",
          "path": "/api/users",
          "ip": "192.168.1.100",
          "date_of_request": "2024-03-15",
          "hour_of_request": "14:30:25"
        }
      ],
      "pagination": {
        "current_page": 1,
        "total_pages": 10,
        "total_records": 150,
        "records_per_page": 15
      }
    }
  }
  ```
- **Resposta em caso de nenhum registro encontrado**:
  ```json
  {
    "error": "Not Found",
    "message": "Not Found: No audits found",
    "status_code": 404
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Retorna registros de auditoria paginados

#### 2. Obter Filtros de Auditoria
- **URL**: `/api/information/audits/audit-filters`
- **Método**: GET
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "available_data": {
        "user_email": [
          "admin@exemplo.com",
          "usuario@exemplo.com"
        ],
        "path": [
          "/api/users",
          "/api/auth/login",
          "/api/data/add-file"
        ],
        "method": [
          "GET",
          "POST",
          "PUT",
          "DELETE"
        ],
        "date_of_request": [
          "2024-03-15",
          "2024-03-14",
          "2024-03-13"
        ]
      }
    }
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Retorna opções de filtros disponíveis para auditoria

### 3. Obter todos os Registros de Auditoria
- **URL**: `/api/information/audits`
- **Método**: GET
- **Nível de acesso**: Administrador
- **Descrição**: Retorna todos os registros de auditoria

### Informações da Máquina

#### 1. Obter Métricas do Sistema
- **URL**: `/api/machine-information`
- **Método**: GET
- **Resposta em caso de sucesso**:
  ```json
  {
    "message": "Operation successful",
    "status": 200,
    "data": {
      "cpu": {
        "name": "Intel(R) Core(TM) i7-8750H CPU @ 2.20GHz",
        "architecture": "x86_64",
        "physical_cores": 6,
        "logical_cores": 12,
        "percent": 45.2,
        "temperature": "65°C"
      },
      "memory": {
        "total_gb": 16.0,
        "available_gb": 8.5,
        "used_gb": 7.5,
        "percent": 46.9,
        "free_percent": 53.1
      },
      "disk": {
        "total_gb": 512.0,
        "used_gb": 256.3,
        "free_gb": 255.7,
        "percent": 50.1,
        "free_percent": 49.9
      },
      "network": {
        "ip": "192.168.1.100"
      },
      "uptime": "2 dias, 14 horas, 32 minutos"
    }
  }
  ```
- **Nível de acesso**: Administrador
- **Descrição**: Retorna informações do sistema e métricas da máquina

## Códigos de Erro Padrão

A API utiliza uma estrutura padronizada para retorno de erros. Todos os erros seguem o formato:

```json
{
  "error": "Tipo do Erro",
  "message": "Mensagem detalhada do erro",
  "status_code": 400/401/403/404/405/500
}
```

### Códigos de Status HTTP Utilizados

- **200 OK**: Operação realizada com sucesso
- **201 CREATED**: Recurso criado com sucesso
- **400 BAD REQUEST**: Erro nos dados enviados pelo cliente
  - Campos obrigatórios vazios
  - Formato de dados inválido
  - Validações de negócio falharam
  - Recursos duplicados
- **401 UNAUTHORIZED**: Credenciais inválidas ou ausentes
  - Login com credenciais incorretas
  - Token JWT inválido ou expirado
- **403 FORBIDDEN**: Acesso negado
  - Usuário sem permissões para a operação
  - Acesso negado a unidades de saúde específicas
  - Perfil de usuário insuficiente
- **404 NOT FOUND**: Recurso não encontrado
  - Usuário, dados ou registros inexistentes
  - Endpoints não existentes
- **405 METHOD NOT ALLOWED**: Método HTTP não permitido
- **500 INTERNAL SERVER ERROR**: Erro interno do servidor
  - Falhas de banco de dados
  - Erros na API de Machine Learning
  - Problemas de processamento interno

### Estrutura de Resposta de Sucesso

Respostas de sucesso seguem o padrão:

```json
{
  "message": "Mensagem de sucesso",
  "status": 200/201,
  "data": { /* dados retornados */ }
}
```

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