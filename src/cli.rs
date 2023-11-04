use crate::{
    auth_token, fetch_cert_ssh, fetch_cert_x509, req_client, CertX509Response, NiocaConfig,
    SshCertType, SshCertificateResponse, ERR_TIMEOUT, VERSION,
};
use chrono::NaiveDateTime;
use clap::{arg, Parser};
use std::fmt::Write;
use std::io::ErrorKind;
use std::ops::Sub;
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs::File;
use tokio::process::Command;
use tokio::{fs, time};

#[cfg(target_family = "unix")]
const FILE_NAME_CONFIG: &str = "config";
#[cfg(not(target_family = "unix"))]
const FILE_NAME_CONFIG: &str = "config.txt";

#[cfg(target_family = "unix")]
const FILE_NAME_EXE: &str = "nioca-client";
#[cfg(not(target_family = "unix"))]
const FILE_NAME_EXE: &str = "nioca-client.exe";

#[cfg(target_family = "unix")]
const SEPARATOR: &str = "/";
#[cfg(not(target_family = "unix"))]
const SEPARATOR: &str = "\\";

/// This client fetches TLS certificates from Nioca
#[derive(Debug, PartialEq, Parser)]
#[command(author, version, about)]
pub(crate) enum CliArgs {
    /// Installs the nioca-client into the system
    Install,
    /// Installs the nioca-client systemd service into the system
    #[cfg(target_family = "unix")]
    InstallService,
    /// Uninstalls the nioca-client from the system
    Uninstall,
    FetchRoot(CmdFetchRoot),
    Daemonize(CmdDaemonize),
    Ssh(CmdSsh),
    X509(CmdX509),
}

/// Fetch the current Nioca root certificate with the given fingerprint SHA256 hash
#[derive(Debug, PartialEq, Parser)]
#[command(author, version)]
pub(crate) struct CmdFetchRoot {
    #[cfg(target_family = "unix")]
    /// Path to the config file (default $HOME/.nioca/config)
    pub config: Option<String>,

    #[cfg(not(target_family = "unix"))]
    /// Path to the config file (default $HOME\.nioca\config.txt)
    #[arg(short, long)]
    pub config: Option<String>,

    #[cfg(target_family = "unix")]
    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "./certs")]
    pub destination: Option<String>,

    #[cfg(not(target_family = "unix"))]
    /// Output path for the fetched certificates
    #[arg(short, long, default_value = ".\\certs")]
    pub destination: Option<String>,

    /// Fetches the root CA's certificate and verifies it with the given fingerprint
    #[arg(short, long)]
    pub fingerprint: String,
}

/// Run the client continuously as a daemon to always renew expiring certificates
#[derive(Debug, PartialEq, Parser)]
#[command(author, version)]
pub(crate) struct CmdDaemonize {
    #[cfg(target_family = "unix")]
    /// Path to the config file (default $HOME/.nioca/config)
    #[arg(short, long)]
    pub config: Option<String>,

    #[cfg(not(target_family = "unix"))]
    /// Path to the config file (default $HOME\.nioca\config.txt)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "./certs")]
    pub destination: String,
}

/// Fetch an SSH certificate
#[derive(Debug, PartialEq, Parser)]
#[command(author, version)]
pub(crate) struct CmdSsh {
    #[cfg(target_family = "unix")]
    /// Path to the config file (default $HOME/.nioca/config)
    #[arg(short, long)]
    pub config: Option<String>,

    #[cfg(not(target_family = "unix"))]
    /// Path to the config file (default $HOME\.nioca\config.txt)
    #[arg(short, long)]
    pub config: Option<String>,

    #[cfg(target_family = "unix")]
    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "./certs")]
    pub destination: String,

    #[cfg(not(target_family = "unix"))]
    /// Output path for the fetched certificates
    #[arg(short, long, default_value = ".\\certs")]
    pub destination: String,

    /// Installs SSH certificates and keys after a successful fetch if the CertType is 'Host'.
    /// Adds the Public Key to known_hosts if the CertType is 'User'.
    #[arg(short = 'i', long, default_value = "true")]
    pub install: bool,
}

/// Fetch a X509 certificate
#[derive(Debug, PartialEq, Parser)]
#[command(author, version)]
pub(crate) struct CmdX509 {
    #[cfg(target_family = "unix")]
    /// Path to the config file (default $HOME/.nioca/config)
    #[arg(short, long)]
    pub config: Option<String>,

    #[cfg(not(target_family = "unix"))]
    /// Path to the config file (default $HOME\.nioca\config.txt)
    #[arg(short, long)]
    pub config: Option<String>,

    #[cfg(target_family = "unix")]
    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "./certs")]
    pub destination: String,

    #[cfg(not(target_family = "unix"))]
    /// Output path for the fetched certificates
    #[arg(short, long, default_value = ".\\certs")]
    pub destination: String,
}

pub async fn execute() -> anyhow::Result<()> {
    let args: CliArgs = CliArgs::parse();
    match args {
        CliArgs::Install => {
            install_on_sys().await?;
        }
        #[cfg(target_family = "unix")]
        CliArgs::InstallService => {
            // check if nioca-client is already installed globally and install if not
            let path = "/usr/local/bin/nioca-client";
            if fs::try_exists(&path).await.is_err() {
                install_on_sys().await?;
            }
            install_systemd_service().await?;
        }
        CliArgs::Uninstall => {
            uninstall_from_sys().await?;
        }
        CliArgs::FetchRoot(cmd) => {
            fetch_root_ca(&cmd).await?;
        }
        CliArgs::Daemonize(cmd) => {
            daemonize(&cmd).await?;
        }
        CliArgs::Ssh(cmd) => {
            fetch_ssh(cmd, false).await?;
        }
        CliArgs::X509(cmd) => fetch_x509(cmd, false).await?,
    }

    Ok(())
}

async fn get_config(path: &Option<String>) -> NiocaConfig {
    if let Some(cfg) = path {
        dotenvy::from_filename(cfg).ok();
    } else {
        let home = match home::home_dir() {
            Some(path) => path,
            None => panic!("Cannot get home directory"),
        };
        let config_dir = format!(
            "{}{}.nioca{}{}",
            home.display(),
            SEPARATOR,
            SEPARATOR,
            FILE_NAME_CONFIG
        );
        match dotenvy::from_filename(config_dir) {
            Ok(_env) => {}
            Err(_) => {
                panic!("Cannot read nioca-client config - Please install first");
            }
        }
    }
    NiocaConfig::from_env().await
}

#[cfg(target_family = "unix")]
async fn install_on_sys() -> anyhow::Result<()> {
    let home = match home::home_dir() {
        Some(path) => path,
        None => panic!("Cannot get home directory"),
    };
    let config_dir = format!("{}/.nioca", home.display());
    println!("Config dir: {}", config_dir);
    fs::create_dir_all(&config_dir).await?;

    let path = "/usr/local/bin";
    println!("Installing nioca-client into {}", path);

    let current_exe = std::env::current_exe().expect("Cannot get nioca-clients own name");
    let target_exe = format!("{}/{}", path, FILE_NAME_EXE);
    let _ = fs::remove_file(&target_exe).await;
    if fs::copy(&current_exe, &target_exe).await.is_err() {
        // if we get an error, we may not be root -> install into $HOME instead
        let target_exe_home = format!("{}/{}", config_dir, FILE_NAME_EXE);
        eprintln!(
            "Unable to install nioca-client globally (not root?), installing it into {} instead.\n\
            You may need to add this directory to your $PATH or copy it manually:\n\n\
            sudo cp {} {}",
            target_exe_home,
            current_exe.to_str().unwrap(),
            target_exe,
        );
        fs::copy(&current_exe, &target_exe_home).await?;
    }

    let empty_env = r#"NIOCA_URL=https://ca.local.dev:8443

#NIOCA_SSH_CLIENT_ID=
#NIOCA_SSH_API_KEY=

#NIOCA_X509_CLIENT_ID=
#NIOCA_X509_API_KEY=
    "#;
    let path_env = format!("{}/{}", config_dir, FILE_NAME_CONFIG);

    // only write the env file, if it does not already exist
    if File::open(&path_env).await.is_err() {
        fs::write(&path_env, empty_env).await?;
        #[cfg(target_family = "unix")]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path_env, Permissions::from_mode(0o600)).await?;
        }
    }

    println!(
        r#"
The nioca-client has been installed successfully.
You should be able to just execute

{} -h

and see the help output.
Before you can use it, you need to adjust the config file and paste the correct values:

{}

When the config is set up, you need to fetch Nioca's root certificate to make sure that all future
connections are valid and secure. You need the Nioca's Root Certificates fingerprint. You either get
it from the logs output of the Nioca container or from the UI.

{} fetch-root -f NiocasRootCertFingerprint

After the Root certificate is installed, you can optionally install it into the systems trust store
too, which is different on every OS. The nioca-client though works without doing this. The root
certificate can be found in

{}/root.pem

and inspected with the default OpenSSL tools, for instance:
openssl x509 -in {}/root.pem -text -noout

If you want to set up an SSH host, you should add a cronjob as root to regularly update the
certificate before it expires. How often depends on your config in Nioca of course.
    "#,
        FILE_NAME_EXE, path_env, FILE_NAME_EXE, path_env, path_env
    );

    Ok(())
}

#[cfg(not(target_family = "unix"))]
async fn install_on_sys() -> anyhow::Result<()> {
    let home = match home::home_dir() {
        Some(path) => path,
        None => panic!("Cannot get home directory"),
    };
    let config_dir = format!("{}\\.nioca", home.display());
    println!("Config dir: {}", config_dir);
    fs::create_dir_all(&config_dir).await?;

    let target_exe = format!("{}{}{}", config_dir, SEPARATOR, FILE_NAME_EXE);
    println!("Installing nioca-client into {}", target_exe);

    let current_exe = std::env::current_exe().expect("Cannot get nioca-clients own name");
    let _ = fs::remove_file(&target_exe).await;
    if fs::copy(&current_exe, &target_exe).await.is_err() {
        eprintln!(
            "Unable to install nioca-client into {}\n\
            You may need to add this directory to your $PATH",
            target_exe,
        );
    }

    let empty_env = r#"NIOCA_URL=https://ca.local.dev:8443

#NIOCA_SSH_CLIENT_ID=
#NIOCA_SSH_API_KEY=

#NIOCA_X509_CLIENT_ID=
#NIOCA_X509_API_KEY=
    "#;
    let path_env = format!("{}{}{}", config_dir, SEPARATOR, FILE_NAME_CONFIG);

    // only write the env file, if it does not already exist
    if File::open(&path_env).await.is_err() {
        fs::write(&path_env, empty_env).await?;
    }

    println!(
        r#"
The nioca-client has been installed successfully.

You need to update your $PATH variable and add: {}
to be able to execute 'nioca-client.exe' directly. Afterwards, you should be able to just execute

{} -h

and see the help output. Otherwise, you can use the full path too

{}{}{} -h

Before you can use it, you need to adjust the config file and paste the correct values:

start {}

When the config is set up, you need to fetch Nioca's root certificate to make sure that all future
connections are valid and secure. You need the Nioca's Root Certificates fingerprint. You either get
it from the logs output of the Nioca container or from the UI.

{} fetch-root -f NiocasRootCertFingerprint

After the Root certificate is installed, you can optionally install it into the systems trust store
too, which is different on every OS. The nioca-client though works without doing this. The root
certificate can be found in

{}{}root.pem
"#,
        config_dir,
        FILE_NAME_EXE,
        config_dir,
        SEPARATOR,
        FILE_NAME_EXE,
        path_env,
        FILE_NAME_EXE,
        config_dir,
        SEPARATOR,
    );

    Ok(())
}

#[cfg(target_family = "unix")]
async fn install_systemd_service() -> anyhow::Result<()> {
    let (path, file_name, contents) = systemd_service_file("root");

    // make sure the system is using systemd
    if fs::try_exists(&path).await.is_err() {
        return Err(anyhow::Error::msg(
            "Only systemd is supported at the moment",
        ));
    }

    // create the service file
    let svc_path = format!("{}/{}", path, file_name);
    fs::write(&svc_path, contents.as_bytes()).await?;

    systemd_reload_services().await?;
    systemd_enable_service(&file_name).await?;

    println!(
        r#"
The 'nioca-client' service has been installed successfully.
Check the status with:
systemctl status nioca-client
"#
    );

    Ok(())
}

#[cfg(target_family = "unix")]
async fn uninstall_from_sys() -> anyhow::Result<()> {
    let home = match home::home_dir() {
        Some(path) => path,
        None => panic!("Cannot get home directory"),
    };
    let config_dir = format!("{}/.nioca", home.display());
    println!("Removing config dir: {}", config_dir);
    if let Err(err) = fs::remove_dir_all(&config_dir).await {
        if err.kind() == ErrorKind::PermissionDenied {
            eprintln!("Permission Denied for removing {}", config_dir);
            return Ok(());
        } else if err.kind() != ErrorKind::NotFound {
            eprintln!("Error removing config: {:?}", err);
            return Ok(());
        }
    }

    let path = "/usr/local/bin";
    println!("Uninstalling nioca-client from {}", path);

    let target_exe = format!("{}/nioca-client", path);
    if let Err(err) = fs::remove_file(&target_exe).await {
        if err.kind() == ErrorKind::PermissionDenied {
            eprintln!(
                "Permission Denied for removing {} - Must be root",
                config_dir
            );
        } else if err.kind() != ErrorKind::NotFound {
            eprintln!("Error removing nioca-client: {:?}", err);
        }
    } else {
        // if we get here, we can be sure we are root -> uninstall systemd services too
        let mut dir = fs::read_dir("/etc/systemd/system/").await?;
        let mut systemd_needs_reload = false;
        while let Some(entry) = dir.next_entry().await? {
            if entry.metadata().await?.is_dir() {
                continue;
            }
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_str().unwrap_or_default();
            if file_name.starts_with("nioca-client") && file_name.ends_with(".service") {
                systemd_needs_reload = true;

                // disable the systemd service and delete the file
                systemd_disable_service(file_name).await?;
                let path = format!("/etc/systemd/system/{}", file_name);
                fs::remove_file(&path).await?;
            }
        }

        if systemd_needs_reload {
            systemd_reload_services().await?;
        }

        println!("The nioca-client has been uninstalled successfully.");
    };

    Ok(())
}

#[cfg(not(target_family = "unix"))]
async fn uninstall_from_sys() -> anyhow::Result<()> {
    let home = match home::home_dir() {
        Some(path) => path,
        None => panic!("Cannot get home directory"),
    };
    let config_dir = format!("{}\\.nioca", home.display());
    println!("Removing config dir: {}", config_dir);
    if let Err(err) = fs::remove_dir_all(&config_dir).await {
        if err.kind() == ErrorKind::PermissionDenied {
            eprintln!("Permission Denied for removing {}", config_dir);
            return Ok(());
        } else if err.kind() != ErrorKind::NotFound {
            eprintln!("Error removing config: {:?}", err);
            return Ok(());
        }
    }

    Ok(())
}

async fn fetch_root_ca(args: &CmdFetchRoot) -> anyhow::Result<()> {
    let config = get_config(&args.config).await;

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .https_only(true)
        // This is mandatory to not have a chicken and egg problem -> integrity is validated with fingerprint
        .danger_accept_invalid_certs(true)
        .user_agent(format!("Nioca Client {}", VERSION))
        .build()
        .expect("Building reqwest client for fetch_root_ca");

    let url = format!("{}/root.pem", config.url);
    println!("Fetching root certificate from {}", url);
    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(root_pem) => {
                    // let destination = destination(&args.destination);
                    // fs::create_dir_all(&destination).await?;

                    println!("Fetched root certificate:\n\n{}\n", root_pem);

                    let hash = fingerprint(root_pem.as_bytes());
                    if hash != args.fingerprint {
                        eprintln!(
                            "Given fingerprint does not match.\nFetched certificates fingerprint: {}",
                            hash
                        );
                    } else {
                        println!("Certificate fingerprint verified");
                        if let Some(dest) = &args.destination {
                            println!("Saving root certificate to {}root.pem", dest);

                            let destination = destination(dest);
                            fs::create_dir_all(&destination).await?;
                            fs::write(format!("{}root.pem", destination), root_pem.as_bytes())
                                .await
                                .expect("Cannot write Root CA PEM to given destination");
                        } else {
                            match home::home_dir() {
                                Some(path) => {
                                    let p = format!("{}{}.nioca", path.display(), SEPARATOR);
                                    fs::create_dir_all(&p).await?;
                                    let target = format!("{}{}root.pem", p, SEPARATOR);
                                    println!("Saving root certificate to {}", target);
                                    fs::write(&target, root_pem.as_bytes())
                                        .await
                                        .expect("Writing Root CA to $HOME");

                                    #[cfg(target_family = "unix")]
                                    println!(
                                        "\nYou can inspect the certificate with default openssl tools:\n\
                                        openssl x509 -in {} -text -noout",
                                        target
                                    )
                                }
                                None => eprintln!("Cannot get home directory"),
                            }
                        }
                    }
                }
                Err(err) => {
                    let msg = format!(
                        "{} - Error deserializing response into CertX509Response: {}",
                        status, err
                    );
                    eprintln!("{}", msg);
                }
            }
        }
        Err(err) => eprintln!("Error fetching TLS certificate from Nioca: {}", err),
    };

    Ok(())
}

async fn daemonize(args: &CmdDaemonize) -> anyhow::Result<()> {
    let cmd_ssh = CmdSsh {
        config: args.config.clone(),
        destination: args.destination.clone(),
        install: true,
    };
    tokio::spawn(async move {
        if let Err(err) = fetch_ssh(cmd_ssh, true).await {
            println!("{}", err);
        }
    });

    // sleep 3 seconds just to not mess up the logging for the very first 2 cert fetches
    time::sleep(Duration::from_secs(3)).await;

    let cmd_x509 = CmdX509 {
        config: args.config.clone(),
        destination: args.destination.clone(),
    };
    tokio::spawn(async move {
        if let Err(err) = fetch_x509(cmd_x509, true).await {
            println!("{}", err);
        }
    });

    let (tx, rx) = flume::unbounded();
    ctrlc::set_handler(move || tx.send(()).unwrap()).expect("Error setting Ctrl-C handler");
    rx.recv_async().await.expect("awaiting exit signal handler");

    Ok(())
}

async fn fetch_ssh(args: CmdSsh, daemonize: bool) -> anyhow::Result<()> {
    let config = get_config(&args.config).await;

    let api_key = if let Some(key) = &config.api_key_ssh {
        key
    } else {
        return Err(anyhow::Error::msg("NIOCA_SSH_API_KEY is not set"));
    };
    let url = if let Some(url) = &config.url_ssh {
        url
    } else {
        return Err(anyhow::Error::msg("NIOCA_SSH_CLIENT_ID is not set"));
    };

    let client = req_client(config.root_cert.clone());
    let bearer = auth_token(api_key);

    let mut next_fetch = *ERR_TIMEOUT;
    loop {
        println!("\nFetching SSH certificate from {}", url);

        match fetch_cert_ssh(&client, url, &bearer).await {
            Ok(resp) => {
                let destination = destination(&args.destination);
                match save_files_ssh(&destination, &resp).await {
                    Ok(valid_until) => {
                        let now = system_to_naive_datetime(SystemTime::now());
                        let diff = valid_until.sub(now).num_seconds() as u64;
                        next_fetch = diff * 90 / 100;
                    }
                    Err(err) => eprintln!("Error fetching SSH certificate: {}", err),
                }

                if daemonize || args.install {
                    install_host_ssh(&resp).await?;
                    install_known_host(&resp).await?;
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }

        match daemonize {
            true => {
                println!("Fetching next SSH certificate in {} seconds", next_fetch);
                time::sleep(Duration::from_secs(next_fetch)).await;
            }
            false => {
                return Ok(());
            }
        }
    }
}

async fn fetch_x509(args: CmdX509, daemonize: bool) -> anyhow::Result<()> {
    let config = get_config(&args.config).await;

    let api_key = if let Some(key) = &config.api_key_x509 {
        key
    } else {
        return Err(anyhow::Error::msg("NIOCA_X509_API_KEY is not set"));
    };
    let url = if let Some(url) = &config.url_x509 {
        url
    } else {
        return Err(anyhow::Error::msg("NIOCA_X509_CLIENT_ID is not set"));
    };

    let client = req_client(config.root_cert.clone());
    let bearer = auth_token(api_key);

    let mut next_fetch = *ERR_TIMEOUT;
    loop {
        println!("\nFetching X509 certificate from {}", url);

        match fetch_cert_x509(&client, url, &bearer).await {
            Ok((certs, not_after_sec)) => {
                let destination = destination(&args.destination);
                match save_files_x509(&destination, &certs).await {
                    Ok(_) => {
                        next_fetch = not_after_sec;
                    }
                    Err(err) => eprintln!("Error fetching X509 certificate: {}", err),
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }

        match daemonize {
            true => {
                println!("Fetching next X509 certificate in {} seconds", next_fetch);
                time::sleep(Duration::from_secs(next_fetch)).await;
            }
            false => {
                return Ok(());
            }
        }
    }
}

pub fn fingerprint(value: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, value);
    let fingerprint = hex::encode(digest.as_ref());
    let fingerprint_full = format!("sha256:{}", fingerprint);
    fingerprint_full
}

async fn save_files_ssh(
    out_dir: &str,
    certs: &SshCertificateResponse,
) -> anyhow::Result<NaiveDateTime> {
    let out_dir = format!("{}ssh{}", out_dir, SEPARATOR);
    fs::create_dir_all(&out_dir).await?;

    println!("Saving SSH certificate to {}", out_dir);
    let path_key = format!("{}id_nioca", out_dir);
    fs::write(&path_key, certs.host_key_pair.id.as_bytes())
        .await
        .expect("Writing Certificate");
    fs::write(
        format!("{}id_nioca.pub", out_dir),
        certs.host_key_pair.id_pub.as_bytes(),
    )
    .await
    .expect("Writing Certificate");
    fs::write(
        format!("{}id_nioca_ca.pub", out_dir),
        certs.user_ca_pub.as_bytes(),
    )
    .await
    .expect("Writing Certificate");

    // the key must only be readable by the current user
    // #[cfg(target_family = "unix")]
    // {
    //     use std::fs::Permissions;
    //     use std::os::unix::fs::PermissionsExt;
    //     fs::set_permissions(&path_key, Permissions::from_mode(0o600)).await?;
    // }
    // the key must only be readable by the current user
    set_perm_user_only(&path_key).await?;

    println!("SSH Certificate saved successfully.");

    let cert = ssh_key::Certificate::from_openssh(&certs.host_key_pair.id_pub)
        .expect("Cannot parse SSH Certificate");
    let valid_until = system_to_naive_datetime(cert.valid_before_time());

    if certs.host_key_pair.typ == Some(SshCertType::User) {
        println!(
            r#"
    Certificate Type: {:?}
    Algorithm: {:?}
    Allowed usernames: {:?}
    Extensions: {:?}
    Certificate valid until: {:?}
        "#,
            cert.cert_type(),
            cert.algorithm(),
            cert.valid_principals(),
            cert.extensions(),
            valid_until,
        );

        println!(
            "Connect to your target with:\n\nssh -i {} USER@IP\n",
            path_key
        );
    } else {
        println!(
            r#"
    Certificate Type: {:?}
    Algorithm: {:?}
    Allowed hostnames: {:?}
    Certificate valid until: {:?}
        "#,
            cert.cert_type(),
            cert.algorithm(),
            cert.valid_principals(),
            valid_until,
        );
    }

    Ok(valid_until)
}

async fn save_files_x509(out_dir: &str, certs: &CertX509Response) -> anyhow::Result<()> {
    let out_dir = format!("{}x509{}", out_dir, SEPARATOR);
    let out_na = format!("{}{}", out_dir, certs.not_after);
    fs::create_dir_all(&out_na).await?;

    println!("Saving certificates to {}", out_dir);
    fs::write(format!("{}cert.pem", out_dir), certs.cert.as_bytes())
        .await
        .expect("Writing Certificate");
    fs::write(format!("{}chain.pem", out_dir), certs.cert_chain.as_bytes())
        .await
        .expect("Writing Certificate");
    let path_key = format!("{}key.pem", out_dir);
    fs::write(&path_key, certs.key.as_bytes())
        .await
        .expect("Writing Certificate");

    println!("Saving certificates to {}", out_na);
    fs::write(
        format!("{}{}cert.pem", out_na, SEPARATOR),
        certs.cert.as_bytes(),
    )
    .await
    .expect("Writing Certificate");
    fs::write(
        format!("{}{}chain.pem", out_na, SEPARATOR),
        certs.cert_chain.as_bytes(),
    )
    .await
    .expect("Writing Certificate");
    let path_key_na = format!("{}{}key.pem", out_na, SEPARATOR);
    fs::write(&path_key_na, certs.key.as_bytes())
        .await
        .expect("Writing Certificate");

    // the key must only be readable by the current user
    set_perm_user_only(&path_key).await?;

    Ok(())
}

#[cfg(target_family = "unix")]
async fn set_perm_user_only(path: &str) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use tokio::fs::File;

    let file = File::open(path).await?;
    let mut perms = file.metadata().await?.permissions();
    perms.set_mode(0o600);
    file.set_permissions(perms).await?;

    let file = File::open(path).await?;
    let mut perms = file.metadata().await?.permissions();
    perms.set_mode(0o600);
    file.set_permissions(perms).await?;

    Ok(())
}

#[cfg(not(target_family = "unix"))]
async fn set_perm_user_only(path: &str) -> anyhow::Result<()> {
    // lets get our username first
    let out = Command::new("powershell.exe")
        .arg("-c")
        .arg("$env:UserName")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;
    let username = String::from_utf8_lossy(&out.stdout).trim().to_string();

    println!("Setting access only to {} for {}", username, path);

    // remove inheritance
    if !Command::new("icacls.exe")
        .arg(path)
        .arg("/inheritance:r")
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?
        .success()
    {
        eprintln!("Error removing file inheritance for {}", path);
    }

    // set ownership
    if !Command::new("icacls.exe")
        .arg(path)
        .arg("/grant")
        .arg(format!("{}:F", username))
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?
        .success()
    {
        eprintln!("Error granting user full access to {}", path);
    }
    if !Command::new("takeown.exe")
        .arg("/F")
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?
        .success()
    {
        eprintln!("Error taking ownership for {}", path);
    }
    if !Command::new("icacls.exe")
        .arg(path)
        .arg("/grant:r")
        .arg(format!("{}:F", username))
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?
        .success()
    {
        eprintln!("Error granting 'read' to {}", path);
    }

    Ok(())
}

async fn install_host_ssh(certs: &SshCertificateResponse) -> anyhow::Result<()> {
    if certs.host_key_pair.typ == Some(SshCertType::User) {
        // eprintln!("Received SSH Cert Type is 'User' - not installing anything");
        return Ok(());
    }

    #[cfg(not(target_family = "unix"))]
    println!("Received an SSH host certificate which cannot be installed automatically on Windows");

    #[cfg(target_family = "unix")]
    {
        let path_ca_pub = "/etc/ssh/id_nioca_ca.pub";
        // fs::write(&path_ca_pub, certs.user_ca_pub.as_bytes()).await?;
        if let Err(err) = fs::write(&path_ca_pub, certs.user_ca_pub.as_bytes()).await {
            if err.kind() == ErrorKind::PermissionDenied {
                let msg = "You can only install SSH Host certificates as root - exiting early";
                eprintln!("{}", msg);
                return Err(anyhow::Error::msg(msg));
            }
        }

        let path_id = "/etc/ssh/id_nioca_host";
        fs::write(&path_id, certs.host_key_pair.id.as_bytes()).await?;
        // the key must only be readable by the current user
        set_perm_user_only(&path_id).await?;

        // TODO `sshd` prints out a weird error when you log in with certificates, even though everything works fine:
        // error: Public key for /etc/ssh/id_nioca_host does not match private key
        // -> try to find the reason for that, since it does not make any sense
        // -> we do not have (and need) any public key, but the certificate instead
        // -> needs another sshd config adjustment here?
        let path_id_pub = "/etc/ssh/id_nioca_host.pub";
        fs::write(&path_id_pub, certs.host_key_pair.id_pub.as_bytes()).await?;

        let config = format!(
            r#"## Nioca SSH certificate configuration
    
    # The User CA to trust - this must be the public key of the 'group' this client belongs to.
    TrustedUserCAKeys {}
    
    # This host's SSH certificate key pair
    HostKey {}
    HostCertificate {}
    
        "#,
            path_ca_pub, path_id, path_id_pub,
        );

        // save new config
        let ssh_config = "/etc/ssh/sshd_config.d/10-nioca.conf";
        println!("Writing sshd certificate config to {}", ssh_config);
        fs::write(&ssh_config, config.as_bytes()).await?;

        // make sure that the *.d folder is included in the sshd_config
        let sshd_config_include = "Include /etc/ssh/sshd_config.d/*.conf";
        let path_sshd_config = "/etc/ssh/sshd_config";
        let mut sshd_config = fs::read_to_string(&path_sshd_config).await?;
        let mut d_is_included = false;
        for line in sshd_config.lines() {
            if line == sshd_config_include {
                d_is_included = true;
                break;
            }
        }
        if !d_is_included {
            writeln!(sshd_config, "{}", sshd_config_include)?;
            fs::write(&path_sshd_config, sshd_config).await?;
        }

        println!("Restarting sshd to read the new config");
        let mut child = Command::new("/usr/bin/systemctl")
            .arg("restart")
            .arg("sshd")
            .stdout(Stdio::piped())
            .spawn()?;

        let status = child.wait().await?;
        if status.success() {
            println!("ssh was restarted successfully");
        } else {
            eprintln!("Error restarting sshd, please check the logs manually");
        }
    }

    Ok(())
}

async fn install_known_host(certs: &SshCertificateResponse) -> anyhow::Result<()> {
    if certs.host_key_pair.typ == Some(SshCertType::Host) {
        eprintln!("Received SSH Cert Type is 'Host' - not adding it to known hosts");
        return Ok(());
    }

    println!("Installing CA certificate into known hosts");

    // TODO remove the known hosts in $HOME? -> would not allow other users to log in?
    let home = match home::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Cannot get home directory - exiting early");
            return Ok(());
        }
    };

    let path = format!("{}{}.ssh", home.display(), SEPARATOR);
    let path_known_hosts = format!("{}{}known_hosts", path, SEPARATOR);
    if File::open(&path_known_hosts).await.is_err() {
        println!(
            "known_hosts file not found in {} - creating it now",
            path_known_hosts
        );
        fs::create_dir_all(&path).await?;
        fs::write(&path_known_hosts, b"").await?;
        #[cfg(target_family = "unix")]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path_known_hosts, Permissions::from_mode(0o600)).await?;
        }
    }

    let entry = format!("@cert-authority * {} nioca-ssh-ca", certs.user_ca_pub);

    let mut known_hosts = fs::read_to_string(&path_known_hosts).await?;
    for line in known_hosts.lines() {
        if line == entry {
            // println!("CA certificate already exists in known_hosts");
            return Ok(());
        }
    }

    // If we get until here, the cert does not exist in the known_hosts - append it to the end
    writeln!(known_hosts, "{}", entry)?;
    fs::write(&path_known_hosts, known_hosts).await?;

    println!("CA certificate has been added to {}", path_known_hosts);

    Ok(())
}

#[inline(always)]
fn destination(destination: &str) -> String {
    if destination.ends_with(SEPARATOR) {
        destination.to_string()
    } else {
        format!("{}{}", destination, SEPARATOR)
    }
}

pub fn system_to_naive_datetime(t: SystemTime) -> chrono::NaiveDateTime {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };

    // calculating the diff makes it possible to convert to naive local without the need for
    // hardcoded timezones
    let utc_now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("The local system time is broken")
        .as_secs() as i64;
    let local_now_secs = chrono::Local::now().naive_local().timestamp();
    let sec_diff = local_now_secs - utc_now_secs;

    chrono::NaiveDateTime::from_timestamp_opt(sec + sec_diff, nsec).unwrap()
}

/// Returns (BasePath, ServiceName, FileContents) for the systemd *.service file
#[cfg(target_family = "unix")]
fn systemd_service_file(user: &str) -> (&str, String, String) {
    let path_base = "/etc/systemd/system";
    let file_name = if user == "root" {
        "nioca-client.service".to_string()
    } else {
        format!("nioca-client-{}.service", user)
    };

    let contents = format!(
        r#"[Unit]
Description=Nioca Client Daemon
Requires=network-online.target
After=network-online.target
StartLimitIntervalSec=0

[Service]
Restart=always
RestartSec=30
User={}
ExecStart=/usr/local/bin/nioca-client daemonize

[Install]
WantedBy=multi-user.target

"#,
        user,
    );

    (path_base, file_name, contents)
}

#[cfg(target_family = "unix")]
async fn systemd_reload_services() -> anyhow::Result<()> {
    println!("Reloading systemd service files");
    let status = Command::new("/usr/bin/systemctl")
        .arg("daemon-reload")
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?;
    if status.success() {
        println!("systemctl daemon-reload successful");
    } else {
        eprintln!("Error for systemctl daemon-reload");
    }

    Ok(())
}

#[cfg(target_family = "unix")]
async fn systemd_enable_service(name: &str) -> anyhow::Result<()> {
    println!("Enabling Service {}", name);
    let status = Command::new("/usr/bin/systemctl")
        .arg("enable")
        .arg(name)
        .arg("--now")
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?;
    if status.success() {
        println!("Service {} enable successfully", name);
    } else {
        eprintln!("Error enabling service {}", name);
    }

    Ok(())
}

#[cfg(target_family = "unix")]
async fn systemd_disable_service(name: &str) -> anyhow::Result<()> {
    println!("Disabling Service {}", name);
    let status = Command::new("/usr/bin/systemctl")
        .arg("disable")
        .arg(name)
        .arg("--now")
        .stdout(Stdio::piped())
        .spawn()?
        .wait()
        .await?;
    if status.success() {
        println!("Service {} disabled successfully", name);
    } else {
        eprintln!("Error disabling service {}", name);
    }

    Ok(())
}
