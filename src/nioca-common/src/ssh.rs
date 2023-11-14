use crate::ErrorResponse;
use anyhow::Error;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshCertificateResponse {
    pub user_ca_pub: String,
    pub host_key_pair: SshKeyPairOpenssh,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SshKeyAlg {
    RsaSha256,
    RsaSha512,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum SshCertType {
    Host,
    User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SshKeyPairOpenssh {
    pub id: String,
    pub id_pub: String,
    pub alg: SshKeyAlg,
    pub typ: Option<SshCertType>,
}

pub async fn fetch_cert_ssh(
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
