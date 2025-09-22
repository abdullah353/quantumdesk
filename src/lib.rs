pub mod ai;
pub mod alerts;
pub mod app;
pub mod config;
pub mod data;
pub mod metrics;
pub mod ui;

pub use app::QuantumDesk;

pub fn run(mut app: QuantumDesk) -> anyhow::Result<()> {
    ui::run(&mut app)
}
