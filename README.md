# iCal Aggregator

A web service built with Axum that merges multiple iCal feeds into a single aggregated calendar feed.

## Features

- Fetches multiple iCal feeds concurrently
- Merges all events into a single calendar
- **Caching**: Calendars are refreshed in the background at configurable intervals
- Exposes the merged calendar as an iCal/ICS endpoint
- Configurable feed URLs and refresh interval via `config.toml`

## Configuration

Edit `config.toml` to add your iCal feed URLs:

```toml
# Server port (default: 3000)
port = 3000

# Refresh interval in seconds (default: 300 = 5 minutes)
# How often to fetch and merge the calendar feeds
refresh_interval_seconds = 300

feeds = [
    "https://calendar.google.com/calendar/ical/your-calendar/basic.ics",
    "https://example.com/events.ics",
]
```

## How Caching Works

The service starts a background task that:
1. Fetches all configured iCal feeds concurrently
2. Merges them into a single calendar
3. Stores the result in memory
4. Repeats this process every `refresh_interval_seconds`

When you request `/calendar.ics`, you get the cached merged calendar instantly without waiting for feeds to be fetched. This provides:
- Fast response times
- Reduced load on upstream calendar servers
- Consistent availability even if upstream feeds are temporarily unavailable

## Running

```bash
cargo run
```

The server will start on `http://0.0.0.0:3000` (or the port specified in config).

## Usage

Access the merged calendar at:
```
http://localhost:3000/calendar.ics
```

You can subscribe to this URL in your calendar application (Google Calendar, Apple Calendar, Outlook, etc.).

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/ical-aggregator`.
