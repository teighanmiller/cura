use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Timelike, Utc};

pub struct Period {
    pub min_time: DateTime<Utc>,
    pub max_time: DateTime<Utc>,
}

static DATE_TIME_STR: &str = "%Y-%m-%d %H:%M:%S %z";
static DATE_STR: &str = "%Y-%m-%d";

pub fn convert_date(date: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(date, DATE_STR)
}

pub fn convert_datetime(datetime: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(datetime, DATE_TIME_STR)
}

pub fn get_current_timezone() -> String {
    iana_time_zone::get_timezone().expect("Failed to get timezone")
}

fn parse_to_utc(utc: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
    DateTime::parse_from_str(utc, DATE_TIME_STR)
}

fn get_utc(utc: &str) -> DateTime<FixedOffset> {
    let datetime = parse_to_utc(utc);

    match datetime {
        Ok(datetime) => datetime,
        Err(error) => {
            panic!(
                "Expected datetime string in format {}, got {}. Resulted in the following error: {}",
                DATE_TIME_STR, utc, error
            );
        }
    }
}

fn get_utc_period(start_time: String, end_time: String) -> Period {
    let first_datetime = get_utc(&start_time);
    let second_datetime = get_utc(&end_time);

    if second_datetime > first_datetime {
        Period {
            min_time: first_datetime.into(),
            max_time: second_datetime.into(),
        }
    } else {
        Period {
            min_time: second_datetime.into(),
            max_time: first_datetime.into(),
        }
    }
}

fn get_day(day: DateTime<Utc>) -> Period {
    let min_time = day
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    let max_time = day
        .with_hour(23)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
        .with_nanosecond(59)
        .unwrap();
    Period { min_time, max_time }
}

fn default_time() -> Period {
    let now = Utc::now();
    get_day(now)
}

pub fn get_period(args: &[String]) -> Period {
    if args.is_empty() || (args.len() == 1 && args[0] == "today") {
        eprintln!("Searching {} for events.", Utc::now());
        default_time()
    } else {
        get_utc_period(args[0].clone(), args[1].clone())
    }
}
