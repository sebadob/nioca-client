use crate::{
    auth_token, fetch_cert_ssh, fetch_cert_x509, req_client, CertX509Response, NiocaConfig,
    SshCertType, SshCertificateResponse, ERR_TIMEOUT, VERSION,
};
use chrono::NaiveDateTime;
use clap::{arg, Parser};
use std::io::ErrorKind;
use std::ops::Sub;
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tokio::{fs, time};

/// This client fetches TLS certificates from Nioca
#[derive(Debug, PartialEq, Parser)]
#[command(author, version, about)]
pub(crate) enum CliArgs {
    /// Installs the nioca-client into the system
    Install,
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
    /// Path to the config file (default $HOME/.nioca/config)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "$HOME/.nioca/")]
    pub destination: String,

    /// Fetches the root CA's certificate and verifies it with the given fingerprint
    #[arg(short, long)]
    pub fingerprint: String,
}

/// Run the client continuously as a daemon to always renew expiring certificates
#[derive(Debug, PartialEq, Parser)]
#[command(author, version)]
pub(crate) struct CmdDaemonize {
    /// Path to the config file (default $HOME/.nioca/config)
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
    /// Path to the config file (default $HOME/.nioca/config)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "./certs")]
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
    /// Path to the config (default $HOME/.nioca/config)
    #[arg(short, long)]
    pub config: Option<String>,

    /// Output path for the fetched certificates
    #[arg(short, long, default_value = "./certs")]
    pub destination: String,
}

pub async fn execute() -> anyhow::Result<()> {
    let args: CliArgs = CliArgs::parse();
    match args {
        CliArgs::Install => {
            install_on_sys().await?;
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
        let config_dir = format!("{}/.nioca/config", home.display());
        match dotenvy::from_filename(config_dir) {
            Ok(_env) => {}
            Err(_) => {
                panic!("Cannot read nioca-client config - Please install first");
            }
        }
    }
    NiocaConfig::from_env().await
}

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
    let target_exe = format!("{}/nioca-client", path);
    let _ = fs::remove_file(&target_exe).await;
    if fs::copy(&current_exe, &target_exe).await.is_err() {
        // if we get an error, we may not be root -> install into $HOME instead
        let target_exe_home = format!("{}/nioca-client", config_dir);
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
    let path_env = format!("{}/config", config_dir);

    // only write the env file, if it does not already exist
    if fs::File::open(&path_env).await.is_err() {
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

nioca-client -h

and see the help output.
Before you can use it, you need to adjust the config file and paste the correct values:

vim {}

When the config is set up, you need to fetch Nioca's root certificate to make sure that all future
connections are valid and secure. You need the Nioca's Root Certificates fingerprint. You either get
it from the logs output of the Nioca container or from the UI.

nioca-client fetch-root -f NiocasRootCertFingerprint

After the Root certificate is installed, you can optionally install it into the systems trust store
too, which is different on every OS. The nioca-client though works without doing this. The root
certificate can be found in

cat {}/root.pem

and inspected with the default OpenSSL tools, for instance:
openssl x509 -in {}/root.pem -text -noout

If you want to set up an SSH host, you should add a cronjob as root to regularly update the
certificate before it expires. How often depends on your config in Nioca of course.
    "#,
        path_env, path_env, path_env
    );

    Ok(())
}

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
        println!("The nioca-client has been uninstalled successfully.");
    };

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
                    let destination = destination(&args.destination);
                    fs::create_dir_all(&destination).await?;

                    println!("Fetched root certificate:\n\n{}\n", root_pem);

                    let hash = fingerprint(root_pem.as_bytes());
                    if hash != args.fingerprint {
                        eprintln!(
                            "Given fingerprint does not match.\nFetched certificates fingerprint: {}",
                            hash
                        );
                    } else {
                        println!("Certificate fingerprint verified");
                        println!("Saving root certificate to {}root.pem", destination);
                        fs::write(format!("{}root.pem", destination), root_pem.as_bytes())
                            .await
                            .expect("Writing Root CA");

                        match home::home_dir() {
                            Some(path) => {
                                let p = format!("{}/.nioca", path.display());
                                fs::create_dir_all(&p).await?;
                                let target = format!("{}/root.pem", p);
                                println!("Saving root certificate to {}", target);
                                fs::write(&target, root_pem.as_bytes())
                                    .await
                                    .expect("Writing Root CA to $HOME");

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
    let out_dir = format!("{}ssh/", out_dir);
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
    #[cfg(target_family = "unix")]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path_key, Permissions::from_mode(0o600)).await?;
    }

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
    let out_dir = format!("{}x509/", out_dir);
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
    fs::write(format!("{}/cert.pem", out_na), certs.cert.as_bytes())
        .await
        .expect("Writing Certificate");
    fs::write(format!("{}/chain.pem", out_na), certs.cert_chain.as_bytes())
        .await
        .expect("Writing Certificate");
    let path_key_na = format!("{}/key.pem", out_na);
    fs::write(&path_key_na, certs.key.as_bytes())
        .await
        .expect("Writing Certificate");

    // the key must only be readable by the current user
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        use tokio::fs::File;

        let file = File::open(path_key).await?;
        let mut perms = file.metadata().await?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms).await?;

        let file = File::open(path_key_na).await?;
        let mut perms = file.metadata().await?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms).await?;
    }

    Ok(())
}

async fn install_host_ssh(certs: &SshCertificateResponse) -> anyhow::Result<()> {
    if certs.host_key_pair.typ == Some(SshCertType::User) {
        eprintln!("Received SSH Cert Type is 'User' - not installing anything");
        return Ok(());
    }

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
    #[cfg(target_family = "unix")]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path_id, Permissions::from_mode(0o600)).await?;
    }

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

    let ssh_config = "/etc/ssh/sshd_config.d/10-nioca.conf";
    println!("Writing sshd certificate config to {}", ssh_config);
    fs::write(&ssh_config, config.as_bytes()).await?;

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

    Ok(())
}

async fn install_known_host(certs: &SshCertificateResponse) -> anyhow::Result<()> {
    use std::fmt::Write;

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

    let path = format!("{}/.ssh", home.display());
    let path_known_hosts = format!("{}/known_hosts", path);
    if fs::File::open(&path_known_hosts).await.is_err() {
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

    let known_hosts = fs::read_to_string(&path_known_hosts).await?;
    let mut known_hosts_new = String::with_capacity(known_hosts.len() + 500);
    for line in known_hosts.lines() {
        if line.starts_with("@cert-authority ") && line.ends_with(" nioca-ssh-ca") {
            if line == entry {
                println!("CA certificate already exists in known_hosts");
            } else {
                // in this case we do have another CA saved which we need to clean up
                // -> do nothing
                println!(
                    "Found old CA certificate in {} which will be cleaned up:\n{}",
                    path_known_hosts, line,
                );
            }
        } else {
            writeln!(known_hosts_new, "{}", line)?;
        }
    }

    // If we get until here, the cert does not exist in the known_hosts - append it to the end
    writeln!(known_hosts_new, "{}", entry)?;
    fs::write(&path_known_hosts, known_hosts_new).await?;

    println!("CA certificate has been added to {}", path_known_hosts);

    Ok(())
}

fn destination(destination: &str) -> String {
    if destination.ends_with('/') {
        destination.to_string()
    } else {
        format!("{}/", destination)
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
