# cura

A Rust CLI for interacting with Google Calendar and the web. Built to serve as the tool layer for [cura_agent](https://github.com/teighanmiller/cura_agent), an AI agent that can manage your calendar and look things up on your behalf — though it works perfectly well as a standalone CLI.

> **Note:** `cura_agent` and 'cura' are currently in progress.

---

## Features

- List today's events or events within a custom time range
- Look up a specific event by name
- Create single or recurring calendar events
- Search the web via DuckDuckGo

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- A Google Cloud project with the **Google Calendar API** enabled
- OAuth 2.0 credentials downloaded as `client_secret.json`

### Setting up Google OAuth credentials

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or select an existing one)
3. Enable the **Google Calendar API** under *APIs & Services > Library*
4. Go to *APIs & Services > Credentials* and create an **OAuth 2.0 Client ID**
   - Application type: **Desktop app**
5. Download the credentials and save the file as `client_secret.json` in the project root

`client_secret.json` is gitignored and should never be committed.

---

## Installation

```bash
git clone https://github.com/teighanmiller/cura.git
cd cura
cargo build --release
```

Place `client_secret.json` in the project root before running.

On first run, cura will open a browser window asking you to authorize access to your Google Calendar. After authorizing, a `token_cache.json` file is written locally so you won't need to re-authenticate on subsequent runs.

---

## Usage

```
cura <COMMAND>
```

### `gcal` — Google Calendar

```
cura gcal <COMMAND> [NAME] [DESCRIPTION] [DATE] [START_TIME] [END_TIME] [FREQ]
```

| Argument | Format | Required |
|---|---|---|
| `DATE` | `YYYY-MM-DD` | For `new-event` |
| `START_TIME` / `END_TIME` | `YYYY-MM-DD HH:MM:SS +OFFSET` | Optional |
| `FREQ` | `daily`, `weekly`, `monthly`, `yearly` | Optional |

#### List today's events

```bash
cura gcal event-list
```

#### List events in a time range

```bash
cura gcal event-list _ _ _ "2026-05-11 09:00:00 +0000" "2026-05-11 17:00:00 +0000"
```

#### Look up an event by name

```bash
cura gcal event-details "Team standup"
```

#### Create a single event

```bash
cura gcal new-event "Team standup" "Weekly sync" 2026-05-12 "2026-05-12 09:00:00 +0000" "2026-05-12 09:30:00 +0000"
```

#### Create a recurring event

```bash
cura gcal new-event "Team standup" "Weekly sync" 2026-05-12 "2026-05-12 09:00:00 +0000" "2026-05-12 09:30:00 +0000" weekly
```

#### Create an all-day event

Omit `START_TIME` and `END_TIME`:

```bash
cura gcal new-event "Conference" "Annual dev conference" 2026-06-01
```

---

### `web` — Web Search

```
cura web [OPTIONS] <QUERY>
```

| Option | Description |
|---|---|
| `-e, --engine` | Search engine (`duck-duck-go`). Defaults to DuckDuckGo. |
| `--max-value` | Maximum number of results to return |

#### Examples

```bash
cura web "rust async runtimes"
cura web "rust async runtimes" --engine duck-duck-go --max-value 5
```

---

## Planned Improvements

- **Multiple calendar support** — target calendars other than `primary` by name or ID
- **Richer recurrence rules** — specify end dates, intervals (e.g. every 2 weeks), and specific days of the week for recurring events
- **Improved web search output** — formatted results with titles, URLs, and snippets instead of raw debug output; support for additional search providers
- **Event deletion and editing** — update or remove existing calendar events from the CLI
- **Structured output** — JSON output mode for piping results into other tools or agents
- **Interactive auth flow improvements** — better handling of expired tokens and re-authentication

---

## Project Structure

```
src/
  main.rs       — CLI argument parsing and dispatch
  auth.rs       — Google OAuth flow and hub construction
  google_cal.rs — Calendar API operations
  time.rs       — Date/time parsing and period helpers
  web.rs        — Web search
```

---

## License

MIT
