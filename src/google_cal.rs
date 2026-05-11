use crate::auth;
use crate::time::{convert_date, convert_datetime, get_current_timezone, get_period};
use chrono::{NaiveDate, NaiveDateTime};
use clap::ValueEnum;
use google_calendar3::Error;
use google_calendar3::api::{Event, EventDateTime, Events};
use http::Response;
use http_body_util::combinators::BoxBody;

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
            .await?;
        Ok(event_to_string(events))
    } else {
        let (_response, events) = hub
            .events()
            .list("primary")
            .time_min(event_query.start_time.unwrap().and_utc())
            .time_max(event_query.end_time.unwrap().and_utc())
            .q(event_query.name.as_str())
            .doit()
            .await?;
        Ok(event_to_string(events))
    }
}

async fn get_events(
    hub: auth::Hub,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<StringOutput, Error> {
    let mut period = get_period(&vec![]);
    if start_time.is_some() & end_time.is_some() {
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
