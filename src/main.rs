use icalendar::Calendar;
use icalendar::CalendarComponent::{Event, Todo, Venue};
use rrule::{RRuleSet, Tz};
use std::{env, fs};

use now::TimeZoneNow;

use chrono::{Duration, FixedOffset, Local, NaiveDateTime, TimeZone};

use itertools::Itertools;

const RRULE_PROPERTIES: [&str; 5] = ["DTSTART", "RRULE", "EXRULE", "RDATE", "EXDATE"];

struct AgendaEntry {
    name: String,
    start: NaiveDateTime,
    duration: Duration,
}

fn fmt_duration(d: chrono::Duration) -> String {
    if d.num_hours() != 0 {
        return format!("{}h", d.num_hours());
    }

    if d.num_minutes() != 0 {
        return format!("{}min", d.num_minutes());
    }

    return format!("{}s", d.num_seconds());
}

fn fmt_agenda_entry(entry: AgendaEntry, when: NaiveDateTime) -> String {
    let delta_start = entry.start - when;
    let delta_end = (entry.start + entry.duration) - when;

    if entry.start > when {
        return format!("{} · {}", entry.name, fmt_duration(delta_start));
    }

    return format!(
        "{} · {}/{}",
        entry.name,
        fmt_duration(delta_start),
        fmt_duration(delta_end)
    );
}

fn as_naive(dt: icalendar::CalendarDateTime) -> NaiveDateTime {
    match dt {
        icalendar::CalendarDateTime::Floating(f) => return f,
        icalendar::CalendarDateTime::Utc(u) => return u.naive_local(),
        icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
            return date_time; // TODO: handle better
        }
    }
}

fn extract_event(
    event: &impl icalendar::Component,
    sod: chrono::DateTime<Tz>,
    eod: chrono::DateTime<Tz>,
    offset: FixedOffset,
) -> Vec<AgendaEntry> {
    let naive_start: NaiveDateTime = match event.get_start() {
        Some(start_time) => match start_time {
            icalendar::DatePerhapsTime::DateTime(dt) => as_naive(dt),
            icalendar::DatePerhapsTime::Date(d) => d.and_hms(0, 0, 0),
        },
        None => return vec![], // TODO: error handle as event is missing a start time
    };

    let duration = match event.get_end() {
        Some(end_time) => match end_time {
            icalendar::DatePerhapsTime::DateTime(et) => as_naive(et) - naive_start,
            icalendar::DatePerhapsTime::Date(_) => offset.end_of_day().naive_local() - naive_start,
        },
        None => return vec![], // TODO: handle the case where we have a start time but
                               // are missing an end time better
    };

    if event.property_value("RRULE").is_none() {
        return vec![AgendaEntry {
            name: event.get_summary().unwrap_or("").to_owned(),
            start: naive_start,
            duration,
        }];
    }

    let props: String = RRULE_PROPERTIES
        .iter()
        .map(|p| match event.property_value(p) {
            Some(x) => format!("{}:{}\n", p, x),
            None => "".to_owned(),
        })
        .collect();

    match props.parse::<RRuleSet>() {
        Ok(rrule) => {
            return rrule
                .after(sod)
                .before(eod)
                .all(100)
                .dates
                .drain(..)
                .map(|a| AgendaEntry {
                    name: event.get_summary().unwrap_or("").to_owned(),
                    start: a.naive_local(),
                    duration,
                })
                .collect();
        }
        Err(_) => return vec![], //println!("No rrule!"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(file_name) = env::args().nth(1) {
        // TODO: Error handle better!
        let file_contents = fs::read_to_string(file_name);
        let parsed_calendar = file_contents?.parse::<Calendar>()?;

        let offset = Local::now().offset().clone();
        let sod = Tz::UTC
            .from_local_datetime(&offset.beginning_of_day().naive_local())
            .unwrap();
        let eod = sod + Duration::hours(32); // Tz::UTC.from_local_datetime(&offset.end_of_day().naive_local()).unwrap();

        let current_time = chrono::offset::Local.now().naive_local();

        let ans: String = parsed_calendar
            .iter()
            .flat_map(|element| {
                match element {
                    Event(e) => extract_event(e, sod, eod, offset),
                    Todo(t) => extract_event(t, sod, eod, offset),
                    Venue(v) => extract_event(v, sod, eod, offset),
                    &_ => vec![], // TODO: LOG!
                }
            })
            .sorted_unstable_by_key(|item| item.start)
            .filter(|item| {
                return (item.start + item.duration) >= current_time
                    && (current_time - item.start).num_hours() < 24;
            })
            .take(2)
            .map(|item| fmt_agenda_entry(item, current_time))
            .intersperse(" » ".to_owned())
            .collect();

        println!("{}", ans);
    }
    return Ok(());
}
