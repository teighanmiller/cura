use crate::auth;
use crate::time::{convert_date, convert_datetime, convert_time, get_current_timezone, get_period};
use chrono::{NaiveDate, NaiveDateTime};
use clap::ValueEnum;
use google_calendar3::Error;
use google_calendar3::api::{Event, EventDateTime, Events};
use http::Response;
use http_body_util::combinators::BoxBody;

// static EVENT_LIST: &[&str] = &["event_list", "event_details", "new_event", "series"];

#[derive(ValueEnum, Clone)]
pub enum GcalCommands {
    EventList,
    EventDetails,
    NewEvent,
}

#[derive(ValueEnum, Clone)]
pub enum SeriesArgs {
    Weekly,
    Monthly,
    Daily,
    Yearly,
}

type CalendarListEntryResponse = Result<
    (
        Response<BoxBody<google_calendar3::hyper::body::Bytes, google_calendar3::hyper::Error>>,
        google_calendar3::api::Event,
    ),
    google_calendar3::Error,
>;

type StringOutput = Result<String, Error>;

struct EventQuery {
    name: String,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
}

struct CalenderEvent {
    name: String,
    description: String,
    date: NaiveDate,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
    freq: Option<SeriesArgs>,
}

impl From<&Vec<String>> for CalenderEvent {
    fn from(args: &Vec<String>) -> Self {
        let date = convert_date(&args[2]).unwrap();
        let start_time = args.get(3).map(|t| date.and_time(convert_time(t).unwrap()));
        let end_time = args.get(4).map(|t| date.and_time(convert_time(t).unwrap()));
        let freq = None;
        CalenderEvent {
            name: args[0].clone(),
            description: args[1].clone(),
            date,
            start_time,
            end_time,
            freq,
        }
    }
}

fn event_to_string(events: Events) -> StringOutput {
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
    name: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<StringOutput, Error> {
    let event_query = EventQuery {
        name: name
            .ok_or("Event name was 'None', please provide an event name.")
            .unwrap(),
        start_time: start_time.map(|t| convert_datetime(&t).unwrap()),
        end_time: end_time.map(|t| convert_datetime(&t).unwrap()),
    };

    if event_query.start_time.is_none() | event_query.end_time.is_none() {
        let (_response, events) = hub
            .events()
            .list("primary")
            .q(event_query.name.as_str())
            .doit()
            .await
            .unwrap();
        Ok(event_to_string(events))
    } else {
        let (_response, events) = hub
            .events()
            .list("primary")
            .time_min(event_query.start_time.unwrap().and_utc())
            .time_max(event_query.start_time.unwrap().and_utc())
            .q(event_query.name.as_str())
            .doit()
            .await
            .unwrap();
        Ok(event_to_string(events))
    }
}

async fn get_events(
    hub: auth::Hub,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<StringOutput, Error> {
    let mut period = get_period(&vec![]);
    if !start_time.is_none() & !end_time.is_none() {
        period = get_period(&vec![start_time.unwrap(), end_time.unwrap()]);
    }
    let (_response, events) = hub
        .events()
        .list("primary")
        .time_min(period.min_time) // Start of today
        .time_max(period.max_time) // End of today
        .doit()
        .await?;

    Ok(event_to_string(events))
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

fn create_event(event_details: CalenderEvent) -> Event {
    let mut req = Event::default();
    req.summary = Some(event_details.name);
    req.description = Some(event_details.description);

    if !event_details.freq.is_none() {
        req.recurrence = Some(vec![get_freq_rule(event_details.freq.unwrap())])
    }

    if event_details.start_time.is_none() | event_details.end_time.is_none() {
        req.start = Some(EventDateTime {
            date: Some(event_details.date),
            date_time: None,
            time_zone: None,
        });
        req.end = Some(EventDateTime {
            date: Some(event_details.date),
            date_time: None,
            time_zone: None,
        });
    } else {
        let naive_st = event_details.start_time;
        let naive_et = event_details.end_time;
        req.start = Some(EventDateTime {
            date: None,
            date_time: naive_st.map(|naive| naive.and_utc()),
            time_zone: Some(get_current_timezone()),
        });
        req.end = Some(EventDateTime {
            date: None,
            date_time: naive_et.map(|naive| naive.and_utc()),
            time_zone: Some(get_current_timezone()),
        });
    }
    req
}

async fn insert_new_event(
    hub: auth::Hub,
    name: Option<String>,
    description: Option<String>,
    date: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    freq: Option<SeriesArgs>,
) -> Result<StringOutput, Error> {
    let cal_event = CalenderEvent {
        name: name.unwrap(),
        description: description.map(|d| d).unwrap(),
        date: date.map(|d| convert_date(&d).unwrap()).unwrap(),
        start_time: start_time.map(|t| convert_datetime(&t).unwrap()),
        end_time: end_time.map(|t| convert_datetime(&t).unwrap()),
        freq: freq,
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
    use google_calendar3::api::{Event, Events};

    fn make_event(summary: &str) -> Event {
        let mut e = Event::default();
        e.summary = Some(summary.to_string());
        e
    }

    #[test]
    fn event_to_string_empty() {
        let events = Events {
            items: None,
            ..Default::default()
        };
        let result = event_to_string(events).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn event_to_string_single_event() {
        let events = Events {
            items: Some(vec![make_event("Team standup")]),
            ..Default::default()
        };
        let result = event_to_string(events).unwrap();
        assert!(result.contains("Team standup"));
    }

    #[test]
    fn event_to_string_multiple_events() {
        let events = Events {
            items: Some(vec![make_event("Meeting A"), make_event("Meeting B")]),
            ..Default::default()
        };
        let result = event_to_string(events).unwrap();
        assert!(result.contains("Meeting A"));
        assert!(result.contains("Meeting B"));
    }

    #[test]
    fn calendar_event_from_vec() {
        let args = vec![
            "Team sync".to_string(),
            "Weekly team sync".to_string(),
            "2026-04-18".to_string(),
            "09:00:00".to_string(),
            "10:00:00".to_string(),
        ];
        let cal_event = CalenderEvent::from(&args);
        assert_eq!(cal_event.name, "Team sync");
        assert_eq!(cal_event.description, "Weekly team sync");
        assert!(cal_event.start_time.is_some());
        assert!(cal_event.end_time.is_some());
    }

    #[test]
    fn create_event_with_times_sets_datetime() {
        let args = vec![
            "My Event".to_string(),
            "A description".to_string(),
            "2026-04-18".to_string(),
            "09:00:00".to_string(),
            "10:00:00".to_string(),
        ];
        let cal_event = CalenderEvent::from(&args);
        let event = create_event(cal_event);
        let start = event.start.unwrap();
        // When times are present, date_time should be set, not date
        assert!(start.date_time.is_some());
        assert!(start.date.is_none());
    }

    #[test]
    fn create_event_summary_and_description() {
        let args = vec![
            "Stand-up".to_string(),
            "Daily stand-up".to_string(),
            "2026-04-18".to_string(),
            "09:00:00".to_string(),
            "09:15:00".to_string(),
        ];
        let cal_event = CalenderEvent::from(&args);
        let event = create_event(cal_event);
        assert_eq!(event.summary.unwrap(), "Stand-up");
        assert_eq!(event.description.unwrap(), "Daily stand-up");
    }
}

pub async fn get_calendar_service(
    command: GcalCommands,
    name: Option<String>,
    description: Option<String>,
    date: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    freq: Option<SeriesArgs>,
) -> Result<String, Error> {
    let hub = auth::login().await;

    // All calls return a StringOutput Type
    match command {
        GcalCommands::EventList => get_events(hub, start_time, end_time).await?,
        GcalCommands::EventDetails => get_event(hub, name, start_time, end_time).await?,
        GcalCommands::NewEvent => {
            insert_new_event(hub, name, description, date, start_time, end_time, freq).await?
        }
    }
}
