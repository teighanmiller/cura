# cura

> **Note: This project is actively under development. Features, APIs, and commands may change.**

A personal productivity CLI built in Rust that integrates with Google Calendar and web search, designed to bring common workflows into the terminal without context-switching to a browser.

## Motivation

`cura` is the tool layer for the [cura_agent](https://github.com/teighanmiller/cura_agent) project — an AI agent that uses this CLI as its interface to external services, similar in spirit to how an MCP server exposes capabilities to a model. Rather than giving the agent direct API access, `cura` acts as a structured abstraction: each subcommand is a discrete, testable operation the agent can invoke by name with well-defined arguments.

It also works as a standalone productivity CLI for day-to-day terminal use — checking your calendar, creating events, and searching the web without leaving the terminal.

## Features

- **Google Calendar** — list events for today or a custom time range, look up event details by name, create all-day, timed, or recurring events, and delete events
- **Web search** — query the web from the terminal via DuckDuckGo and get formatted results inline
- **Time** — fetch the current local time, date, or datetime

## Planned features

- **Multi-provider search** — pluggable search engine backend with support for additional providers beyond DuckDuckGo
- **Search result re-ranking** — relevance scoring to surface higher-quality results for agent consumption
- **Event editing** — update existing event details (title, time, description)
- **Multi-calendar support** — configurable calendar targeting rather than defaulting to the primary calendar
- **Google Tasks integration** — task creation, listing, and completion via the Google Tasks API
- **Gmail integration** — read and send email to expand the agent's communication capabilities
- **Filesystem access** — file retrieval and organization operations to support context-fetching workflows in the agent
- **Configurable output formats** — structured output modes (JSON, table, plain text) to better suit agent parsing vs. human reading

## Tech stack

| Dependency | Purpose |
|---|---|
| `clap` | CLI argument parsing with derive macros |
| `google-calendar3` | Google Calendar API client |
| `yup-oauth2` | Google OAuth 2.0 authentication |
| `tokio` | Async runtime |
| `chrono` | Date and time parsing |
| `websearch` | DuckDuckGo web search |
| `anyhow` | Ergonomic error handling |

## Prerequisites

- Rust (stable toolchain)
- A Google Cloud project with the Calendar API enabled and OAuth 2.0 credentials

## Setup

### 1. Google Cloud credentials

1. Go to the [Google Cloud Console](https://console.cloud.google.com/) and create or select a project.
2. Enable the **Google Calendar API**.
3. Create **OAuth 2.0 credentials** (Desktop application type) and download the JSON file.
4. Rename the file to `client_secret.json` and place it in the project root.

On first run, `cura` opens a browser window for you to authorize access. The resulting token is cached in `token_cache.json` so subsequent runs don't require re-authorization.

### 2. Build

```bash
cargo build --release
```

The binary is placed at `target/release/cura`. Optionally, add it to your `$PATH`:

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
cura gcal new-event --name "Team standup" --description "Daily sync" --date 2026-04-24 --start-time "2026-04-24 09:00:00 -0700" --end-time "2026-04-24 09:30:00 -0700"
```

**Create a recurring event:**
```bash
cura gcal new-event --name "Weekly Standup" --description "Team sync" --date 2026-05-19 --start-time "2026-05-19 09:00:00 -0700" --end-time "2026-05-19 09:30:00 -0700" --freq weekly
```

Supported `--freq` values: `daily`, `weekly`, `monthly`, `yearly`.

**Delete an event:**
```bash
cura gcal delete-event --name "Weekly Standup"
```

If the name matches more than one event, narrow the search with `--start-time` and `--end-time`:
```bash
cura gcal delete-event --name "Weekly Standup" --start-time "2026-05-19 00:00:00 -0700" --end-time "2026-05-19 23:59:59 -0700"
```

> **Note:** Deleting a recurring event by name removes the entire series.

#### Date and time formats

| Field | Format | Example |
|---|---|---|
| `--date` | `YYYY-MM-DD` | `2026-04-24` |
| `--start-time` / `--end-time` | `YYYY-MM-DD HH:MM:SS ±HHMM` | `2026-04-24 09:00:00 -0700` |

### Time

**Get the current time:**
```bash
cura time time
```

**Get the current date:**
```bash
cura time date
```

**Get the current date and time:**
```bash
cura time date-time
```

All time commands return local time. Output formats:

| Subcommand | Format | Example |
|---|---|---|
| `time` | `HH:MM:SS` | `14:30:00` |
| `date` | `YYYY-MM-DD` | `2026-05-15` |
| `date-time` | `YYYY-MM-DD HH:MM:SS` | `2026-05-15 14:30:00` |

### Web search

**Basic search:**
```bash
cura web "your search query"
```

**Limit results:**
```bash
cura web "your search query" --max-value 5
```

**Specify a search engine** (DuckDuckGo is the default):
```bash
cura web "your search query" --engine duck-duck-go
```

## Project structure

```
src/
  main.rs       — CLI entry point, argument parsing, and subcommand dispatch
  auth.rs       — Google OAuth2 authentication and token caching
  google_cal.rs — Google Calendar API commands and response formatting
  web.rs        — Web search integration and output formatting
  time.rs       — Time subcommands (current time/date/datetime) and date/time parsing utilities
```

## Running tests

```bash
cargo test
```

## CI

GitHub Actions runs `cargo build` and `cargo test` on every push and pull request.

## License

MIT
