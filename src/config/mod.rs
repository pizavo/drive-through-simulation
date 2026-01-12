pub mod customer;
pub mod fixed;
pub mod random;

use fixed::FixedSimConfig;
use random::RandomSimConfig;
use serde::Deserialize;
use std::path::Path;

/// Main configuration structure for the simulation
#[derive(Debug, Deserialize)]
pub struct Config {
    pub fixed_simulation: FixedSimConfig,
    pub random_simulation: RandomSimConfig,
}

impl Config {
    /// Loads configuration from a YAML file
    ///
    /// Also supports environment variable overrides with the prefix "APP__"
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        let config: Self = settings.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }

    /// Validates that at least one simulation is enabled
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.fixed_simulation.enabled && !self.random_simulation.enabled {
            return Err(
                "At least one simulation (fixed or random) must be enabled in config.yaml".into(),
            );
        }
        Ok(())
    }
}
