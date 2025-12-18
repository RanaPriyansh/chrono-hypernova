use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use askama::Template;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::types::GlobalMessage;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct WebMarketData {
    pub asset_name: String,
    pub strike: f64,
    pub fair_value: f64,
    pub best_ask: f64,
    pub edge: f64,
}

pub struct AppState {
    pub markets: Mutex<HashMap<String, WebMarketData>>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    markets: Vec<WebMarketData>,
}

#[get("/")]
async fn index(data: web::Data<Arc<AppState>>) -> impl Responder {
    let markets_map = data.markets.lock().unwrap();
    let mut markets_vec: Vec<WebMarketData> = markets_map.values().cloned().collect();
    
    // Sort by edge descending
    markets_vec.sort_by(|a, b| b.edge.partial_cmp(&a.edge).unwrap());

    let html = DashboardTemplate { markets: markets_vec }.render().unwrap();
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub struct WebDashboard {
    rx: broadcast::Receiver<GlobalMessage>,
    state: Arc<AppState>,
}

impl WebDashboard {
    pub fn new(rx: broadcast::Sender<GlobalMessage>) -> Self {
        Self {
            rx: rx.subscribe(),
            state: Arc::new(AppState {
                markets: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub async fn run(mut self) -> std::io::Result<()> {
        let state = self.state.clone();
        
        // Background task to update state from broadcast
        tokio::spawn(async move {
            while let Ok(msg) = self.rx.recv().await {
                let mut markets = self.state.markets.lock().unwrap();
                match msg {
                    GlobalMessage::FairValueUpdate(fv) => {
                        let entry = markets.entry(fv.market_id.clone()).or_insert(WebMarketData {
                            asset_name: "Unknown".into(),
                            strike: 0.0,
                            fair_value: 0.0,
                            best_ask: 0.0,
                            edge: 0.0,
                        });
                        entry.fair_value = fv.fair_price;
                        entry.edge = entry.fair_value - entry.best_ask;
                    }
                    GlobalMessage::PolymarketUpdate(book) => {
                        let entry = markets.entry(book.market_id.clone()).or_insert(WebMarketData {
                            asset_name: "Unknown".into(),
                            strike: 0.0,
                            fair_value: 0.0,
                            best_ask: 0.0,
                            edge: 0.0,
                        });
                        entry.best_ask = book.best_ask;
                        entry.edge = entry.fair_value - entry.best_ask;
                    }
                    GlobalMessage::MarketsDiscovered(new_markets) => {
                        for m in new_markets {
                            let entry = markets.entry(m.market_id.clone()).or_insert(WebMarketData {
                                asset_name: format!("{:?}", m.asset),
                                strike: m.strike,
                                fair_value: 0.0,
                                best_ask: 0.0,
                                edge: 0.0,
                            });
                            entry.asset_name = format!("{:?}", m.asset);
                            entry.strike = m.strike;
                        }
                    }
                    _ => {}
                }
            }
        });

        println!("Starting Web Dashboard at http://0.0.0.0:3000");
        
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(index)
        })
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
    }
}
