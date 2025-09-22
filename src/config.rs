use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub update_interval_ms: u64,
    pub cache_ttl_secs: u64,
    pub venues: Vec<VenueConfig>,
    pub compact_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1_000,
            cache_ttl_secs: 60,
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
            compact_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueConfig {
    pub name: String,
    pub symbols: Vec<String>,
}

impl AppConfig {
    pub fn with_overrides(
        update_interval_ms: Option<u64>,
        cache_ttl_secs: Option<u64>,
        compact_mode: bool,
    ) -> Self {
        let mut base = Self::default();
        if let Some(ms) = update_interval_ms {
            base.update_interval_ms = ms.max(100);
        }
        if let Some(ttl) = cache_ttl_secs {
            base.cache_ttl_secs = ttl.max(5);
        }
        base.compact_mode = compact_mode;
        base
    }
}
