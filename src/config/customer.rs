use crate::duration::deserialize_duration;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FixedCustomerConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub arrival: f64,
    #[serde(deserialize_with = "deserialize_duration")]
    pub service: f64,
}
