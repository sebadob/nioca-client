use crate::{auth_token, fetch_cert_x509, req_client, CertX509Response, NiocaConfig, ERR_TIMEOUT};
use axum_server::tls_rustls::RustlsConfig;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time;
use tracing::{error, info};

pub async fn spawn(config: NiocaConfig) -> anyhow::Result<watch::Receiver<Option<RustlsConfig>>> {
    let (tx, rx) = watch::channel(None);

    let api_key = if let Some(key) = &config.api_key_x509 {
        key.to_string()
    } else {
        return Err(anyhow::Error::msg("NIOCA_X509_API_KEY is not set"));
    };
    let url = if let Some(url) = &config.url_x509 {
        url.to_string()
    } else {
        return Err(anyhow::Error::msg("NIOCA_X509_CLIENT_ID is not set"));
    };

    tokio::spawn(async move {
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

async fn send_config(certs: &CertX509Response, tx: &watch::Sender<Option<RustlsConfig>>) {
    let chain = format!("{}\n{}", certs.cert, certs.cert_chain);
    let chain_vec = chain.as_bytes().to_vec();
    let key_vec = certs.key.as_bytes().to_vec();

    let cfg = RustlsConfig::from_pem(chain_vec, key_vec)
        .await
        .expect("Building RustlsConfig from Nioca certs");

    tx.send(Some(cfg)).expect("Sending RustlsConfig");
}
