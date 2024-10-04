use icalendar::Calendar;
use icalendar::CalendarComponent::{Event, Todo, Venue};
use now::DateTimeNow;
use rrule::{RRuleSet, Tz};
use std::{env, fs};

use chrono::{Duration, Local, NaiveDateTime, TimeZone};

use itertools::Itertools;

const RRULE_PROPERTIES: [&str; 5] = ["DTSTART", "RRULE", "EXRULE", "RDATE", "EXDATE"];

struct AgendaEntry {
    name: String,
    start: NaiveDateTime,
    duration: Duration,
}

#[derive(Clone,Copy)]
enum DisplayMode {
    Default, 
    Compact
}

fn fmt_duration(d: Duration) -> String {
    if d.num_hours() != 0 {
        return format!("{}h", d.num_hours());
    }

    if d.num_minutes() != 0 {
        return format!("{}min", d.num_minutes());
    }

    format!("{}s", d.num_seconds())
}

fn fmt_agenda_entry(mode: DisplayMode, entry: AgendaEntry, when: NaiveDateTime) -> String {
    match mode {
        DisplayMode::Default => fmt_agenda_entry_default(entry, when),
        DisplayMode::Compact => fmt_agenda_entry_compact(entry, when)
    }
}

fn fmt_agenda_entry_compact(entry: AgendaEntry, when: NaiveDateTime) -> String {
    let time_until = entry.start.signed_duration_since(when);
    let time_remaining = (entry.start + entry.duration).signed_duration_since(when);

    if time_until.num_minutes() > 0 || time_until.num_hours() > 0 {
        return format!(
        "{} · {}",
        entry.name,
        fmt_duration(time_until));
    }

    format!(
        "{} · {}/{}",
        entry.name,
        fmt_duration(time_until),
        fmt_duration(time_remaining))
}

fn fmt_agenda_entry_default(entry: AgendaEntry, when: NaiveDateTime) -> String {
    let start_time = entry.start.format("%H:%M").to_string();
    let time_until = when.signed_duration_since(entry.start);

    if time_until.num_minutes() > 0 || time_until.num_hours() > 0 {
        return format!(
            "{} {} ({} ago)",
            entry.name,
            start_time,
            fmt_duration(time_until)
        );
    }

    format!(
        "{} {} (in {})",
        entry.name,
        start_time,
        fmt_duration(time_until.abs())
    )
}

fn as_naive(dt: icalendar::CalendarDateTime) -> NaiveDateTime {
    match dt {
        icalendar::CalendarDateTime::Floating(f) => f,
        icalendar::CalendarDateTime::Utc(u) => {
            Local.from_utc_datetime(&u.naive_utc()).naive_local()
        }
        icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => date_time,
    }
}

fn extract_event(
    event: &impl icalendar::Component,
    sod: chrono::DateTime<Local>,
    eod: chrono::DateTime<Local>,
) -> Vec<AgendaEntry> {
    let naive_start: NaiveDateTime = match event.get_start() {
        Some(start_time) => match start_time {
            icalendar::DatePerhapsTime::DateTime(dt) => as_naive(dt),
            icalendar::DatePerhapsTime::Date(d) => Local
                .from_local_date(&d)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .naive_local(),
        },
        None => return vec![], // TODO: error handle as event is missing a start time
    };

    let duration = match event.get_end() {
        Some(end_time) => match end_time {
            icalendar::DatePerhapsTime::DateTime(et) => as_naive(et) - naive_start,
            icalendar::DatePerhapsTime::Date(_) => {
                Local::now().end_of_day().naive_local() - naive_start
            }
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
                .after(sod.with_timezone(&Tz::UTC))
                .before(eod.with_timezone(&Tz::UTC))
                .all(100)
                .dates
                .drain(..)
                .map(|a| AgendaEntry {
                    name: event.get_summary().unwrap_or("").to_owned(),
                    start: Local.from_utc_datetime(&a.naive_utc()).naive_local(),
                    duration,
                })
                .collect();
        }
        Err(_) => vec![], //println!("No rrule!"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args : Vec<_> = env::args().collect();
    if args.len() < 1 {
        println!("Calendar file not provided");
        return Result::Ok(());
    }

    println!("{args:?}");
    let mode = if args.len() == 3 { match args.get(1).unwrap().as_str() {
        "--display-compact" => DisplayMode::Compact,
        _ => DisplayMode::Default,
    }} else { DisplayMode::Default };

    if let Some(file_name) = args.last() {
        let file_contents = fs::read_to_string(file_name)?;
        let parsed_calendar = file_contents.parse::<Calendar>()?;

        let now = Local::now();

        let current_time = now.naive_local();

        let extract_start = now - Duration::hours(32);
        let extract_end = now + Duration::hours(32);

        let ans: String = Itertools::intersperse_with(
            parsed_calendar
                .iter()
                .flat_map(|element| {
                    match element {
                        Event(e) => extract_event(e, extract_start, extract_end),
                        Todo(t) => extract_event(t, extract_start, extract_end),
                        Venue(v) => extract_event(v, extract_start, extract_end),
                        &_ => vec![], // TODO: LOG!
                    }
                })
                .sorted_unstable_by_key(|item| item.start)
                .filter(|item| {
                    (item.start + item.duration) >= current_time
                        && (current_time - item.start).num_hours() < 24
                })
                .take(2)
                .map(|item| fmt_agenda_entry(mode, item, current_time)),
            || " » ".to_owned(),
        )
        .collect();

        println!("{}", ans);
    }
    Ok(())
}
