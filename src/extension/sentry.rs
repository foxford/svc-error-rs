use std::borrow::Cow;
use std::sync::Arc;
use std::thread;

use serde_derive::Deserialize;

use once_cell::sync::OnceCell;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Sender = crossbeam_channel::Sender<Cmd>;

static SENTRY_TX: OnceCell<Sender> = OnceCell::new();

/// Sentry configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// The URL that Sentry gives when creating a project there.
    pub dsn: String,
    /// staging, production etc.
    pub environment: Option<String>,
    /// Some string to identify the instance of your service.
    pub server_name: Option<String>,
    /// The release to be sent with events.
    pub release: Option<Cow<'static, str>>,
}

enum Cmd {
    Terminate,
    AnyhowError(Arc<anyhow::Error>),
}

/// Spawns a thread that sends errors to Sentry.
/// The thread will be spawned only on first invocation, any other calls to init will return None.
/// Returned JoinHandle should be joined on shutdown so remaining events wont be lost.
pub fn init(config: &Config) -> Option<thread::JoinHandle<()>> {
    let config = config.to_owned();
    let (tx, rx) = crossbeam_channel::unbounded::<Cmd>();
    match SENTRY_TX.try_insert(tx) {
        Err(_) => None,
        Ok(_) => {
            std::env::set_var("RUST_BACKTRACE", "1");
            std::env::set_var("RUST_LIB_BACKTRACE", "1");

            let handle = thread::spawn(move || {
                let options = options(&config);
                let _guard = sentry::init((config.dsn, options));

                sentry::configure_scope(|scope| {
                    if let Ok(kns) = std::env::var("KUBE_NAMESPACE") {
                        scope.set_tag("kube_namespace", kns);
                    }
                });

                for message in rx {
                    match message {
                        Cmd::Terminate => return,
                        Cmd::AnyhowError(err) => {
                            sentry::integrations::anyhow::capture_anyhow(&err);
                        }
                    }
                }
            });
            Some(handle)
        }
    }
}

fn options(config: &Config) -> sentry::ClientOptions {
    let mut options: sentry::ClientOptions = sentry::ClientOptions {
        attach_stacktrace: true,
        release: config.release.clone(),
        ..Default::default()
    };

    if let Some(environment) = config.environment.clone() {
        options.environment = Some(Cow::from(environment));
    }

    if let Some(server_name) = config.server_name.clone() {
        options.server_name = Some(Cow::from(server_name));
    }

    options
}

/// Send error to Sentry
pub fn send(err: Arc<anyhow::Error>) -> Result<(), Error> {
    match SENTRY_TX.get() {
        None => Ok(()),
        Some(tx) => tx
            .send(Cmd::AnyhowError(err))
            .map_err(|err| format!("Failed to send error to Sentry: {err}").into()),
    }
}

/// Terminate Sentry thread
pub fn terminate() -> Result<(), Error> {
    match SENTRY_TX.get() {
        None => Ok(()),
        Some(tx) => tx
            .send(Cmd::Terminate)
            .map_err(|err| format!("Failed to send shutdown signal to Sentry: {err}").into()),
    }
}
