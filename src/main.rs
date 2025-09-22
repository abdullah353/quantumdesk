use clap::Parser;
use quantumdesk::QuantumDesk;

#[derive(Parser, Debug)]
#[command(author, version, about = "QuantumDesk - AI-powered funding monitor", long_about = None)]
struct Cli {
    /// Milliseconds between UI refresh ticks
    #[arg(long = "refresh-ms", default_value_t = 1_000)]
    refresh_ms: u64,

    /// Seconds to cache exchange responses before re-fetching
    #[arg(long = "cache-ttl", default_value_t = 60)]
    cache_ttl_secs: u64,

    /// Render a condensed layout with minimal chrome
    #[arg(long = "compact", default_value_t = false)]
    compact: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = quantumdesk::config::AppConfig::with_overrides(
        Some(cli.refresh_ms),
        Some(cli.cache_ttl_secs),
        cli.compact,
    );
    quantumdesk::run(QuantumDesk::new(config))
}
