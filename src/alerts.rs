use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct AlertStatus {
    pub name: String,
    pub is_triggered: bool,
    pub threshold: String,
    pub last_triggered: Option<DateTime<Utc>>,
}

impl AlertStatus {
    pub fn placeholder(name: impl Into<String>, threshold: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_triggered: false,
            threshold: threshold.into(),
            last_triggered: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct AlertManager {
    pub alerts: Vec<AlertStatus>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn triggered_count(&self) -> usize {
        self.alerts
            .iter()
            .filter(|alert| alert.is_triggered)
            .count()
    }
}
