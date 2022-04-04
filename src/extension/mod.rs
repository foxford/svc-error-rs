#[cfg(feature = "diesel")]
mod diesel;

#[cfg(feature = "sqlx")]
mod sqlx;

#[cfg(feature = "r2d2")]
mod r2d2;

#[cfg(feature = "svc-agent")]
mod svc_agent;

#[cfg(feature = "svc-authn")]
mod svc_authn;

#[cfg(feature = "svc-authz")]
mod svc_authz;

/// Integration with Sentry exception tracker.
///
/// Setup:
/// 1. Enable `sentry-extension` feature in `Cargo.toml`:
///    ```toml
///    svc_error = { version = "0.1", features = ["sentry-extension"] }
///    ```
/// 2. Add `sentry` section to your app's config of type `svc_error::extension::sentry::Config`:
///    ```rust
///     use serde::Deserialize;
///
///    #[derive(Deserialize)]
///    struct MyAppConfig {
///        sentry: svc_error::extension::sentry::Config,
///    }
///    ```
/// 3. When initializing your app call:
///    ```rust,ignore
///    svc_error::extension::sentry::init(&config.sentry);
///    ```
///    It spawns a thread for asynchonous error sending.
/// 4. Send an `svc_error::Error` or any other type that implements `svc_error::ProblemDetails`
///    to whenever you like it to be sent to Sentry:
///    ```rust,ignore
///    let err = svc_error::Error::builder().detail("Something bad").build();
///    svc_error::extension::sentry::send(err)?;
///    ```
#[cfg(feature = "sentry")]
pub mod sentry;
