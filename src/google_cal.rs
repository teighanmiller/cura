use crate::auth;
use crate::time::{convert_date, convert_datetime, convert_time, get_current_timezone, get_period};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};
use google_calendar3::Error;
use google_calendar3::api::{Event, EventDateTime, Events};
use http::Response;
use http_body_util::combinators::BoxBody;

static EVENT_LIST: &[&str] = &["event_list", "event_details", "new_event", "series"];

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

impl From<&Vec<String>> for EventQuery {
    fn from(v: &Vec<String>) -> Self {
        let mut iter = v.iter();
        EventQuery {
            name: iter.next().unwrap().to_string(),
            start_time: iter.next().map(|t| convert_datetime(t).unwrap()),
            end_time: iter.next().map(|t| convert_datetime(t).unwrap()),
        }
    }
}

struct CalenderEvent {
    name: String,
    description: String,
    date: NaiveDate,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
}

impl From<&Vec<String>> for CalenderEvent {
    fn from(v: &Vec<String>) -> Self {
        let mut iter = v.iter();
        CalenderEvent {
            name: iter.next().unwrap().to_string(),
            description: iter.next().unwrap().to_string(),
            date: convert_date(iter.next().unwrap()).unwrap(),
            start_time: iter.next().map(|t| convert_time(t).unwrap()),
            end_time: iter.next().map(|t| convert_time(t).unwrap()),
        }
    }
}

fn event_to_string(events: Events) -> StringOutput {
    let mut output = String::new();
    let events = events.items.unwrap_or_default();
    for event in events {
        let formatted = format!("Event: {}\n", event.summary.unwrap_or_default());
        output.push_str(&formatted);
    }
    Ok(output)
}

async fn get_event(hub: auth::Hub, event_details: &Vec<String>) -> StringOutput {
    let details = EventQuery::from(event_details);
    if details.start_time.is_none() | details.end_time.is_none() {
        let (_response, events) = hub
            .events()
            .list("primary")
            .q(details.name.as_str())
            .doit()
            .await
            .unwrap();
        event_to_string(events)
    } else {
        let (_response, events) = hub
            .events()
            .list("primary")
            .time_min(details.start_time.unwrap().and_utc())
            .time_max(details.start_time.unwrap().and_utc())
            .q(details.name.as_str())
            .doit()
            .await
            .unwrap();
        event_to_string(events)
    }
}

async fn get_events(hub: auth::Hub, arguments: &Vec<String>) -> StringOutput {
    let period = get_period(arguments);
    let (_response, events) = hub
        .events()
        .list("primary")
        .time_min(period.min_time) // Start of today
        .time_max(period.max_time) // End of today
        .doit()
        .await?;

    event_to_string(events)
}

async fn insert_event(hub: auth::Hub, event: Event, cal_id: &str) -> CalendarListEntryResponse {
    hub.events().insert(event, cal_id).doit().await
}

fn create_event(event_details: CalenderEvent) -> Event {
    let mut req = Event::default();
    req.summary = Some(event_details.name);
    req.description = Some(event_details.description);

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
        let naive_st = event_details
            .date
            .and_time(event_details.start_time.unwrap());
        let naive_et = event_details.date.and_time(event_details.end_time.unwrap());
        req.start = Some(EventDateTime {
            date: None,
            date_time: Some(naive_st.and_local_timezone(Local).unwrap().to_utc()),
            time_zone: Some(get_current_timezone()),
        });
        req.end = Some(EventDateTime {
            date: None,
            date_time: Some(naive_et.and_local_timezone(Local).unwrap().to_utc()),
            time_zone: Some(get_current_timezone()),
        });
    }
    req
}

async fn insert_new_event(hub: auth::Hub, event_details: &Vec<String>) -> StringOutput {
    let cal_event = CalenderEvent::from(event_details);
    let event = create_event(cal_event);
    let result = insert_event(hub, event, "primary").await;
    match result {
        Ok(_success) => Ok("Event added successfully!".to_string()),
        Err(e) => {
            Ok(format!("Failed to add event because of the following error: {}", e).to_string())
        }
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

pub async fn get_calendar_service(command: &String, arguments: &Vec<String>) {
    let hub = auth::login().await;

    // All calls return a StringOutput Type
    let results = match command.as_str() {
        "event_list" => get_events(hub, arguments).await,
        "event_details" => get_event(hub, arguments).await,
        "new_event" => insert_new_event(hub, arguments).await,
        "series" => todo!(), // create a repeating event
        _ => panic!("unknown command! Please use one of: {:?}", EVENT_LIST),
    };

    match results {
        Ok(events) => {
            println!("{}", events);
        }
        Err(error) => println!("Whoops, encountered error {}", error),
    };
}
