use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    pub venue: String,
    pub symbol: String,
    pub spot_price: f64,
    pub perp_price: Option<f64>,
    pub funding_rate: f64,
    pub predicted_funding_rate: f64,
    pub next_funding_time: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl MarketSnapshot {
    pub fn placeholder(
        venue: impl Into<String>,
        symbol: impl Into<String>,
        spot_price: f64,
        perp_price: Option<f64>,
        funding_rate: f64,
        predicted_funding_rate: f64,
        next_funding_time: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            venue: venue.into(),
            symbol: symbol.into(),
            spot_price,
            perp_price,
            funding_rate,
            predicted_funding_rate,
            next_funding_time,
            last_updated: Utc::now(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DataHub;

impl DataHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(&self) -> &'static str {
        "Simulated feed"
    }
}
