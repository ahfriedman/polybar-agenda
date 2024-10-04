use std::{env, fs};

mod calendar {
    use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
    use chrono_tz::Tz;
    use icalendar::{Calendar, CalendarComponent, Component, DatePerhapsTime};
    use itertools::Itertools;
    use now::DateTimeNow;
    use rrule::{RRuleSet, Tz as RRuleTz};
    use std::str::FromStr;

    // Custom error type for better error handling
    #[derive(Debug)]
    pub enum CalendarError {
        MissingStartTime,
        MissingEndTime,
        InvalidTimezone(String),
        RRuleParseError(String),
    }

    // Constants
    const RRULE_PROPERTIES: [&str; 5] = ["DTSTART", "RRULE", "EXRULE", "RDATE", "EXDATE"];
    pub const MAX_EVENTS: u16 = 100;
    pub const HOURS_AHEAD: i64 = 32;
    pub const HOURS_BEHIND: i64 = 32;

    pub struct AgendaEntry {
        pub name: String,
        pub start: NaiveDateTime,
        pub duration: Duration,
    }

    impl AgendaEntry {
        pub fn new(name: String, start: NaiveDateTime, duration: Duration) -> Self {
            Self {
                name,
                start,
                duration,
            }
        }
    }

    #[derive(Clone, Copy)]
    pub enum DisplayMode {
        Default,
        Compact,
    }

    // Convert CalendarDateTime to NaiveDateTime
    pub fn as_naive(dt: icalendar::CalendarDateTime) -> Result<NaiveDateTime, CalendarError> {
        match dt {
            icalendar::CalendarDateTime::Floating(f) => Ok(f),
            icalendar::CalendarDateTime::Utc(u) => {
                Ok(Local.from_utc_datetime(&u.naive_utc()).naive_local())
            }
            icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                let tz = Tz::from_str(&tzid)
                    .map_err(|_| CalendarError::InvalidTimezone(tzid.clone()))?;
                tz.from_local_datetime(&date_time)
                    .single()
                    .ok_or_else(|| CalendarError::InvalidTimezone(tzid))
                    .map(|dt| dt.naive_local())
            }
        }
    }

    // Extract events from a calendar component
    pub fn extract_event(
        event: &impl Component,
        sod: DateTime<Local>,
        eod: DateTime<Local>,
    ) -> Result<Vec<AgendaEntry>, CalendarError> {
        let start = event.get_start().ok_or(CalendarError::MissingStartTime)?;
        let naive_start = match start {
            DatePerhapsTime::DateTime(dt) => as_naive(dt)?,
            DatePerhapsTime::Date(d) => Local
                .from_local_date(&d)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .naive_local(),
        };

        let duration = match event.get_end() {
            Some(end_time) => match end_time {
                DatePerhapsTime::DateTime(et) => as_naive(et)? - naive_start,
                DatePerhapsTime::Date(_) => Local::now().end_of_day().naive_local() - naive_start,
            },
            None => return Err(CalendarError::MissingEndTime),
        };

        let name = event.get_summary().unwrap_or("").to_owned();

        if event.property_value("RRULE").is_none() {
            return Ok(vec![AgendaEntry::new(name, naive_start, duration)]);
        }

        let props: String = RRULE_PROPERTIES
            .iter()
            .filter_map(|&p| event.property_value(p).map(|x| format!("{}:{}\n", p, x)))
            .collect();

        let rrule = props
            .parse::<RRuleSet>()
            .map_err(|e| CalendarError::RRuleParseError(e.to_string()))?;

        Ok(rrule
            .after(sod.with_timezone(&RRuleTz::UTC))
            .before(eod.with_timezone(&RRuleTz::UTC))
            .all(MAX_EVENTS)
            .dates
            .into_iter()
            .map(|a| {
                AgendaEntry::new(
                    name.clone(),
                    Local.from_utc_datetime(&a.naive_utc()).naive_local(),
                    duration,
                )
            })
            .collect())
    }

    pub fn format_duration(d: Duration) -> String {
        if d.num_hours() != 0 {
            return format!("{}h", d.num_hours());
        }
        if d.num_minutes() != 0 {
            return format!("{}min", d.num_minutes());
        }
        format!("{}s", d.num_seconds())
    }

    pub fn format_agenda_entry(
        mode: DisplayMode,
        entry: &AgendaEntry,
        when: NaiveDateTime,
    ) -> String {
        match mode {
            DisplayMode::Default => format_agenda_entry_default(entry, when),
            DisplayMode::Compact => format_agenda_entry_compact(entry, when),
        }
    }

    pub fn format_agenda_entry_compact(entry: &AgendaEntry, when: NaiveDateTime) -> String {
        let time_until = entry.start.signed_duration_since(when);
        let time_remaining = (entry.start + entry.duration).signed_duration_since(when);

        if time_until.num_minutes() > 0 || time_until.num_hours() > 0 {
            format!("{} · {}", entry.name, format_duration(time_until))
        } else {
            format!(
                "{} · {}/{}",
                entry.name,
                format_duration(time_until.abs()),
                format_duration(time_remaining)
            )
        }
    }

    pub fn format_agenda_entry_default(entry: &AgendaEntry, when: NaiveDateTime) -> String {
        let start_time = entry.start.format("%H:%M").to_string();
        let time_until = when.signed_duration_since(entry.start);

        if time_until.num_seconds() > 0 {
            format!(
                "{} {} ({} ago)",
                entry.name,
                start_time,
                format_duration(time_until)
            )
        } else {
            format!(
                "{} {} (in {})",
                entry.name,
                start_time,
                format_duration(time_until.abs())
            )
        }
    }

    pub fn process_calendar(
        calendar: &Calendar,
        mode: DisplayMode,
        now: DateTime<Local>,
    ) -> String {
        let current_time = now.naive_local();
        let extract_start = now - Duration::hours(HOURS_BEHIND);
        let extract_end = now + Duration::hours(HOURS_AHEAD);

        calendar
            .iter()
            .filter_map(|element| match element {
                CalendarComponent::Event(e) => extract_event(e, extract_start, extract_end).ok(),
                CalendarComponent::Todo(t) => extract_event(t, extract_start, extract_end).ok(),
                CalendarComponent::Venue(v) => extract_event(v, extract_start, extract_end).ok(),
                _ => None,
            })
            .flatten()
            .sorted_unstable_by_key(|item| item.start)
            .filter(|item| {
                (item.start + item.duration) >= current_time
                    && (current_time - item.start).num_hours() < 24
            })
            .take(2)
            .map(|item| format_agenda_entry(mode, &item, current_time))
            .intersperse(" » ".to_owned())
            .collect()
    }
}

use calendar::{process_calendar, DisplayMode};
use chrono::Local;
use icalendar::Calendar;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Calendar file not provided".into());
    }

    let mode = if args.len() >= 3 && args[1] == "--display-compact" {
        DisplayMode::Compact
    } else {
        DisplayMode::Default
    };

    let file_name = args.last().unwrap();
    let file_contents = fs::read_to_string(file_name)?;
    let parsed_calendar = file_contents.parse::<Calendar>()?;

    let now = Local::now();

    let formatted_agenda = process_calendar(&parsed_calendar, mode, now);

    println!("{}", formatted_agenda);
    Ok(())
}

#[cfg(test)]
mod tests;
