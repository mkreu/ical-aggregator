use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use icalendar::{Calendar, Component, Event, EventLike};
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use arc_swap::ArcSwap;

#[derive(Debug, Deserialize)]
struct Config {
    feeds: Vec<String>,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_refresh_interval")]
    refresh_interval_seconds: u64,
}

fn default_port() -> u16 {
    3000
}

fn default_refresh_interval() -> u64 {
    300 // 5 minutes
}

// Shared state for cached calendar
#[derive(Clone)]
struct AppState {
    cached_calendar: Arc<ArcSwap<String>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config_content = fs::read_to_string("config.toml")
        .expect("Failed to read config.toml");
    let config: Config = toml::from_str(&config_content)
        .expect("Failed to parse config.toml");

    info!("Loaded {} feed URLs from config", config.feeds.len());
    info!("Refresh interval: {} seconds", config.refresh_interval_seconds);

    // Create shared state for cached calendar
    let cached_calendar = Arc::new(ArcSwap::from_pointee(String::new()));
    let state = AppState {
        cached_calendar: cached_calendar.clone(),
    };

    // Spawn background task to refresh calendar periodically
    let feed_urls = config.feeds.clone();
    let refresh_interval = config.refresh_interval_seconds;
    tokio::spawn(async move {
        refresh_calendar_loop(feed_urls, cached_calendar, refresh_interval).await;
    });

    // Build the router
    let app = Router::new()
        .route("/calendar.ics", get(serve_cached_calendar))
        .with_state(state);

    // Start the server
    let addr = format!("0.0.0.0:{}", config.port);
    info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn refresh_calendar_loop(
    feed_urls: Vec<String>,
    cached_calendar: Arc<ArcSwap<String>>,
    refresh_interval_seconds: u64,
) {
    loop {
        info!("Refreshing calendar cache...");
        
        match fetch_and_merge_calendars(&feed_urls).await {
            Ok(merged_ical) => {
                cached_calendar.store(Arc::new(merged_ical));
                info!("Calendar cache updated successfully");
            }
            Err(e) => {
                tracing::error!("Failed to refresh calendar: {}", e);
            }
        }
        
        tokio::time::sleep(Duration::from_secs(refresh_interval_seconds)).await;
    }
}

async fn serve_cached_calendar(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let calendar = state.cached_calendar.load();
    
    if calendar.is_empty() {
        return Err(AppError(Box::from("Calendar not yet loaded")));
    }
    
    Ok((
        StatusCode::OK,
        [("Content-Type", "text/calendar; charset=utf-8")],
        calendar.to_string(),
    ))
}

async fn fetch_and_merge_calendars(feed_urls: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    // Fetch all calendars concurrently
    let client = reqwest::Client::new();
    let mut fetch_tasks = Vec::new();

    for url in feed_urls {
        let client = client.clone();
        let url = url.clone();
        let task = tokio::spawn(async move {
            let response = client.get(&url).send().await?;
            let text = response.text().await?;
            Ok::<String, reqwest::Error>(text)
        });
        fetch_tasks.push(task);
    }

    // Wait for all fetches to complete
    let results = futures::future::join_all(fetch_tasks).await;

    // Create a new merged calendar
    let mut merged_calendar = Calendar::new();
    merged_calendar.name("Merged Calendar");
    merged_calendar.description("Aggregated from multiple iCal feeds");

    // Parse and merge all calendars
    for result in results {
        match result {
            Ok(Ok(ical_text)) => {
                if let Err(e) = merge_calendar_events(&ical_text, &mut merged_calendar) {
                    tracing::warn!("Failed to parse calendar: {}", e);
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("Failed to fetch calendar: {}", e);
            }
            Err(e) => {
                tracing::warn!("Task failed: {}", e);
            }
        }
    }

    // Return the merged calendar as iCal format
    Ok(merged_calendar.to_string())
}

fn merge_calendar_events(ical_text: &str, merged: &mut Calendar) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the iCal text using the parser
    let unfolded = icalendar::parser::unfold(ical_text);
    let parsed = icalendar::parser::read_calendar(&unfolded)?;
    
    // Extract events and add them to the merged calendar
    for component in parsed.components {
        if component.name == "VEVENT" {
            // Create a new event from the parsed component
            let mut event = Event::new();
            
            for property in component.properties {
                match property.name.as_ref() {
                    "UID" => { event.uid(property.val.as_ref()); }
                    "SUMMARY" => { event.summary(property.val.as_ref()); }
                    "DESCRIPTION" => { event.description(property.val.as_ref()); }
                    "LOCATION" => { event.location(property.val.as_ref()); }
                    "DTSTART" => { 
                        event.add_property("DTSTART", property.val.as_ref());
                    }
                    "DTEND" => { 
                        event.add_property("DTEND", property.val.as_ref());
                    }
                    "DTSTAMP" => { 
                        event.add_property("DTSTAMP", property.val.as_ref());
                    }
                    _ => { 
                        event.add_property(property.name.as_ref(), property.val.as_ref());
                    }
                }
            }
            
            merged.push(event);
        }
    }
    
    Ok(())
}

// Error handling
struct AppError(Box<dyn std::error::Error>);

impl<E> From<E> for AppError
where
    E: std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        AppError(Box::new(err))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Application error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error: {}", self.0),
        )
            .into_response()
    }
}
