use crate::DEV_MODE;
use leptos::leptos_config::Env;
use leptos::LeptosOptions;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tracing::warn;

pub type AppState = axum::extract::Extension<Arc<Config>>;

#[derive(Debug)]
pub struct Config {
    // pub db: DbPool,
    pub leptos_options: LeptosOptions,
}

impl Config {
    pub async fn new(port: u16) -> anyhow::Result<Arc<Self>> {
        // let db = database::new_pool().await?;

        let env = if env::var("DEV_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .expect("Cannot parse DEV_MODE to bool")
        {
            DEV_MODE.set(true).unwrap();
            warn!("Running in DEV_MODE");
            Env::DEV
        } else {
            DEV_MODE.set(false).unwrap();
            Env::PROD
        };

        // Setting get_configuration(None) means we'll be using cargo-leptos's env values
        // For deployment these variables are:
        // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
        // Alternately a file can be specified such as Some("Cargo.toml")
        // The file would need to be included with the executable when moved to deployment
        let site_addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port);
        let leptos_options = LeptosOptions::builder()
            .env(env)
            .output_name("nioca-client")
            .site_root("target/site")
            .site_pkg_dir("pkg")
            .site_addr(site_addr)
            .build();
        // let conf = get_configuration(None).await.unwrap();
        // let mut leptos_options = conf.leptos_options;
        // leptos_options.site_addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port);

        let slf = Self { leptos_options };

        Ok(Arc::new(slf))
    }
}
