use crate::{auth_token, fetch_cert_x509, req_client, CertX509Response, NiocaConfig, ERR_TIMEOUT};
use anyhow::Error;
use der::Document;
use rustls::ServerConfig;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time;
use tracing::{error, info};

pub async fn spawn(config: NiocaConfig) -> anyhow::Result<watch::Receiver<Option<ServerConfig>>> {
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

    actix_web::rt::spawn(async move {
        let client = req_client(config.root_cert.clone());
        let bearer = auth_token(&api_key);
        let mut next_fetch;

        loop {
            match fetch_cert_x509(&client, &url, &bearer).await {
                Ok((certs, not_after_sec)) => {
                    next_fetch = Some(not_after_sec);
                    send_config(&certs, &tx).await;
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

async fn send_config(certs: &CertX509Response, tx: &watch::Sender<Option<ServerConfig>>) {
    let chain_doc = pem_to_der(&certs.cert_chain).unwrap();
    let chain = rustls::Certificate(chain_doc.to_vec());
    let cert_doc = pem_to_der(&certs.cert).unwrap();
    let cert = rustls::Certificate(cert_doc.to_vec());
    let certs_vec = vec![chain, cert];

    let key_doc = pem_to_der(&certs.key).unwrap();
    let key = rustls::PrivateKey(key_doc.to_vec());

    let cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs_vec, key)
        .map_err(|err| error!("Error building rustls ServerConfig: {}", err))
        .expect("bad certificate/key");

    tx.send(Some(cfg)).expect("Sending rustls::ServerConfig");
}

fn pem_to_der(pem: &str) -> anyhow::Result<Document> {
    match Document::from_pem(pem) {
        Ok(der) => Ok(der.1),
        Err(err) => {
            error!("{}", err);
            Err(Error::msg(err))
        }
    }
}
