use crate::auth;
use crate::time::{convert_date, convert_datetime, get_current_timezone, get_period};
use chrono::{NaiveDate, NaiveDateTime};
use clap::error::Result;
use clap::{Args, Subcommand, ValueEnum};
use google_calendar3::Error;
use google_calendar3::api::{Event, EventDateTime, Events};
use http::Response;
use http_body_util::combinators::BoxBody;

#[derive(Subcommand, Clone)]
pub enum GcalCommands {
    /// List upcoming calendar events
    EventList {
        /// Start of the time range (e.g. "2024-01-15 09:00")
        #[arg(short, long)]
        start_time: Option<String>,
        /// End of the time range (e.g. "2024-01-15 17:00")
        #[arg(short, long)]
        end_time: Option<String>,
    },
    /// Get details for a specific event by name
    EventDetails {
        /// Name or keyword to search for
        #[arg(short, long)]
        name: String,
        /// Start of the search window (e.g. "2024-01-15 09:00")
        #[arg(short, long)]
        start_time: Option<String>,
        /// End of the search window (e.g. "2024-01-15 17:00")
        #[arg(short, long)]
        end_time: Option<String>,
    },
    /// Create a new calendar event
    NewEvent {
        /// Event title
        #[arg(short, long)]
        name: String,
        /// Event description
        #[arg(short, long)]
        description: Option<String>,
        /// Event date (e.g. "2024-01-15")
        #[arg(long)]
        date: Option<String>,
        /// Start time (e.g. "2024-01-15 09:00")
        #[arg(short, long)]
        start_time: Option<String>,
        /// End time (e.g. "2024-01-15 10:00")
        #[arg(short, long)]
        end_time: Option<String>,
        /// Recurrence frequency
        #[arg(short, long)]
        freq: Option<SeriesArgs>,
    },
    /// Delete a calendar event
    DeleteEvent {
        /// Event title
        #[arg(short, long)]
        name: String,
        /// Start time (e.g. "2024-01-15 09:00")
        #[arg(short, long)]
        start_time: Option<String>,
        /// End time (e.g. "2024-01-15 10:00")
        #[arg(short, long)]
        end_time: Option<String>,
    },
}

#[derive(ValueEnum, Clone)]
pub enum SeriesArgs {
    /// Repeat every week
    Weekly,
    /// Repeat every month
    Monthly,
    /// Repeat every day
    Daily,
    /// Repeat every year
    Yearly,
}

/// Google Calendar commands
#[derive(Args, Clone)]
pub struct GcalArgs {
    #[command(subcommand)]
    command: GcalCommands,
}

type CalendarListEntryResponse = Result<
    (
        Response<BoxBody<google_calendar3::hyper::body::Bytes, google_calendar3::hyper::Error>>,
        google_calendar3::api::Event,
    ),
    google_calendar3::Error,
>;

type StringOutput = Result<String, Box<Error>>;

struct EventQuery {
    name: String,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
}

struct CalendarEvent {
    name: String,
    description: String,
    date: NaiveDate,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
    freq: Option<SeriesArgs>,
}

fn event_to_string(event: Event) -> StringOutput {
    Ok(format!("Event: {}\n", event.summary.unwrap_or_default()))
}

fn events_to_string(events: Events) -> StringOutput {
    let mut output = String::new();
    let events = events.items.unwrap_or_default();
    if events.is_empty() {
        println!("No events found in calendar.")
    }
    for event in events {
        let formatted = format!("Event: {}\n", event.summary.unwrap_or_default());
        output.push_str(&formatted);
    }
    Ok(output)
}

async fn get_event(
    hub: auth::Hub,
    name: String,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Event, anyhow::Error> {
    match _get_event(hub, name, start_time, end_time).await {
        Ok(events) => {
            if events.items.is_none() {
                Err(anyhow::anyhow!("No events found in calendar.".to_string()))
            } else if events.items.iter().len() > 1 {
                Err(anyhow::anyhow!(
                    "The following events were found: {:?}, please provide more specific search parameters so only one event is found",
                    events.items
                ))
            } else {
                Ok(events.items.clone().unwrap().first().unwrap().clone())
            }
        }
        Err(e) => Err(e),
    }
}

async fn _get_event(
    hub: auth::Hub,
    name: String,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Events, anyhow::Error> {
    let event_query = EventQuery {
        name,
        start_time: start_time.map(|t| convert_datetime(&t)).transpose()?,
        end_time: end_time.map(|t| convert_datetime(&t)).transpose()?,
    };

    if let (Some(start), Some(end)) = (event_query.start_time, event_query.end_time) {
        let (_response, events) = hub
            .events()
            .list("primary")
            .time_min(start.and_utc())
            .time_max(end.and_utc())
            .q(event_query.name.as_str())
            .doit()
            .await?;
        // Ok(event_to_string(events))
        Ok(events)
    } else {
        let (_response, events) = hub
            .events()
            .list("primary")
            .q(event_query.name.as_str())
            .doit()
            .await?;
        Ok(events)
    }
}

async fn get_events(
    hub: auth::Hub,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<StringOutput, anyhow::Error> {
    let mut period = get_period(&[]);
    if let (Some(start), Some(end)) = (start_time, end_time) {
        period = get_period(&[start, end]);
    }
    let (_response, events) = hub
        .events()
        .list("primary")
        .time_min(period.min_time) // Start of today
        .time_max(period.max_time) // End of today
        .doit()
        .await?;

    Ok(events_to_string(events))
}

async fn get_event_details(
    hub: auth::Hub,
    name: String,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<StringOutput, anyhow::Error> {
    let event_result = get_event(hub, name, start_time, end_time).await;
    match event_result {
        Ok(event) => Ok(event_to_string(event)),
        Err(e) => Err(e),
    }
}

async fn insert_event(hub: auth::Hub, event: Event, cal_id: &str) -> CalendarListEntryResponse {
    hub.events().insert(event, cal_id).doit().await
}

fn get_freq_rule(freq: SeriesArgs) -> String {
    match freq {
        SeriesArgs::Daily => "RRULE:FREQ=DAILY".to_string(),
        SeriesArgs::Weekly => "RRULE:FREQ=WEEKLY".to_string(),
        SeriesArgs::Monthly => "RRULE:FREQ=MONTHLY".to_string(),
        SeriesArgs::Yearly => "RRULE:FREQ=YEARLY".to_string(),
    }
}

fn create_event(event_details: CalendarEvent) -> Event {
    let mut event = Event {
        summary: Some(event_details.name),
        description: Some(event_details.description),
        recurrence: match event_details.freq {
            Some(rec) => Some(vec![get_freq_rule(rec)]),
            _ => None,
        },
        ..Default::default()
    };

    if let (Some(start), Some(end)) = (event_details.start_time, event_details.end_time) {
        event.start = Some(EventDateTime {
            date: None,
            date_time: Some(start.and_utc()),
            time_zone: Some(get_current_timezone()),
        });
        event.end = Some(EventDateTime {
            date: None,
            date_time: Some(end.and_utc()),
            time_zone: Some(get_current_timezone()),
        });
    } else {
        event.start = Some(EventDateTime {
            date: Some(event_details.date),
            date_time: None,
            time_zone: None,
        });
        event.end = Some(EventDateTime {
            date: Some(event_details.date),
            date_time: None,
            time_zone: None,
        });
    }
    event
}

async fn delete_event(
    hub: auth::Hub,
    name: String,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<StringOutput, anyhow::Error> {
    let event = get_event(hub.clone(), name.clone(), start_time, end_time).await?;

    let results = hub
        .events()
        .delete("primary", event.id.unwrap().as_str())
        .send_updates("all")
        .send_notifications(true)
        .doit()
        .await;

    match results {
        Ok(_result) => Ok(Ok(
            format!("Successfully deleted event {}", name).to_string()
        )),
        Err(e) => Err(e.into()),
    }
}

async fn insert_new_event(
    hub: auth::Hub,
    name: String,
    description: Option<String>,
    date: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    freq: Option<SeriesArgs>,
) -> Result<StringOutput, Error> {
    let cal_event = CalendarEvent {
        name,
        description: description.unwrap(),
        date: date.map(|d| convert_date(&d).unwrap()).unwrap(),
        start_time: start_time.map(|t| convert_datetime(&t).unwrap()),
        end_time: end_time.map(|t| convert_datetime(&t).unwrap()),
        freq,
    };
    let event = create_event(cal_event);
    let result = insert_event(hub, event, "primary").await;
    match result {
        Ok(_success) => Ok(Ok("Event added successfully!".to_string())),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn create_event_date_only() {
        let cal = CalendarEvent {
            name: "Meeting".to_string(),
            description: "Team sync".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            start_time: None,
            end_time: None,
            freq: None,
        };
        let event = create_event(cal);
        assert_eq!(event.summary.unwrap(), "Meeting");
        assert_eq!(event.description.unwrap(), "Team sync");
        assert!(event.recurrence.is_none());
        assert!(event.start.as_ref().unwrap().date_time.is_none());
    }

    #[test]
    fn create_event_with_recurrence() {
        let cal = CalendarEvent {
            name: "Standup".to_string(),
            description: "Daily".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            start_time: None,
            end_time: None,
            freq: Some(SeriesArgs::Daily),
        };
        let event = create_event(cal);
        let rec = event.recurrence.unwrap();
        assert_eq!(rec, vec!["RRULE:FREQ=DAILY"]);
    }
}

pub async fn get_calendar_service(args: GcalArgs) -> Result<String, anyhow::Error> {
    let hub = auth::login().await;

    // All calls return a StringOutput Type
    match args.command {
        GcalCommands::EventList {
            start_time,
            end_time,
        } => get_events(hub, start_time, end_time)
            .await?
            .map_err(|e| anyhow::anyhow!("{e}")),
        GcalCommands::EventDetails {
            name,
            start_time,
            end_time,
        } => get_event_details(hub, name, start_time, end_time)
            .await?
            .map_err(|e| anyhow::anyhow!("{e}")),
        GcalCommands::NewEvent {
            name,
            description,
            date,
            start_time,
            end_time,
            freq,
        } => insert_new_event(hub, name, description, date, start_time, end_time, freq)
            .await?
            .map_err(|e| anyhow::anyhow!("{e}")),
        GcalCommands::DeleteEvent {
            name,
            start_time,
            end_time,
        } => delete_event(hub, name, start_time, end_time)
            .await?
            .map_err(|e| anyhow::anyhow!("{e}")),
    }
}
