use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    pub venue: String,
    pub instrument_label: String,
    pub symbol: String,
    pub spot_price: f64,
    pub perp_price: Option<f64>,
    pub funding_rate: f64,
    pub predicted_funding_rate: Option<f64>,
    pub next_funding_time: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl MarketSnapshot {
    pub fn placeholder(
        venue: impl Into<String>,
        instrument_label: impl Into<String>,
        symbol: impl Into<String>,
        spot_price: f64,
        perp_price: Option<f64>,
        funding_rate: f64,
        predicted_funding_rate: Option<f64>,
        next_funding_time: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            venue: venue.into(),
            instrument_label: instrument_label.into(),
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

pub struct CollectionOutcome {
    pub snapshots: Vec<MarketSnapshot>,
    pub warnings: Vec<String>,
}

pub struct DataHub {
    http: Client,
    cache: HashMap<String, CachedSnapshot>,
    status_label: String,
}

struct CachedSnapshot {
    snapshot: MarketSnapshot,
    fetched_at: DateTime<Utc>,
}

enum SnapshotOutcome {
    Fresh(MarketSnapshot),
    Stale(MarketSnapshot, String),
}

impl DataHub {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent("QuantumDesk/0.1 (https://github.com/quantumdesk)")
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            cache: HashMap::new(),
            status_label: "Initializing feeds".into(),
        }
    }

    pub fn status(&self) -> &str {
        &self.status_label
    }

    pub fn collect(&mut self, config: &AppConfig) -> CollectionOutcome {
        let ttl = ChronoDuration::seconds(config.cache_ttl_secs as i64);
        let mut snapshots = Vec::new();
        let mut warnings = Vec::new();

        for venue in &config.venues {
            for symbol in &venue.symbols {
                match self.load_snapshot(&venue.name, symbol, ttl) {
                    Ok(SnapshotOutcome::Fresh(snapshot)) => snapshots.push(snapshot),
                    Ok(SnapshotOutcome::Stale(snapshot, warning)) => {
                        snapshots.push(snapshot);
                        warnings.push(warning);
                    }
                    Err(err) => warnings.push(err.to_string()),
                }
            }
        }

        self.status_label = if warnings.is_empty() {
            "Live feeds stable".into()
        } else {
            "Live feeds (degraded)".into()
        };

        CollectionOutcome {
            snapshots,
            warnings,
        }
    }

    fn load_snapshot(
        &mut self,
        venue: &str,
        symbol: &str,
        ttl: ChronoDuration,
    ) -> Result<SnapshotOutcome> {
        let key = cache_key(venue, symbol);
        let now = Utc::now();

        if let Some(entry) = self.cache.get(&key) {
            let age = now - entry.fetched_at;
            if age < ttl {
                return Ok(SnapshotOutcome::Fresh(entry.snapshot.clone()));
            }
        }

        match self.fetch_snapshot(venue, symbol) {
            Ok(mut snapshot) => {
                snapshot.last_updated = now;
                self.cache.insert(
                    key,
                    CachedSnapshot {
                        snapshot: snapshot.clone(),
                        fetched_at: now,
                    },
                );
                Ok(SnapshotOutcome::Fresh(snapshot))
            }
            Err(fetch_err) => {
                if let Some(entry) = self.cache.get(&key) {
                    return Ok(SnapshotOutcome::Stale(
                        entry.snapshot.clone(),
                        format!(
                            "{} {} fetch failed ({}); showing cached data",
                            venue, symbol, fetch_err
                        ),
                    ));
                }
                Err(anyhow!(
                    "{} {} fetch failed and no cache available ({})",
                    venue,
                    symbol,
                    fetch_err
                ))
            }
        }
    }

    fn fetch_snapshot(&self, venue: &str, symbol: &str) -> Result<MarketSnapshot> {
        match (venue, symbol) {
            ("Bitfinex", "tBTCUSD") => self.fetch_bitfinex_spot(symbol),
            ("Bitfinex", "tBTCF0:USTF0") => self.fetch_bitfinex_perp(symbol),
            ("Deribit", "BTC-USD") => self.fetch_deribit_index(symbol),
            ("Deribit", "BTC-PERPETUAL") => self.fetch_deribit_perp(symbol),
            _ => Err(anyhow!(
                "unsupported venue/symbol combination: {} {}",
                venue,
                symbol
            )),
        }
    }

    fn fetch_bitfinex_spot(&self, symbol: &str) -> Result<MarketSnapshot> {
        let url = format!("https://api-pub.bitfinex.com/v2/ticker/{}", symbol);
        let data: Vec<f64> = self
            .http
            .get(url)
            .send()
            .context("bitfinex spot request failed")?
            .json()
            .context("bitfinex spot payload parse failed")?;

        let last_price = *data
            .get(6)
            .context("bitfinex ticker missing last price field")?;

        Ok(MarketSnapshot {
            venue: "Bitfinex".into(),
            instrument_label: "Spot".into(),
            symbol: symbol.into(),
            spot_price: last_price,
            perp_price: None,
            funding_rate: 0.0,
            predicted_funding_rate: None,
            next_funding_time: None,
            last_updated: Utc::now(),
        })
    }

    fn fetch_bitfinex_perp(&self, symbol: &str) -> Result<MarketSnapshot> {
        let ticker_url = format!("https://api-pub.bitfinex.com/v2/ticker/{}", symbol);
        let ticker: Vec<f64> = self
            .http
            .get(ticker_url)
            .send()
            .context("bitfinex perp ticker request failed")?
            .json()
            .context("bitfinex perp ticker parse failed")?;

        let last_price = *ticker
            .get(6)
            .context("bitfinex perp ticker missing last price")?;

        let status_url = format!(
            "https://api-pub.bitfinex.com/v2/status/deriv?keys={}",
            symbol
        );
        let status_payload: Vec<Vec<Value>> = self
            .http
            .get(status_url)
            .send()
            .context("bitfinex deriv status request failed")?
            .json()
            .context("bitfinex deriv status parse failed")?;

        let entry = status_payload
            .into_iter()
            .next()
            .context("bitfinex deriv status empty")?;

        let next_funding_time = entry
            .get(8)
            .and_then(|value| value.as_i64())
            .and_then(ms_to_datetime);
        let funding_rate = entry.get(9).and_then(|value| value.as_f64()).unwrap_or(0.0);
        let predicted_funding_rate = entry.get(12).and_then(|value| value.as_f64());

        let mark_price = entry
            .get(3)
            .and_then(|value| value.as_f64())
            .unwrap_or(last_price);

        Ok(MarketSnapshot {
            venue: "Bitfinex".into(),
            instrument_label: "Perp".into(),
            symbol: symbol.into(),
            spot_price: mark_price,
            perp_price: Some(last_price),
            funding_rate,
            predicted_funding_rate,
            next_funding_time,
            last_updated: Utc::now(),
        })
    }

    fn fetch_deribit_index(&self, symbol: &str) -> Result<MarketSnapshot> {
        let url = "https://www.deribit.com/api/v2/public/get_index_price?index_name=btc_usd";
        let resp: DeribitIndexResponse = self
            .http
            .get(url)
            .send()
            .context("deribit index request failed")?
            .json()
            .context("deribit index parse failed")?;

        let DeribitIndexResponse { result } = resp;
        let updated_at = result
            .timestamp
            .and_then(ms_to_datetime)
            .unwrap_or_else(|| Utc::now());

        Ok(MarketSnapshot {
            venue: "Deribit".into(),
            instrument_label: "Index".into(),
            symbol: symbol.into(),
            spot_price: result.index_price,
            perp_price: None,
            funding_rate: 0.0,
            predicted_funding_rate: None,
            next_funding_time: None,
            last_updated: updated_at,
        })
    }

    fn fetch_deribit_perp(&self, symbol: &str) -> Result<MarketSnapshot> {
        let url = format!(
            "https://www.deribit.com/api/v2/public/ticker?instrument_name={}",
            symbol
        );
        let resp: DeribitTickerResponse = self
            .http
            .get(url)
            .send()
            .context("deribit ticker request failed")?
            .json()
            .context("deribit ticker parse failed")?;

        let result = resp.result;
        let index_price = result.index_price;
        let mark_price = result
            .mark_price
            .or(result.last_price)
            .or(index_price)
            .unwrap_or(0.0);
        let spot_price = index_price.unwrap_or(mark_price);
        let last_updated = result
            .timestamp
            .and_then(ms_to_datetime)
            .unwrap_or_else(|| Utc::now());
        let next_funding = result.next_funding_time.and_then(ms_to_datetime);

        Ok(MarketSnapshot {
            venue: "Deribit".into(),
            instrument_label: "Perp".into(),
            symbol: symbol.into(),
            spot_price,
            perp_price: Some(mark_price),
            funding_rate: result.current_funding.unwrap_or(0.0),
            predicted_funding_rate: result.funding_8h,
            next_funding_time: next_funding,
            last_updated,
        })
    }
}

fn cache_key(venue: &str, symbol: &str) -> String {
    format!("{}::{}", venue, symbol)
}

fn ms_to_datetime(ms: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_millis(ms)
}

#[derive(Debug, Deserialize)]
struct DeribitIndexResponse {
    result: DeribitIndexResult,
}

#[derive(Debug, Deserialize)]
struct DeribitIndexResult {
    index_price: f64,
    timestamp: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DeribitTickerResponse {
    result: DeribitTickerResult,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct DeribitTickerResult {
    instrument_name: String,
    mark_price: Option<f64>,
    last_price: Option<f64>,
    index_price: Option<f64>,
    current_funding: Option<f64>,
    funding_8h: Option<f64>,
    next_funding_time: Option<i64>,
    timestamp: Option<i64>,
}
