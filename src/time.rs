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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike, Utc};

    #[test]
    fn convert_date_valid() {
        assert!(convert_date("2024-01-15").is_ok());
    }

    #[test]
    fn convert_date_invalid_format() {
        assert!(convert_date("01/15/2024").is_err());
    }

    #[test]
    fn convert_date_empty() {
        assert!(convert_date("").is_err());
    }

    #[test]
    fn convert_datetime_valid() {
        assert!(convert_datetime("2024-01-15 09:30:00 +0000").is_ok());
    }

    #[test]
    fn convert_datetime_invalid_format() {
        assert!(convert_datetime("2024-01-15").is_err());
    }

    #[test]
    fn convert_datetime_empty() {
        assert!(convert_datetime("").is_err());
    }

    #[test]
    fn parse_to_utc_valid() {
        assert!(parse_to_utc("2024-01-15 09:30:00 +0000").is_ok());
    }

    #[test]
    fn parse_to_utc_with_offset() {
        assert!(parse_to_utc("2024-01-15 09:30:00 -0500").is_ok());
    }

    #[test]
    fn parse_to_utc_invalid() {
        assert!(parse_to_utc("not-a-date").is_err());
    }

    #[test]
    fn get_utc_valid() {
        let _ = get_utc("2024-01-15 09:30:00 +0000");
    }

    #[test]
    #[should_panic]
    fn get_utc_panics_on_invalid() {
        get_utc("bad-input");
    }

    #[test]
    fn get_utc_period_ordered() {
        let p = get_utc_period(
            "2024-01-15 08:00:00 +0000".to_string(),
            "2024-01-15 10:00:00 +0000".to_string(),
        );
        assert!(p.min_time <= p.max_time);
    }

    #[test]
    fn get_utc_period_reversed_input() {
        let p = get_utc_period(
            "2024-01-15 10:00:00 +0000".to_string(),
            "2024-01-15 08:00:00 +0000".to_string(),
        );
        assert!(p.min_time <= p.max_time);
    }

    #[test]
    fn get_day_bounds() {
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 0).unwrap();
        let p = get_day(now);
        assert_eq!(p.min_time.hour(), 0);
        assert_eq!(p.min_time.minute(), 0);
        assert_eq!(p.max_time.hour(), 23);
        assert_eq!(p.max_time.minute(), 59);
        assert_eq!(p.max_time.second(), 59);
    }
}
