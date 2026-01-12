use crate::event::EventType;

/// Represents a single event in the simulation history
/// Note: Currently unused as we stream events directly to CSV instead of buffering
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub time: f64,
    pub event: EventType,
    pub cust_id: usize,
    pub queue_len: usize,
    pub busy_servers: usize,
}
