use crate::{auth_token, fetch_cert_x509, req_client, CertX509Response, NiocaConfig, ERR_TIMEOUT};
use anyhow::Error;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time;
use tracing::{error, info};

pub struct NiocaGeneric;

impl NiocaGeneric {
    pub fn spawn(config: NiocaConfig) -> anyhow::Result<watch::Receiver<Option<CertX509Response>>> {
        let (tx, rx) = watch::channel(None);

        let api_key = if let Some(key) = &config.api_key_x509 {
            key.to_string()
        } else {
            return Err(Error::msg("NIOCA_X509_API_KEY is not set"));
        };
        let url = if let Some(url) = &config.url_x509 {
            url.to_string()
        } else {
            return Err(Error::msg("NIOCA_X509_CLIENT_ID is not set"));
        };

        tokio::spawn(async move {
            let client = req_client(config.root_cert.clone());
            let bearer = auth_token(&api_key);
            let mut next_fetch;

            loop {
                match fetch_cert_x509(&client, &url, &bearer).await {
                    Ok((certs, not_after_sec)) => {
                        next_fetch = Some(not_after_sec);
                        if let Err(err) = tx.send(Some(certs)) {
                            error!("Sending CertX509Response: {:?}", err);
                        }
                    }
                    Err(err) => {
                        error!("{}", err);
                        next_fetch = None;
                    }
                }

                let sleep_sec = if let Some(n) = next_fetch {
                    n
                } else {
                    *ERR_TIMEOUT
                };

                info!("Fetching next certificate in {} seconds", sleep_sec);
                time::sleep(Duration::from_secs(sleep_sec)).await;
            }
        });

        Ok(rx)
    }
}
