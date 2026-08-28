use time::OffsetDateTime;
use time::macros::format_description;

const DATE: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]");
const DISPLAY: &[time::format_description::BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

pub fn rfc3339(value: OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}

pub fn display(value: OffsetDateTime) -> String {
    value.format(DISPLAY).unwrap()
}

pub fn date(value: OffsetDateTime) -> String {
    value.date().to_string()
}

pub mod option {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer};
    use time::format_description::well_known::Rfc3339;
    use time::{Date, OffsetDateTime, Time};

    use super::DATE;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<OffsetDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(value) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };
        if let Ok(value) = OffsetDateTime::parse(&value, &Rfc3339) {
            return Ok(Some(value));
        }

        Date::parse(&value, DATE)
            .map(|date| date.with_time(Time::MIDNIGHT).assume_utc())
            .map(Some)
            .map_err(D::Error::custom)
    }
}
