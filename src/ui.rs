use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::{Frame, Terminal};

use crate::app::QuantumDesk;

pub fn run(app: &mut QuantumDesk) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_loop(app, &mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    app: &mut QuantumDesk,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;

        let poll_timeout = Duration::from_millis(app.refresh_interval_ms().max(100));
        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                },
                Event::Resize(_, _) => {
                    // re-render on next iteration automatically
                }
                _ => {}
            }
        }

        app.tick()?;
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &QuantumDesk) {
    let size = frame.size();
    let mut show_alerts_panel = !app.is_compact();
    let margin = if app.is_compact() { 0 } else { 1 } as u16;

    let metrics = &app.state.metrics_summary;
    let header_text = format!(
        "Venues online: {} | Avg funding: {:+.2} bps",
        metrics.venues_online,
        metrics.average_funding_rate * 10_000.0
    );

    let mut header_lines = vec![
        Line::styled(header_text, Style::default().fg(Color::Cyan)),
        Line::styled(
            app.state.status_line.clone(),
            Style::default().fg(Color::Gray),
        ),
    ];
    if app.is_compact() {
        header_lines.push(Line::from("Press 'q' or Esc to exit"));
    }

    let header_height = header_lines.len() as u16 + 2;

    let mut constraints = vec![Constraint::Length(header_height), Constraint::Min(5)];
    if show_alerts_panel {
        let mut base_lines = 2 + app.state.alerts.len() as u16;
        if !app.state.warnings.is_empty() {
            base_lines += 1 + app.state.warnings.len() as u16;
        }
        base_lines += 1; // exit instructions
        let mut alerts_height = base_lines + 2; // include block borders
        let max_height = size.height.saturating_sub(header_height.saturating_add(5));
        if max_height < 3 {
            show_alerts_panel = false;
        } else {
            alerts_height = alerts_height.clamp(3, max_height);
            constraints.push(Constraint::Length(alerts_height));
        }
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(constraints)
        .split(size);

    let header = Paragraph::new(header_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("QuantumDesk v{}", env!("CARGO_PKG_VERSION"))),
    );
    frame.render_widget(header, chunks[0]);

    let format_rate = |rate: f64| format!("{:.2}%", rate * 100.0);
    let table_rows = app
        .state
        .market_snapshots
        .iter()
        .map(|snapshot| {
            let price = snapshot.perp_price.unwrap_or(snapshot.spot_price);
            let instrument = format!("{}:{}", snapshot.instrument_label, snapshot.symbol);
            let current_rate = format_rate(snapshot.funding_rate);
            let predicted_rate = snapshot
                .predicted_funding_rate
                .map(|rate| format_rate(rate))
                .unwrap_or_else(|| "-".to_string());
            let funding_display = format!("{} / {}", current_rate, predicted_rate);

            Row::new(vec![
                Cell::from(snapshot.venue.clone()),
                Cell::from(instrument),
                Cell::from(format!("{:.2}", price)),
                Cell::from(funding_display),
                Cell::from(
                    snapshot
                        .next_funding_time
                        .map(|ts| ts.format("%H:%M UTC").to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(snapshot.last_updated.format("%H:%M:%S").to_string()),
            ])
        })
        .collect::<Vec<_>>();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Length(14),
        Constraint::Length(18),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(table_rows, widths)
        .header(
            Row::new(vec![
                "Venue",
                "Instrument",
                "Price",
                "Funding (APY)",
                "Next",
                "Updated",
            ])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .title("Funding Monitor")
                .borders(Borders::ALL),
        )
        .column_spacing(1);
    frame.render_widget(table, chunks[1]);

    if show_alerts_panel {
        let mut alert_lines = vec![Line::styled(
            "Alerts".to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )];
        alert_lines.extend(app.state.alerts.iter().map(|alert| {
            let status_color = if alert.is_triggered {
                Color::Red
            } else {
                Color::Gray
            };
            Line::styled(
                format!("• {} | Threshold {}", alert.name, alert.threshold),
                Style::default().fg(status_color),
            )
        }));

        if !app.state.warnings.is_empty() {
            alert_lines.push(Line::from(""));
            alert_lines.push(Line::styled(
                "Warnings".to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
            for warning in &app.state.warnings {
                alert_lines.push(Line::styled(
                    format!("• {}", warning),
                    Style::default().fg(Color::Yellow),
                ));
            }
        }

        alert_lines.push(Line::from(""));
        alert_lines.push(Line::from("Press 'q' or Esc to exit"));

        let alerts = Paragraph::new(alert_lines).block(
            Block::default()
                .title("Alerts & Shortcuts")
                .borders(Borders::ALL),
        );
        let alerts_chunk = chunks[chunks.len() - 1];
        frame.render_widget(alerts, alerts_chunk);
    }
}
