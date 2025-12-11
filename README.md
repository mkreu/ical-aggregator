# iCal Aggregator

A web service built with Axum that merges multiple iCal feeds into a single aggregated calendar feed.

## Features

- Fetches multiple iCal feeds concurrently
- Merges all events into a single calendar
- Exposes the merged calendar as an iCal/ICS endpoint
- Configurable feed URLs via `config.toml`

## Configuration

Edit `config.toml` to add your iCal feed URLs:

```toml
port = 3000

feeds = [
    "https://calendar.google.com/calendar/ical/your-calendar/basic.ics",
    "https://example.com/events.ics",
]
```

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
