//! CBOR-friendly number deserializers that accept either integer or float tokens.

use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(untagged)]
enum Number {
    Float(f64),
    Signed(i64),
    Unsigned(u64),
}

impl Number {
    fn as_f64<E: Error>(&self) -> Result<f64, E> {
        let value = match self {
            Self::Float(value) => *value,
            Self::Signed(value) => *value as f64,
            Self::Unsigned(value) => *value as f64,
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err(E::custom("number must be finite"))
        }
    }
}

pub fn f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Number::deserialize(deserializer)?.as_f64()
}

pub fn vec_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<f64>, D::Error> {
    Vec::<Number>::deserialize(deserializer)?
        .iter()
        .map(Number::as_f64)
        .collect()
}

pub fn option_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
    Option::<Number>::deserialize(deserializer)?
        .as_ref()
        .map(Number::as_f64)
        .transpose()
}
