# QuantumDesk Roadmap

QuantumDesk aims to become the most capable AI-powered trading terminal. This roadmap highlights near-term milestones and the long-term vision.

## 0.1.0 – Foundation (In Progress)
- [ ] Ratatui-based core layout with responsive resizing and theming.
- [ ] Async runtime scaffolding (Tokio) for polling exchanges concurrently.
- [ ] Bitfinex and Deribit REST/WebSocket clients with shared market schema.
- [ ] Funding monitor widgets (spot vs perp, predicted funding, next settlement).
- [ ] Local alert manager with threshold configuration and terminal notifications.
- [ ] Config system (TOML + env overrides) for API keys and refresh cadence.
- [ ] Telescope-style command palette for symbols, panes, and alert/A.I. actions.

## 0.2.0 – Intelligence Layer
- [ ] MCP-compatible AI orchestrator for delegated analysis and plug-in agents.
- [ ] LLM-based alert explanations and strategy suggestions.
- [ ] Funding prediction models (gradient boosting + venue-specific adjustments).
- [ ] IBIT/FBTC ETF normalization vs BTC index price.
- [ ] Persistence of historical snapshots (SQLite/Parquet) for trend panes.

## 0.3.0 – Connectivity Expansion
- [ ] Add Binance, OKX, Bybit, CME crypto futures, and FX spot feeds.
- [ ] Pluggable data adapters (`ExchangeProvider` trait + dynamic registry).
- [ ] Plugin SDK so community can ship indicators, risk modules, or macros.
- [ ] Streaming economic calendar (FOMC, CPI, NFP) with alert hooks.
- [ ] Macro news sentiment feed (e.g., Fed speeches, ETF net inflows).

## 0.4.0 – Multi-Channel Experience
- [ ] Terminal notifier integration (Unix `notify-send`, macOS `terminal-notifier`).
- [ ] Web companion dashboard (Tauri or web assembly build).
- [ ] Mobile-friendly alerts via push gateway.
- [ ] Role-based workspace profiles (trader, researcher, quant).

## 1.0 – Global Desk
- [ ] Battle-tested release with CI/CD, packaging, and installers.
- [ ] 24/7 reliability checks (heartbeat monitor, auto-reconnect, rate-limit governance).
- [ ] Backtesting playground with AI-assisted strategy prototyping.
- [ ] Community strategy marketplace and leaderboard.
- [ ] Full documentation site with tutorials, recipes, and integrations.

## Contributing to the Roadmap
- Look for GitHub milestones (e.g., `milestone:v0.1.0`) to see active objectives.
- Use Discussions to pitch alterations or new modules before implementation.
- Submit design docs for large features—architecture debates are encouraged.

*This roadmap will evolve as we gather feedback. Help shape it by opening an issue or joining the community calls.*
