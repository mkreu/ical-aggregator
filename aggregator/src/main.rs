use arc_swap::ArcSwap;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use icalendar::{Calendar, CalendarComponent, Component};
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
struct CalendarFeed {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    feeds: Vec<CalendarFeed>,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_refresh_interval")]
    refresh_interval_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct Rule {
    name: String,
    conditions: Vec<Condition>,
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize, Clone)]
struct Condition {
    field: String,
    op: ConditionOp,
    value: String,
}

#[derive(Debug, Deserialize, Clone)]
enum ConditionOp {
    Contains,
}

#[derive(Debug, Deserialize, Clone)]
struct Action {
    field: String,
    op: ActionOp,
    value: String,
}

#[derive(Debug, Deserialize, Clone)]
enum ActionOp {
    Set,
    Prepend,
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
    let config_content = fs::read_to_string("config.toml").expect("Failed to read config.toml");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse config.toml");

    info!("Loaded {} feed URLs from config", config.feeds.len());
    info!(
        "Refresh interval: {} seconds",
        config.refresh_interval_seconds
    );

    // Create shared state for cached calendar
    let cached_calendar = Arc::new(ArcSwap::from_pointee(String::new()));
    let state = AppState {
        cached_calendar: cached_calendar.clone(),
    };

    // Spawn background task to refresh calendar periodically
    let refresh_interval = config.refresh_interval_seconds;
    tokio::spawn(async move {
        refresh_calendar_loop(
            config.feeds,
            config.rules,
            cached_calendar,
            refresh_interval,
        )
        .await;
    });

    // Build the router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/calendar.ics", get(serve_cached_calendar))
        .with_state(state);

    // Start the server
    let addr = format!("0.0.0.0:{}", config.port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server failed");
}

async fn refresh_calendar_loop(
    feeds: Vec<CalendarFeed>,
    rules: Vec<Rule>,
    cached_calendar: Arc<ArcSwap<String>>,
    refresh_interval_seconds: u64,
) {
    loop {
        info!("Refreshing calendar cache...");

        match fetch_and_merge_calendars(&feeds, &rules).await {
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

async fn serve_index() -> Html<String> {
    match fs::read_to_string("index.html") {
        Ok(content) => Html(content),
        Err(_) => Html("<html><body><h1>Error loading index.html</h1></body></html>".to_string()),
    }
}

async fn fetch_and_merge_calendars(
    feeds: &[CalendarFeed],
    rules: &[Rule],
) -> Result<String, Box<dyn std::error::Error>> {
    // Fetch all calendars concurrently
    let client = reqwest::Client::new();
    let mut fetch_tasks = Vec::new();

    for feed in feeds {
        let client = client.clone();
        let feed = feed.clone();
        let task = tokio::spawn(async move {
            let response = client.get(&feed.url).send().await?;
            let text = response.text().await?;
            Ok::<(String, String), reqwest::Error>((text, feed.id))
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
            Ok(Ok((ical_text, calendar_id))) => {
                if let Err(e) =
                    merge_calendar_events(&ical_text, &mut merged_calendar, rules, &calendar_id)
                {
                    tracing::warn!("Failed to parse calendar '{}': {}", calendar_id, e);
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

fn merge_calendar_events(
    ical_text: &str,
    merged: &mut Calendar,
    rules: &[Rule],
    calendar_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the iCal text using the parser
    let parsed: Calendar = ical_text.parse()?;

    // Extract events and add them to the merged calendar
    for component in parsed.components {
        if let CalendarComponent::Event(mut event) = component {
            event.add_property("X-CALENDAR-SOURCE", calendar_id);
            for rule in rules {
                let mut conditions_met = true;
                for condition in &rule.conditions {
                    let field_value = event.property_value(&condition.field).unwrap_or("");
                    match condition.op {
                        ConditionOp::Contains => {
                            if !field_value.contains(&condition.value) {
                                conditions_met = false;
                                break;
                            }
                        }
                    }
                }
                if conditions_met {
                    info!(
                        "Applying rule '{}' to event '{}'",
                        rule.name,
                        event.get_summary().unwrap_or("Unnamed Event")
                    );
                    for action in &rule.actions {
                        match action.op {
                            ActionOp::Set => {
                                event.add_property(&action.field, &action.value);
                            }
                            ActionOp::Prepend => {
                                let current_value =
                                    event.property_value(&action.field).unwrap_or("");
                                let new_value = format!("{}{}", action.value, current_value);
                                event.add_property(&action.field, &new_value);
                            }
                        }
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
