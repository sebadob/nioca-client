use anyhow::Error;
use chrono::Utc;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

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
pub struct CertX509Response {
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
pub enum X509CertFormat {
    Pem,
    Der,
    PKCS12,
}

pub async fn fetch_cert_x509(
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
