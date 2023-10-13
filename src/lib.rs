use anyhow::Error;
use chrono::Utc;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;
use std::string::ToString;
use std::time::Duration;
#[cfg(feature = "cli")]
use tokio::fs;
use tracing::debug;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) static ERR_TIMEOUT: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| {
    env::var("ERROR_TIMEOUT")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()
        .expect("Cannot parse ERROR_TIMEOUT to u64")
});

#[cfg(feature = "actix")]
pub mod actix;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "cli")]
pub mod cli;

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
                        match fs::read_to_string(&try_root_pem_path).await {
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CertX509 {
    pub cert: String,
    pub cert_fingerprint: String,
    pub cert_chain: String,
    pub key: String,
    pub cert_format: X509CertFormat,
    /// not after as a unix timestamp in UTC format
    pub not_after: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct CertX509Response {
    pub cert: String,
    pub cert_fingerprint: String,
    pub cert_chain: String,
    pub key: String,
    pub cert_format: X509CertFormat,
    /// not after as a unix timestamp in UTC format
    pub not_after: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum X509CertFormat {
    Pem,
    Der,
    PKCS12,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NiocaErrorResponse {
    pub typ: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "ssh")]
pub struct SshCertificateResponse {
    pub user_ca_pub: String,
    pub host_key_pair: SshKeyPairOpenssh,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
#[cfg(feature = "ssh")]
pub struct SshKeyPairOpenssh {
    pub id: String,
    pub id_pub: String,
    pub alg: SshKeyAlg,
    pub typ: Option<SshCertType>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[allow(dead_code)]
#[cfg(feature = "ssh")]
pub enum SshKeyAlg {
    RsaSha256,
    RsaSha512,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(dead_code)]
#[cfg(feature = "ssh")]
pub enum SshCertType {
    Host,
    User,
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

pub(crate) fn req_client(root_cert: Option<reqwest::Certificate>) -> reqwest::Client {
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

pub(crate) fn auth_token(api_key: &str) -> String {
    format!("Bearer {}", api_key)
}

#[cfg(feature = "ssh")]
pub(crate) async fn fetch_cert_ssh(
    client: &reqwest::Client,
    url: &str,
    bearer: &str,
) -> anyhow::Result<SshCertificateResponse> {
    match client
        .post(url)
        .header(AUTHORIZATION, bearer)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();

            if status.is_success() {
                match resp.json::<SshCertificateResponse>().await {
                    Ok(kp) => Ok(kp),
                    Err(err) => {
                        let msg = format!(
                            "{} - Error deserializing response into SshCertificateResponse: {}",
                            status, err
                        );
                        Err(Error::msg(msg))
                    }
                }
            } else if status == reqwest::StatusCode::METHOD_NOT_ALLOWED {
                let msg = r#"
'405 Method Not Allowed' from Nioca Server.
This usually happens if Nioca is sealed. Check and unseal if necessary."#;
                Err(Error::msg(msg))
            } else {
                match resp.json::<ErrorResponse>().await {
                    Ok(err) => Err(Error::msg(err.message)),
                    Err(err) => {
                        let msg = format!(
                            "{} - Error deserializing response into ErrorResponse: {}",
                            status, err
                        );
                        Err(Error::msg(msg))
                    }
                }
            }
        }
        Err(err) => Err(Error::msg(format!(
            "Error fetching SSH certificate from Nioca: {}",
            err
        ))),
    }
}

pub(crate) async fn fetch_cert_x509(
    client: &reqwest::Client,
    url: &str,
    bearer: &str,
) -> anyhow::Result<(CertX509Response, u64)> {
    match client
        .post(url)
        .header(AUTHORIZATION, bearer)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<CertX509Response>().await {
                Ok(certs) => {
                    // Nioca returns the not_after in seconds
                    let now = Utc::now().timestamp();
                    let diff = certs.not_after.saturating_sub(now);
                    let renew = diff as u64 * 90 / 100;
                    Ok((certs, renew))
                }
                Err(err) => {
                    let msg = format!(
                        "{} - Error deserializing response into CertX509Response: {}",
                        status, err
                    );
                    Err(Error::msg(msg))
                }
            }
        }
        Err(err) => Err(Error::msg(format!(
            "Error fetching TLS certificate from Nioca: {}",
            err
        ))),
    }
}
