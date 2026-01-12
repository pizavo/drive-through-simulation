use super::customer::FixedCustomerConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FixedSimConfig {
    pub enabled: bool,
    pub num_windows: usize,
    pub customers: Vec<FixedCustomerConfig>,
    pub history_file: String,
}
