use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use atom::AtomSetOnce;
use lazy_static::lazy_static;
use sentry::protocol::{value::Value, Event, Level};
use serde_derive::Deserialize;

use crate::ProblemDetailsReadOnly;

type Error = Box<dyn std::error::Error + Send + Sync>;
type BoxedProblemDetails = Box<dyn ProblemDetailsReadOnly + Send>;
type Sender = mpsc::Sender<BoxedProblemDetails>;

lazy_static! {
    static ref SENTRY_TX: AtomSetOnce<Arc<RwLock<Sender>>> = AtomSetOnce::empty();
}

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

/// Spawns a thread that sends errors to Sentry.
pub fn init(config: &Config) {
    let config = config.to_owned();
    let (tx, rx) = mpsc::channel::<BoxedProblemDetails>();
    SENTRY_TX.set_if_none(Arc::new(RwLock::new(tx)));

    thread::spawn(move || {
        let _guard = sentry::init(config.dsn);
        sentry::integrations::panic::register_panic_handler();

        for err in rx {
            let mut event: Event = err.into();

            if let Some(environment) = config.environment.clone() {
                event.environment = Some(Cow::from(environment));
            }

            if let Some(server_name) = config.server_name.clone() {
                event.server_name = Some(Cow::from(server_name));
            }

            sentry::capture_event(event);
        }
    });
}

/// Send error to Sentry
pub fn send<E>(err: E) -> Result<(), Error>
where
    E: ProblemDetailsReadOnly + Send + 'static,
{
    match SENTRY_TX.get() {
        None => Ok(()),
        Some(tx_lock) => tx_lock
            .read()
            .map_err(|err| format!("Failed to acquire Sentry tx lock: {}", err).into())
            .and_then(|tx| {
                tx.send(Box::new(err))
                    .map_err(|err| format!("Failed to send error to Sentry: {}", err).into())
            }),
    }
}

impl Into<Event<'static>> for BoxedProblemDetails {
    fn into(self) -> Event<'static> {
        let mut extra = BTreeMap::new();

        extra.insert(String::from("type"), Value::from(self.kind()));
        extra.insert(String::from("title"), Value::from(self.title()));

        extra.insert(
            String::from("status_code"),
            Value::from(self.status_code().as_str()),
        );

        Event {
            message: self.detail().map(|s| s.to_owned()),
            level: Level::Error,
            extra,
            ..Default::default()
        }
    }
}
