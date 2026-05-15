---
description: Context and guidance for the Google Calendar CLI integration test workflow
---

## Integration test workflow overview

This project has a GitHub Actions workflow at `.github/workflows/integration-test.yml` that runs on every pull request to `main`. It builds the release binary and runs end-to-end tests against the real Google Calendar API using a service account.

### Authentication

`src/auth.rs` supports two auth modes selected at runtime:

- **CI (service account)** — when `GOOGLE_SERVICE_ACCOUNT_KEY` env var is set, `login()` parses it as a `ServiceAccountKey` JSON string and uses `ServiceAccountAuthenticator`. The service account operates on its own primary calendar.
- **Local (installed flow)** — when the env var is absent, falls back to `InstalledFlowAuthenticator` with `client_secret.json` and token cache at `token_cache.json`.

To run locally with service account auth: `GOOGLE_SERVICE_ACCOUNT_KEY=$(cat your-key.json) cura gcal ...`

To activate CI: add the contents of the service account JSON key file as a GitHub secret named `GOOGLE_SERVICE_ACCOUNT_KEY` (Settings → Secrets → Actions).

### Test scenarios

Each scenario is a separate GitHub Actions step so failures are visible individually.

| Step | Commands exercised |
|---|---|
| All-day event (today) | `new-event --date`, `event-list` (default window), `event-details`, `delete-event` |
| All-day event (specific date) | `new-event --date`, `event-list --start-time --end-time`, `event-details` with range, `delete-event` |
| Timed event | `new-event --start-time --end-time`, `event-list`, `event-details`, `delete-event` |
| Weekly recurring series | `new-event --freq weekly`, `event-list`, `delete-event` |
| Monthly recurring series | `new-event --freq monthly`, `event-list`, `delete-event` |

### Conventions

- Event names follow `ci-{run_id}-{scenario}` to avoid collisions between concurrent runs.
- Every step has a `trap cleanup EXIT` that deletes the test event even if an assertion fails mid-step.
- Timestamps passed to `--start-time` and `--end-time` must use the format `"YYYY-MM-DD HH:MM:SS +0000"` — this is what `convert_datetime` in `src/time.rs` expects.
- The workflow uses long flags (`--name`, `--date`, etc.) throughout to avoid a known short-flag conflict between `--description` and `--date` in the `new-event` subcommand.

### Adding a new test

Add a new step to `.github/workflows/integration-test.yml` following this pattern:

```yaml
- name: "Test: <scenario name>"
  run: |
    set -euo pipefail
    NAME="ci-${{ github.run_id }}-<unique-suffix>"

    cleanup() { $BIN gcal delete-event --name "$NAME" 2>/dev/null || true; }
    trap cleanup EXIT

    # create
    $BIN gcal new-event --name "$NAME" --description "CI test" --date "$TODAY"

    # assert exists
    if ! $BIN gcal event-list | grep -q "$NAME"; then
      echo "FAIL: ..."
      exit 1
    fi

    # delete
    $BIN gcal delete-event --name "$NAME"

    # assert gone
    if $BIN gcal event-list | grep -q "$NAME"; then
      echo "FAIL: ..."
      exit 1
    fi
```
