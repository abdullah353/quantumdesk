use crate::data::MarketSnapshot;

#[derive(Debug, Default)]
pub struct MetricsEngine;

impl MetricsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn summarize(&self, snapshots: &[MarketSnapshot]) -> MetricsSummary {
        if snapshots.is_empty() {
            return MetricsSummary::default();
        }

        let funding_sum: f64 = snapshots.iter().map(|s| s.funding_rate).sum();
        MetricsSummary {
            venues_online: snapshots.len(),
            average_funding_rate: funding_sum / snapshots.len() as f64,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MetricsSummary {
    pub venues_online: usize,
    pub average_funding_rate: f64,
}
