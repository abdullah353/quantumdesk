use anyhow::Result;
use chrono::{Duration, Utc};

use crate::ai::AiOrchestrator;
use crate::alerts::{AlertManager, AlertStatus};
use crate::config::AppConfig;
use crate::data::{DataHub, MarketSnapshot};
use crate::metrics::{MetricsEngine, MetricsSummary};

#[derive(Debug)]
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
                "Bitfinex",
                "tBTCUSD",
                65_420.12,
                None,
                0.0003,
                0.00045,
                Some(Utc::now() + Duration::minutes(50)),
            ),
            MarketSnapshot::placeholder(
                "Bitfinex",
                "tBTCF0:USTF0",
                65_410.25,
                Some(65_430.50),
                0.00065,
                0.00072,
                Some(Utc::now() + Duration::minutes(50)),
            ),
            MarketSnapshot::placeholder(
                "Deribit",
                "BTC-USD",
                65_398.00,
                None,
                0.00028,
                0.00031,
                Some(Utc::now() + Duration::minutes(42)),
            ),
            MarketSnapshot::placeholder(
                "Deribit",
                "BTC-PERPETUAL",
                65_402.89,
                Some(65_420.11),
                0.00052,
                0.0006,
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
        self.state.metrics_summary = self.metrics.summarize(&self.state.market_snapshots);
        self.state.alerts = self.alerts.alerts.clone();
        self.refresh_status_line();
        Ok(())
    }

    fn refresh_status_line(&mut self) {
        self.state.status_line = format!(
            "QuantumDesk • Refresh {}ms • Feed: {} • AI: {} • Triggered alerts: {}",
            self.config.update_interval_ms,
            self.data_hub.status(),
            self.ai.readiness_label(),
            self.alerts.triggered_count()
        );
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub market_snapshots: Vec<MarketSnapshot>,
    pub metrics_summary: MetricsSummary,
    pub alerts: Vec<AlertStatus>,
    pub status_line: String,
}

impl AppState {
    pub fn new(snapshots: Vec<MarketSnapshot>, alerts: Vec<AlertStatus>) -> Self {
        Self {
            market_snapshots: snapshots,
            metrics_summary: MetricsSummary::default(),
            alerts,
            status_line: "QuantumDesk • Press 'q' to quit".into(),
        }
    }
}
