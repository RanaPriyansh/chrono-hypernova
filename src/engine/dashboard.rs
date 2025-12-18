use crate::types::{GlobalMessage, FairValueUpdate, OrderbookUpdate};
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};
use std::io::{self, Stdout};

struct DashboardState {
    fair_values: HashMap<String, f64>,
    market_prices: HashMap<String, (f64, f64)>, // Bid, Ask
    logs: Vec<String>,
}

pub struct DashboardActor {
    rx: broadcast::Receiver<GlobalMessage>,
    state: DashboardState,
}

impl DashboardActor {
    pub fn new(rx: broadcast::Sender<GlobalMessage>) -> Self {
        Self {
            rx: rx.subscribe(),
            state: DashboardState {
                fair_values: HashMap::new(),
                market_prices: HashMap::new(),
                logs: Vec::new(),
            },
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        // Setup Terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main Loop
        let mut tick_rate = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = tick_rate.tick() => {
                    terminal.draw(|f| self.ui(f))?;
                }
                Ok(msg) = self.rx.recv() => {
                    self.update_state(msg);
                }
            }

            // Handle Input (Q to quit)
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        break;
                    }
                }
            }
        }

        // Cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn update_state(&mut self, msg: GlobalMessage) {
        match msg {
            GlobalMessage::FairValueUpdate(fv) => {
                self.state.fair_values.insert(fv.market_id, fv.fair_price);
            }
            GlobalMessage::PolymarketUpdate(book) => {
                self.state.market_prices.insert(book.market_id, (book.best_bid, book.best_ask));
            }
            _ => {}
        }
        
        // Keep last 20 logs (Mocking logs for now since tracing goes to stdout)
        // In a real app we'd attach a tracing subscriber that sends to a channel this actor reads
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(10), // Header
                Constraint::Percentage(50), // Markets Table
                Constraint::Percentage(40), // Logs
            ])
            .split(f.size());

        // 1. Header
        let header = Paragraph::new("PolyArb v1.0 - The Sniper")
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        f.render_widget(header, chunks[0]);

        // 2. Markets Table
        let rows: Vec<Row> = self.state.fair_values.iter().map(|(id, fv)| {
            let (bid, ask) = self.state.market_prices.get(id).unwrap_or(&(0.0, 0.0));
            // Calculate Edge
            let edge = fv - ask;
            let edge_style = if edge > 0.02 { Style::default().fg(Color::Green) } else { Style::default() };

            Row::new(vec![
                id.clone(),
                format!("{:.4}", fv),
                format!("{:.4}", bid),
                format!("{:.4}", ask),
                format!("{:.4}", edge),
            ]).style(edge_style)
        }).collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]
        )
        .header(Row::new(vec!["Market", "Fair Value", "Bid", "Ask", "Edge"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title("Live Pricing"));
        
        f.render_widget(table, chunks[1]);
        
        // 3. Footer/Logs
        let footer = Paragraph::new("Press 'q' to exit.")
            .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, chunks[2]);
    }
}
