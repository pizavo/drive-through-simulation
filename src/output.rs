use crate::event::EventType;

/// Message sent to the output thread for ordered printing
#[derive(Debug, Clone)]
pub struct OutputMessage {
    pub time: f64,
    pub event: EventType,
    pub cust_id: usize,
    pub queue_len: usize,
    pub busy_servers: usize,
    pub num_windows: usize,
}

