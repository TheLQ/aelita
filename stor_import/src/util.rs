use serde::{Deserialize, Deserializer};

pub fn none_on_negative_deserializer<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = i64::deserialize(deserializer)?;
    if raw < 0 {
        Ok(None)
    } else {
        Ok(Some(raw as u64))
    }
}
