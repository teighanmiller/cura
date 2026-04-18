use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Timelike, Utc};

pub struct Period {
    pub min_time: DateTime<Utc>,
    pub max_time: DateTime<Utc>,
}

// static DATE_STR: &str = "%Y-%m-%d";
static DATE_TIME_STR: &str = "%Y-%m-%d %H:%M:%S %z";
static DATE_STR: &str = "%Y-%m-%d";
static TIME_STR: &str = "%H:%M:%S";

pub fn convert_time(time: &String) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(&time, TIME_STR)
}

pub fn convert_date(date: &String) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(&date, DATE_STR)
}

pub fn get_current_timezone() -> String {
    let tz_str = iana_time_zone::get_timezone().expect("Failed to get timezone");
    tz_str
}

fn parse_to_utc(utc: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
    DateTime::parse_from_str(utc, DATE_TIME_STR)
}

fn get_utc(utc: &str) -> DateTime<FixedOffset> {
    let datetime = parse_to_utc(&utc);

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

fn get_utc_period(period: &Vec<String>) -> Period {
    let first_datetime = get_utc(&period[0]);
    let second_datetime = get_utc(&period[1]);

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
    Period {
        min_time: min_time,
        max_time: max_time,
    }
}

fn default_time() -> Period {
    let now = Utc::now();
    get_day(now)
}

pub fn get_period(period: &Vec<String>) -> Period {
    if period.len() == 0 {
        println!("No dates provided, defaulting to today as time period to search.");
        default_time()
    } else if period.len() == 1 {
        match period[0].as_str() {
            "today" => default_time(),
            _ => default_time(),
        }
    } else {
        get_utc_period(period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn convert_time_valid() {
        let result = convert_time(&"14:30:00".to_string());
        assert!(result.is_ok());
        let t = result.unwrap();
        assert_eq!(t.hour(), 14);
        assert_eq!(t.minute(), 30);
    }

    #[test]
    fn convert_time_invalid_format() {
        let result = convert_time(&"2:30 PM".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn convert_date_valid() {
        let result = convert_date(&"2026-04-18".to_string());
        assert!(result.is_ok());
        let d = result.unwrap();
        assert_eq!(d.year(), 2026);
        assert_eq!(d.month(), 4);
        assert_eq!(d.day(), 18);
    }

    #[test]
    fn convert_date_invalid_format() {
        let result = convert_date(&"04/18/2026".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn get_period_empty_returns_today() {
        let period = get_period(&vec![]);
        assert!(period.min_time <= period.max_time);
        // min should be start of some day (hour 0)
        assert_eq!(period.min_time.hour(), 0);
        assert_eq!(period.max_time.hour(), 23);
    }

    #[test]
    fn get_period_today_keyword() {
        let period = get_period(&vec!["today".to_string()]);
        assert_eq!(period.min_time.hour(), 0);
        assert_eq!(period.max_time.hour(), 23);
    }

    #[test]
    fn get_period_two_datetimes_ordered() {
        let args = vec![
            "2026-04-18 09:00:00 +0000".to_string(),
            "2026-04-18 17:00:00 +0000".to_string(),
        ];
        let period = get_period(&args);
        assert!(period.min_time < period.max_time);
    }

    #[test]
    fn get_period_two_datetimes_reversed_input() {
        // Later time given first — should still produce min < max
        let args = vec![
            "2026-04-18 17:00:00 +0000".to_string(),
            "2026-04-18 09:00:00 +0000".to_string(),
        ];
        let period = get_period(&args);
        assert!(period.min_time < period.max_time);
    }
}
