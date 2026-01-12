#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Arrival,
    ServiceStart,
    ServiceEnd,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Arrival => write!(f, "Arrival"),
            EventType::ServiceStart => write!(f, "ServiceStart"),
            EventType::ServiceEnd => write!(f, "ServiceEnd"),
        }
    }
}
