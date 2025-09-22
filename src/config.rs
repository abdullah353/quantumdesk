use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub update_interval_ms: u64,
    pub venues: Vec<VenueConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1_000,
            venues: vec![
                VenueConfig {
                    name: "Bitfinex".into(),
                    symbols: vec!["tBTCUSD".into(), "tBTCF0:USTF0".into()],
                },
                VenueConfig {
                    name: "Deribit".into(),
                    symbols: vec!["BTC-USD".into(), "BTC-PERPETUAL".into()],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueConfig {
    pub name: String,
    pub symbols: Vec<String>,
}
