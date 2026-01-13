pub mod customer;
pub mod fixed;
pub mod random;

use fixed::FixedSimConfig;
use random::RandomSimConfig;
use serde::Deserialize;
use std::path::Path;

// External config crate (avoid confusion with local config module)
use config as config_crate;

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
        let settings = config_crate::Config::builder()
            .add_source(config_crate::File::from(path.as_ref()))
            .add_source(config_crate::Environment::with_prefix("APP").separator("__"))
            .build()?;

        let mut config: Self = settings.try_deserialize()?;
        config.validate()?;
        config.normalize()?;
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

    /// Normalizes configuration data (e.g., sorts customers by arrival time)
    fn normalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Sort fixed simulation customers by arrival time
        // This is critical because the simulation processes them sequentially
        if self.fixed_simulation.enabled {
            self.fixed_simulation.customers.sort_by(|a, b| {
                a.arrival
                    .partial_cmp(&b.arrival)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        Ok(())
    }
}
