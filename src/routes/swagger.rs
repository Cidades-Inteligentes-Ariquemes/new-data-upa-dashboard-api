use actix_web::{get, HttpResponse, Responder, web};
use std::fs;

/// Configure routes for swagger documentation
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/swagger")
            .service(get_swagger_ui)
            .service(get_swagger_yaml)
            .service(check_yaml)
            .service(get_swagger_simple)
            .service(get_raw_yaml)
    );
}

/// Serve the raw YAML content for debugging
#[get("/raw")]
async fn get_raw_yaml() -> impl Responder {
    use log::info;
    let yaml_path = "/home/diogo/DEV/new-data-upa-dashboard-api/swagger.yaml";
    
    info!("Requisição para /api/swagger/raw recebida");
    
    match fs::read_to_string(yaml_path) {
        Ok(contents) => {
            HttpResponse::Ok()
                .content_type("text/plain; charset=utf-8")
                .body(contents)
        },
        Err(e) => {
            HttpResponse::NotFound().body(format!("Swagger YAML file not found: {}", e))
        }
    }
}

/// Serve a simple Swagger UI
#[get("/simple")]
async fn get_swagger_simple() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(r#"
            <!DOCTYPE html>
            <html>
            <head>
              <meta charset="UTF-8">
              <title>UPA Dashboard API - Swagger UI (Simples)</title>
              <link rel="stylesheet" type="text/css" href="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/3.52.0/swagger-ui.min.css" />
            </head>
            <body>
              <div id="swagger-ui"></div>
              <script src="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/3.52.0/swagger-ui-bundle.js"></script>
              <script>
                const ui = SwaggerUIBundle({
                  url: '/api/swagger/swagger.yaml',
                  dom_id: '#swagger-ui',
                  presets: [
                    SwaggerUIBundle.presets.apis
                  ],
                  layout: "BaseLayout"
                });
              </script>
            </body>
            </html>
        "#)
}

#[get("/check")]
async fn check_yaml() -> impl Responder {
    use log::info;
    let yaml_path = "/home/diogo/DEV/new-data-upa-dashboard-api/swagger.yaml";
    
    match std::fs::metadata(yaml_path) {
        Ok(metadata) => {
            info!("Arquivo swagger.yaml existe, tamanho: {} bytes", metadata.len());
            if let Ok(contents) = fs::read_to_string(yaml_path) {
                info!("Primeiros 100 caracteres: {}", &contents[..100.min(contents.len())]);
                HttpResponse::Ok().body(format!("Arquivo swagger.yaml existe, tamanho: {} bytes", metadata.len()))
            } else {
                info!("Erro ao ler o arquivo");
                HttpResponse::InternalServerError().body("Erro ao ler o arquivo")
            }
        },
        Err(e) => {
            info!("Erro ao acessar o arquivo: {}", e);
            HttpResponse::NotFound().body(format!("Arquivo não encontrado: {}", e))
        }
    }
}

/// Serve the Swagger UI
#[get("")]
async fn get_swagger_ui() -> impl Responder {
    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>UPA Dashboard API - Swagger UI</title>
        <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@3.52.0/swagger-ui.css">
        <style>
            html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
            *, *:before, *:after { box-sizing: inherit; }
            body { margin: 0; background: #fafafa; }
        </style>
    </head>
    <body>
        <div id="swagger-ui"></div>
        <script src="https://unpkg.com/swagger-ui-dist@3.52.0/swagger-ui-bundle.js"></script>
        <script>
            window.onload = function() {
                console.log("Carregando Swagger UI...");
                
                // Função para pegar o valor de um parâmetro da URL
                function getQueryParam(name) {
                    const urlParams = new URLSearchParams(window.location.search);
                    return urlParams.get(name);
                }
                
                // API Key do parâmetro URL (opcional)
                const apiKeyFromUrl = getQueryParam('apiKey');
                
                const ui = SwaggerUIBundle({
                    url: "/api/swagger/swagger.yaml",
                    dom_id: '#swagger-ui',
                    deepLinking: true,
                    presets: [
                        SwaggerUIBundle.presets.apis
                    ],
                    plugins: [
                        SwaggerUIBundle.plugins.DownloadUrl
                    ],
                    layout: "BaseLayout",
                    docExpansion: 'list'
                });
                
                // Se tiver uma API key na URL, aplica automaticamente
                if (apiKeyFromUrl) {
                    ui.preauthorizeApiKey("apiKeyAuth", apiKeyFromUrl);
                }
                
                // Adiciona um campo para inserir a API key manualmente
                setTimeout(function() {
                    const header = document.querySelector('.swagger-ui .topbar');
                    if (header) {
                        const apiKeyInput = document.createElement('div');
                        apiKeyInput.style.display = 'flex';
                        apiKeyInput.style.alignItems = 'center';
                        apiKeyInput.style.margin = '0 10px';
                        
                        apiKeyInput.innerHTML = '<input id="api_key_input" type="text" placeholder="API Key" style="margin-right: 5px; padding: 5px; border-radius: 4px; border: 1px solid #ccc;"> <button id="apply_key" style="padding: 5px 10px; border-radius: 4px; background: #89bf04; color: white; border: none;">Aplicar</button>';
                        
                        header.appendChild(apiKeyInput);
                        
                        document.getElementById('apply_key').addEventListener('click', function() {
                            const apiKey = document.getElementById('api_key_input').value;
                            if (apiKey) {
                                ui.preauthorizeApiKey("apiKeyAuth", apiKey);
                                alert('API Key aplicada!');
                            }
                        });
                    }
                }, 1000);
            }
        </script>
    </body>
    </html>
    "#;
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// Serve the swagger.yaml file
#[get("/swagger.yaml")]
async fn get_swagger_yaml() -> impl Responder {
    use log::info;
    let yaml_path = "/home/diogo/DEV/new-data-upa-dashboard-api/swagger.yaml";
    
    info!("Requisição para /api/swagger/swagger.yaml recebida");
    
    match fs::read_to_string(yaml_path) {
        Ok(contents) => {
            info!("Arquivo swagger.yaml carregado com sucesso, enviando {} bytes", contents.len());
            HttpResponse::Ok()
                .content_type("application/x-yaml")
                .insert_header(("Access-Control-Allow-Origin", "*"))
                .body(contents)
        },
        Err(e) => {
            info!("Erro ao ler o arquivo swagger.yaml: {}", e);
            HttpResponse::NotFound().body(format!("Swagger YAML file not found: {}", e))
        }
    }
}