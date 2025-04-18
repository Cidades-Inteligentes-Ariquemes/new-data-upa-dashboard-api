use async_trait::async_trait;
use std::error::Error;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use crate::domain::models::auth_pronto::{UserPronto, ProfileInfo};
use crate::domain::repositories::auth_pronto::AuthProntoRepository;
use crate::utils::config_env::Config as AppConfig;
use log::{error, info};

pub struct SqlServerAuthProntoRepository {
    config: AppConfig,
}

impl SqlServerAuthProntoRepository {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    async fn get_client(&self) -> Result<Client<tokio_util::compat::Compat<TcpStream>>, Box<dyn Error + Send + Sync>> {
        let mut config = Config::new();

        let server_port = format!("{}:{}", self.config.server, self.config.port);
        config.host(&server_port);
        config.database(&self.config.database);
        config.authentication(tiberius::AuthMethod::sql_server(
            &self.config.user_pronto,
            &self.config.password,
        ));
        config.trust_cert();

        let tcp = TcpStream::connect(server_port).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(config, tcp.compat_write()).await?;
        Ok(client)
    }
}

#[async_trait]
impl AuthProntoRepository for SqlServerAuthProntoRepository {
    async fn get_user_pronto_by_username_with_fullname(&self, username: &str) -> Result<Option<UserPronto>, Box<dyn Error + Send + Sync>> {
        let mut client = match self.get_client().await {
            Ok(client) => client,
            Err(e) => {
                error!("Error connecting to SQL Server: {}", e);
                return Err(e);
            }
        };

        let query = "SELECT 
                        Login.LoginCodigo, 
                        Login.LoginSenha, 
                        CAST(Login.LoginOUsuario AS VARCHAR(50)) AS LoginOUsuario, 
                        CAST(Login.LoginId AS VARCHAR(50)) AS LoginId, 
                        Usuario.UsuarioNome
                    FROM Login
                    JOIN Usuario ON Login.LoginOUsuario = Usuario.UsuarioId
                    WHERE Login.LoginCodigo = @P1";

        let stream = client
            .query(query, &[&username])
            .await?;

        let result = stream
            .into_first_result()
            .await?;

        if result.is_empty() {
            return Ok(None);
        }

        let row = &result[0];

        // Logando os tipos das colunas para debug
        info!("Coluna 0 (LoginCodigo): {:?}", row.try_get::<&str, _>(0));
        info!("Coluna 1 (LoginSenha): {:?}", row.try_get::<&str, _>(1));
        info!("Coluna 2 (LoginOUsuario): {:?}", row.try_get::<&str, _>(2));
        info!("Coluna 3 (LoginId): {:?}", row.try_get::<&str, _>(3));
        info!("Coluna 4 (UsuarioNome): {:?}", row.try_get::<&str, _>(4));

        let user = UserPronto {
            username: row.get::<&str, _>(0).unwrap_or_default().to_string(),
            password_pronto: row.get::<&str, _>(1).unwrap_or_default().to_string(),
            userid: row.get::<&str, _>(2).unwrap_or_default().to_string(),
            login_id: row.get::<&str, _>(3).unwrap_or_default().to_string(),
            fullname: row.get::<&str, _>(4).unwrap_or_default().to_string(),
        };

        info!("User found by username: {}, userid: {}, login_id: {}", 
              user.username, user.userid, user.login_id);
        Ok(Some(user))
    }

    async fn get_user_profiles_by_login_and_unit_id(&self, login_id: &str, unit_id: i32) -> Result<Vec<ProfileInfo>, Box<dyn Error + Send + Sync>> {
        let mut client = match self.get_client().await {
            Ok(client) => client,
            Err(e) => {
                error!("Error connecting to SQL Server: {}", e);
                return Err(e);
            }
        };
    
        info!("Buscando perfis para login_id: {} e unit_id: {}", login_id, unit_id);
    
        // Modificando a consulta para usar VARCHAR e converter todos os campos numéricos para VARCHAR
        let query = "SELECT
                    CAST(Perfil.PerfilId AS VARCHAR(50)) AS PerfilId,
                    Perfil.PerfilNome,
                    Login.LoginCodigo,
                    Usuario.UsuarioNome,
                    CAST(LoginAlocacao.UnidadeId AS VARCHAR(50)) AS UnidadeId
                FROM
                    Perfil
                JOIN
                    LoginPerfil ON Perfil.PerfilId = LoginPerfil.PerfilId
                JOIN
                    LoginAlocacao ON LoginPerfil.LoginAlocacaoId = LoginAlocacao.LoginAlocacaoId
                JOIN
                    Login ON LoginAlocacao.LoginId = Login.LoginId
                JOIN
                    Usuario ON Login.LoginOUsuario = Usuario.UsuarioId
                WHERE
                    CAST(LoginAlocacao.LoginId AS VARCHAR(50)) = @P1
                    AND LoginAlocacao.UnidadeId = @P2";
    
        let stream = client
            .query(query, &[&login_id, &unit_id])
            .await?;
    
        let result = stream
            .into_first_result()
            .await?;
    
        let mut profiles = Vec::new();
        for row in result {
            // Log para debug
            info!("Perfil - Coluna 0 (PerfilId): {:?}", row.try_get::<&str, _>(0));
            info!("Perfil - Coluna 1 (PerfilNome): {:?}", row.try_get::<&str, _>(1));
            info!("Perfil - Coluna 2 (LoginCodigo): {:?}", row.try_get::<&str, _>(2));
            info!("Perfil - Coluna 3 (UsuarioNome): {:?}", row.try_get::<&str, _>(3));
            info!("Perfil - Coluna 4 (UnidadeId): {:?}", row.try_get::<&str, _>(4));
            
            // Tratando todos os campos como string e convertendo para números quando necessário
            let profile = ProfileInfo {
                perfil_id: row.get::<&str, _>(0)
                    .unwrap_or_default()
                    .parse::<i32>()
                    .unwrap_or_default(),
                perfil_nome: row.get::<&str, _>(1).unwrap_or_default().to_string(),
                login_codigo: row.get::<&str, _>(2).unwrap_or_default().to_string(),
                usuario_nome: row.get::<&str, _>(3).unwrap_or_default().to_string(),
                unidade_id: row.get::<&str, _>(4)
                    .unwrap_or_default()
                    .parse::<i32>()
                    .unwrap_or_default(),
            };
            profiles.push(profile);
        }
    
        info!("Found {} profiles for login_id: {}, unit_id: {}", profiles.len(), login_id, unit_id);
        
        // Adicionando log mais detalhado para debug
        if profiles.is_empty() {
            info!("No profiles found for user with login_id: {}", login_id);
        } else {
            for (i, profile) in profiles.iter().enumerate() {
                info!("Profile {}: id={}, nome={}", i, profile.perfil_id, profile.perfil_nome);
            }
        }
        
        Ok(profiles)
    }
}