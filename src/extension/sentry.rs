use std::collections::BTreeMap;

use sentry::protocol::{value::Value, Event, Level};

use crate::Error;

impl Into<Event<'_>> for Error {
    fn into(self) -> Event<'_> {
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
