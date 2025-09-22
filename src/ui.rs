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

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    // Re-render on next loop iteration automatically
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(size);

    let metrics = &app.state.metrics_summary;
    let header_text = format!(
        "Venues online: {} | Avg funding: {:+.2} bps",
        metrics.venues_online,
        metrics.average_funding_rate * 10_000.0
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("QuantumDesk v{}", env!("CARGO_PKG_VERSION"))),
        );
    frame.render_widget(header, chunks[0]);

    let table_rows = app
        .state
        .market_snapshots
        .iter()
        .map(|snapshot| {
            Row::new(vec![
                Cell::from(snapshot.venue.clone()),
                Cell::from(snapshot.symbol.clone()),
                Cell::from(format!("{:.2}", snapshot.spot_price)),
                Cell::from(
                    snapshot
                        .perp_price
                        .map(|price| format!("{:.2}", price))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(format!("{:+.2} bps", snapshot.funding_rate * 10_000.0)),
                Cell::from(format!(
                    "{:+.2} bps",
                    snapshot.predicted_funding_rate * 10_000.0
                )),
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
        Constraint::Length(15),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(table_rows, widths)
        .header(
            Row::new(vec![
                "Venue",
                "Symbol",
                "Spot",
                "Perp",
                "Funding",
                "Predicted",
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

    let mut alert_lines = vec![Line::from("Alerts".to_string())];
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
    alert_lines.push(Line::from(""));
    alert_lines.push(Line::from("Press 'q' or Esc to exit"));

    let alerts = Paragraph::new(alert_lines).block(
        Block::default()
            .title("Alerts & Shortcuts")
            .borders(Borders::ALL),
    );
    frame.render_widget(alerts, chunks[2]);
}
