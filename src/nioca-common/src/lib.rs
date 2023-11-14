use serde::Deserialize;
use std::env;
use std::time::Duration;
use tracing::debug;

#[cfg(feature = "ssh")]
pub mod ssh;

pub mod x509;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub static ERR_TIMEOUT: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| {
    env::var("ERROR_TIMEOUT")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()
        .expect("Cannot parse ERROR_TIMEOUT to u64")
});

#[inline]
pub fn auth_token(api_key: &str) -> String {
    format!("Bearer {}", api_key)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub typ: ErrorResponseType,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ErrorResponseType {
    BadRequest,
    Connection,
    Database,
    DatabaseIo,
    Forbidden,
    Internal,
    InvalidToken,
    NotFound,
    Unauthorized,
    ServiceUnavailable,
    TooManyRequests,
}

#[derive(Debug, Clone)]
pub struct NiocaConfig {
    pub url: String,
    pub url_ssh: Option<String>,
    pub url_x509: Option<String>,
    pub root_cert: Option<reqwest::Certificate>,
    pub root_pem: Option<String>,
    pub api_key_ssh: Option<String>,
    pub api_key_x509: Option<String>,
}

impl NiocaConfig {
    pub async fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let mut url = env::var("NIOCA_URL").expect("NIOCA_URL is not set");
        if url.ends_with('/') {
            let _ = url.split_off(url.len() - 1);
        }

        let api_key_x509 = env::var("NIOCA_X509_API_KEY").map(Some).unwrap_or(None);
        let client_id_x509 = env::var("NIOCA_X509_CLIENT_ID").map(Some).unwrap_or(None);
        let url_x509 = client_id_x509.map(|id| format!("{}/api/clients/x509/{}/cert", url, id));

        let api_key_ssh = env::var("NIOCA_SSH_API_KEY").map(Some).unwrap_or(None);
        let client_id_ssh = env::var("NIOCA_SSH_CLIENT_ID").map(Some).unwrap_or(None);
        let url_ssh = client_id_ssh.map(|id| format!("{}/api/clients/ssh/{}/cert", url, id));

        let (root_pem, root_cert) = match env::var("NIOCA_ROOT_PEM") {
            Ok(root_pem) => {
                let root_cert = reqwest::tls::Certificate::from_pem(root_pem.as_bytes())
                    .expect("Cannot build Root TLS from given NIOCA_ROOT_PEM");
                (Some(root_pem), Some(root_cert))
            }
            #[cfg(not(feature = "cli"))]
            Err(_) => (None, None),
            #[cfg(feature = "cli")]
            Err(_) => {
                // if we do not have a configured env var, try to find an existing root PEM in
                // the nioca config dir
                match home::home_dir() {
                    Some(path) => {
                        let try_root_pem_path = format!("{}/.nioca/root.pem", path.display());
                        match tokio::fs::read_to_string(&try_root_pem_path).await {
                            Ok(root_pem) => {
                                let root_cert =
                                    reqwest::tls::Certificate::from_pem(root_pem.as_bytes())
                                        .expect("Cannot build Root TLS from given NIOCA_ROOT_PEM");
                                (Some(root_pem), Some(root_cert))
                            }
                            Err(_) => (None, None),
                        }
                    }
                    None => (None, None),
                }
            }
        };

        debug!("Nioca URL: {}", url);
        Self {
            url,
            url_ssh,
            url_x509,
            root_cert,
            root_pem,
            api_key_ssh,
            api_key_x509,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NiocaErrorResponse {
    pub typ: String,
    pub message: String,
}

pub fn req_client(root_cert: Option<reqwest::Certificate>) -> reqwest::Client {
    let mut client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .https_only(true)
        .user_agent(format!("Nioca Client {}", VERSION));

    if let Some(root_cert) = root_cert {
        client = client.add_root_certificate(root_cert);
    }

    client
        .build()
        .expect("Building reqwest client for fetch_cert")
}
