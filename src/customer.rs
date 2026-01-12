/// Represents a customer in the simulation.
#[derive(Debug, Clone)]
pub struct Customer {
    pub arrival_time: f64,
    pub service_duration: f64,
    pub service_start_time: Option<f64>,
    pub service_end_time: Option<f64>,
}
