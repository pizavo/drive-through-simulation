use crate::duration::deserialize_duration;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RandomSimConfig {
    pub enabled: bool,
    pub num_windows: usize,
    #[serde(deserialize_with = "deserialize_duration")]
    pub avg_arrival_interval: f64,
    #[serde(deserialize_with = "deserialize_duration")]
    pub min_service_time: f64,
    #[serde(deserialize_with = "deserialize_duration")]
    pub max_service_time: f64,
    #[serde(deserialize_with = "deserialize_duration")]
    pub max_simulation_time: f64,
    pub history_file: String,
}
