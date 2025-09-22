use anyhow::Result;
use chrono::{Duration, Utc};

use crate::ai::AiOrchestrator;
use crate::alerts::{AlertManager, AlertStatus};
use crate::config::AppConfig;
use crate::data::{CollectionOutcome, DataHub, MarketSnapshot};
use crate::metrics::{MetricsEngine, MetricsSummary};

pub struct QuantumDesk {
    pub state: AppState,
    config: AppConfig,
    data_hub: DataHub,
    metrics: MetricsEngine,
    alerts: AlertManager,
    ai: AiOrchestrator,
}

impl Default for QuantumDesk {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}

impl QuantumDesk {
    pub fn new(config: AppConfig) -> Self {
        let data_hub = DataHub::new();
        let metrics = MetricsEngine::new();
        let alerts = AlertManager::new();
        let ai = AiOrchestrator::new();

        let market_snapshots = vec![
            MarketSnapshot::placeholder(
                "Bitfinex", "Spot", "tBTCUSD", 65_420.12, None, 0.0, None, None,
            ),
            MarketSnapshot::placeholder(
                "Bitfinex",
                "Perp",
                "tBTCF0:USTF0",
                65_410.25,
                Some(65_430.50),
                0.00065,
                Some(0.00072),
                Some(Utc::now() + Duration::minutes(50)),
            ),
            MarketSnapshot::placeholder(
                "Deribit", "Index", "BTC-USD", 65_398.00, None, 0.0, None, None,
            ),
            MarketSnapshot::placeholder(
                "Deribit",
                "Perp",
                "BTC-PERPETUAL",
                65_402.89,
                Some(65_420.11),
                0.00052,
                Some(0.0006),
                Some(Utc::now() + Duration::minutes(42)),
            ),
        ];

        let mut alert_manager = alerts;
        alert_manager.alerts = vec![
            AlertStatus::placeholder("Bitfinex Funding", "> 75 bps"),
            AlertStatus::placeholder("Deribit Funding", "< -25 bps"),
            AlertStatus::placeholder("IBIT Premium", "> 1.5%"),
        ];

        let mut state = AppState::new(market_snapshots, alert_manager.alerts.clone());
        state.metrics_summary = metrics.summarize(&state.market_snapshots);

        let mut desk = Self {
            state,
            config,
            data_hub,
            metrics,
            alerts: alert_manager,
            ai,
        };
        desk.refresh_status_line();
        desk
    }

    pub fn tick(&mut self) -> Result<()> {
        let CollectionOutcome {
            snapshots,
            warnings,
        } = self.data_hub.collect(&self.config);

        if !snapshots.is_empty() {
            self.state.market_snapshots = snapshots;
        }
        self.state.warnings = warnings;
        self.state.metrics_summary = self.metrics.summarize(&self.state.market_snapshots);
        self.state.alerts = self.alerts.alerts.clone();
        self.refresh_status_line();
        Ok(())
    }

    pub fn refresh_interval_ms(&self) -> u64 {
        self.config.update_interval_ms
    }

    pub fn is_compact(&self) -> bool {
        self.config.compact_mode
    }

    fn refresh_status_line(&mut self) {
        let mut parts = vec![
            format!(
                "Mode {}",
                if self.is_compact() { "compact" } else { "full" }
            ),
            format!("Refresh {}ms", self.config.update_interval_ms),
            format!("Cache {}s", self.config.cache_ttl_secs),
            format!("Feed {}", self.data_hub.status()),
            format!("AI {}", self.ai.readiness_label()),
            format!("Alerts {}", self.alerts.triggered_count()),
        ];

        if let Some(warning) = summarize_warnings(&self.state.warnings) {
            parts.push(warning);
        } else {
            parts.push("Feeds healthy".into());
        }

        self.state.status_line = parts.join(" | ");
    }
}

fn summarize_warnings(warnings: &[String]) -> Option<String> {
    if warnings.is_empty() {
        return None;
    }

    let count = warnings.len();
    let suffix = if count > 1 { "warnings" } else { "warning" };
    let preview = warnings
        .first()
        .map(|warning| truncate_for_status(warning))
        .unwrap_or_default();
    Some(format!("{} {}: {}", count, suffix, preview))
}

fn truncate_for_status(text: &str) -> String {
    const MAX_LEN: usize = 80;
    if text.len() <= MAX_LEN {
        return text.to_string();
    }

    let mut truncated = String::with_capacity(MAX_LEN + 1);
    for (idx, ch) in text.chars().enumerate() {
        if idx >= MAX_LEN {
            truncated.push('…');
            break;
        }
        truncated.push(ch);
    }
    truncated
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub market_snapshots: Vec<MarketSnapshot>,
    pub metrics_summary: MetricsSummary,
    pub alerts: Vec<AlertStatus>,
    pub warnings: Vec<String>,
    pub status_line: String,
}

impl AppState {
    pub fn new(snapshots: Vec<MarketSnapshot>, alerts: Vec<AlertStatus>) -> Self {
        Self {
            market_snapshots: snapshots,
            metrics_summary: MetricsSummary::default(),
            alerts,
            warnings: Vec::new(),
            status_line: "QuantumDesk • Press 'q' to quit".into(),
        }
    }
}
