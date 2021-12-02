use std::borrow::Cow;
use std::collections::BTreeMap;
use std::thread;

use sentry::protocol::{value::Value, Event, Level};
use serde_derive::Deserialize;

use crate::ProblemDetailsReadOnly;
use once_cell::sync::OnceCell;

type Error = Box<dyn std::error::Error + Send + Sync>;
type BoxedProblemDetails = Box<dyn ProblemDetailsReadOnly + Send>;
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
}

enum Cmd {
    Terminate,
    NewError(BoxedProblemDetails),
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
                        Cmd::NewError(err) => {
                            let event: Event = err.into();
                            sentry::capture_event(event);
                        }
                    }
                }
            });
            Some(handle)
        }
    }
}

fn options(config: &Config) -> sentry::ClientOptions {
    let mut options: sentry::ClientOptions = Default::default();

    if let Some(environment) = config.environment.clone() {
        options.environment = Some(Cow::from(environment));
    }

    if let Some(server_name) = config.server_name.clone() {
        options.server_name = Some(Cow::from(server_name));
    }

    options
}

/// Send error to Sentry
pub fn send<E>(err: E) -> Result<(), Error>
where
    E: ProblemDetailsReadOnly + Send + 'static,
{
    match SENTRY_TX.get() {
        None => Ok(()),
        Some(tx) => tx
            .send(Cmd::NewError(Box::new(err)))
            .map_err(|err| format!("Failed to send error to Sentry: {}", err).into()),
    }
}

/// Terminate Sentry thread
pub fn terminate() -> Result<(), Error> {
    match SENTRY_TX.get() {
        None => Ok(()),
        Some(tx) => tx
            .send(Cmd::Terminate)
            .map_err(|err| format!("Failed to send shutdown signal to Sentry: {}", err).into()),
    }
}

impl Into<Event<'static>> for BoxedProblemDetails {
    fn into(self) -> Event<'static> {
        let mut extra = BTreeMap::new();

        for (extra_key, value) in self.extras() {
            extra.insert(extra_key.to_owned(), Value::from(value.to_owned()));
        }

        extra.insert(String::from("type"), Value::from(self.kind()));
        extra.insert(String::from("title"), Value::from(self.title()));

        extra.insert(
            String::from("status_code"),
            Value::from(self.status_code().as_str()),
        );

        Event {
            message: self.detail().map(|s| s.to_owned()),
            fingerprint: vec![self.kind().to_owned().into()].into(),
            level: Level::Error,
            extra,
            ..Default::default()
        }
    }
}
