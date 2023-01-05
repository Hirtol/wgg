use std::collections::HashMap;

pub fn build_map<'key, 'value, const N: usize>(
    array: [(&'key str, Option<&'value str>); N],
) -> HashMap<&'key str, &'value str> {
    // Use a manual for loop instead of iterators so we can call `with_capacity`
    // and avoid reallocating.
    let mut map = HashMap::with_capacity(N);
    for (key, value) in array {
        if let Some(value) = value {
            map.insert(key, value);
        }
    }
    map
}

pub mod date_format_parser {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

    /// Attempt to deserialize a Jumbo DateTime.
    /// Usually Jumbo produces a RFC3339 compliant timestamp, but sometimes it's lacking a Timezone specifier.
    /// This causes a parsing failure, so we manually add the alternative here.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse()
            .or_else(|_| Utc.datetime_from_str(&s, FORMAT))
            .map_err(serde::de::Error::custom)
    }
}
