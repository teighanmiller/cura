# cura

A command-line tool for interacting with Google Calendar and performing web searches, written in Rust.

## Features

- **Google Calendar** — list events, look up event details, and create new events
- **Web search** — search the web from the terminal via DuckDuckGo

## Prerequisites

- Rust (stable)
- A Google Cloud project with the Calendar API enabled and OAuth 2.0 credentials

## Setup

### Google Calendar credentials

1. Go to the [Google Cloud Console](https://console.cloud.google.com/) and create a project.
2. Enable the **Google Calendar API**.
3. Create **OAuth 2.0 credentials** (Desktop application type) and download the JSON file.
4. Rename the downloaded file to `client_secret.json` and place it in the project root.

On first run, cura will open a browser window for you to authorize access. The token is cached in `token_cache.json` for subsequent runs.

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/cura`. Optionally, copy it somewhere on your `$PATH`:

```bash
cp target/release/cura /usr/local/bin/
```

## Usage

### Google Calendar

**List today's events:**
```bash
cura gcal event-list
```

**List events in a time range:**
```bash
cura gcal event-list --start-time "2026-04-24 09:00:00 +0000" --end-time "2026-04-24 17:00:00 +0000"
```

**Look up an event by name:**
```bash
cura gcal event-details --name "Team standup"
```

**Create an all-day event:**
```bash
cura gcal new-event --name "Conference" --description "Annual company conf" --date 2026-05-01
```

**Create a timed event:**
```bash
cura gcal new-event --name "Team standup" --description "Daily sync" --date 2026-04-24 --start-time "09:00:00" --end-time "09:30:00"
```

#### Date and time formats

| Field | Format | Example |
|---|---|---|
| `--date` | `YYYY-MM-DD` | `2026-04-24` |
| `--start-time` / `--end-time` (event creation) | `HH:MM:SS` | `09:00:00` |
| `--start-time` / `--end-time` (event lookup) | `YYYY-MM-DD HH:MM:SS ±HHMM` | `2026-04-24 09:00:00 +0000` |

### Web search

```bash
cura web "your search query"
```

**Limit the number of results:**
```bash
cura web "your search query" --max-value 5
```

**Specify a search engine** (DuckDuckGo is the default and currently the only option):
```bash
cura web "your search query" --engine duck-duck-go
```

## Running tests

```bash
cargo test
```

## Project structure

```
src/
  main.rs       — CLI argument parsing and subcommand dispatch
  auth.rs       — Google OAuth2 authentication
  google_cal.rs — Google Calendar API commands and logic
  web.rs        — Web search integration
  time.rs       — Date/time parsing and period utilities
```
