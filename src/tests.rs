#[cfg(test)]
use crate::calendar::*;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use icalendar::{Calendar, CalendarComponent, Component, Event, EventLike};
use itertools::Itertools;

// Helper function to create a test event
fn create_test_event(summary: &str, start: NaiveDateTime, duration: Duration) -> Event {
    let mut event = Event::new();
    event.summary(summary);
    event.starts(start);
    event.ends(start + duration);
    event
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(Duration::hours(2)), "2h");
    assert_eq!(format_duration(Duration::minutes(30)), "30min");
    assert_eq!(format_duration(Duration::seconds(45)), "45s");
    assert_eq!(
        format_duration(Duration::hours(1) + Duration::minutes(30)),
        "1.5h"
    );
    assert_eq!(format_duration(Duration::zero()), "0s");
}

#[test]
fn test_format_agenda_entry_default() {
    let now = NaiveDate::from_ymd_opt(2023, 5, 1)
        .unwrap()
        .and_hms_opt(14, 0, 0)
        .unwrap();

    // Test future event
    let future_event = AgendaEntry::new(
        "Future Event".to_string(),
        now + Duration::minutes(30),
        Duration::hours(1),
    );
    assert_eq!(
        format_agenda_entry_default(&future_event, now),
        "Future Event 14:30 (in 30min)"
    );

    // Test past event
    let past_event = AgendaEntry::new(
        "Past Event".to_string(),
        now - Duration::minutes(30),
        Duration::hours(1),
    );
    assert_eq!(
        format_agenda_entry_default(&past_event, now),
        "Past Event 13:30 (30min ago)"
    );
}

#[test]
fn test_format_agenda_entry_compact() {
    let now = NaiveDate::from_ymd_opt(2023, 5, 1)
        .unwrap()
        .and_hms_opt(14, 0, 0)
        .unwrap();

    // Test future event
    let future_event = AgendaEntry::new(
        "Future Event".to_string(),
        now + Duration::minutes(45),
        Duration::hours(1),
    );
    assert_eq!(
        format_agenda_entry_compact(&future_event, now),
        "Future Event · 45min"
    );

    // Test ongoing event
    let ongoing_event = AgendaEntry::new(
        "Ongoing Event".to_string(),
        now - Duration::minutes(45),
        Duration::hours(2),
    );
    assert_eq!(
        format_agenda_entry_compact(&ongoing_event, now),
        "Ongoing Event · 45min/1.25h"
    );
}

#[test]
fn test_as_naive() {
    // Test Floating time
    let floating = icalendar::CalendarDateTime::Floating(
        NaiveDate::from_ymd_opt(2023, 5, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap(),
    );
    assert_eq!(
        as_naive(floating).unwrap(),
        NaiveDate::from_ymd_opt(2023, 5, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap()
    );

    // Test UTC time
    let utc = icalendar::CalendarDateTime::Utc(chrono::DateTime::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(2023, 5, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap(),
        chrono::Utc,
    ));
    assert!(as_naive(utc).is_ok());

    // Test WithTimezone
    let with_tz = icalendar::CalendarDateTime::WithTimezone {
        date_time: NaiveDate::from_ymd_opt(2023, 5, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap(),
        tzid: "America/New_York".to_string(),
    };
    assert!(as_naive(with_tz).is_ok());

    // Test invalid timezone
    let invalid_tz = icalendar::CalendarDateTime::WithTimezone {
        date_time: NaiveDate::from_ymd_opt(2023, 5, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap(),
        tzid: "Invalid/Timezone".to_string(),
    };
    assert!(as_naive(invalid_tz).is_err());
}

#[test]
fn test_extract_event() {
    let now = Local::now();
    let sod = now - Duration::hours(HOURS_BEHIND);
    let eod = now + Duration::hours(HOURS_AHEAD);

    // Test single event
    let single_event = create_test_event("Single Event", now.naive_local(), Duration::hours(1));
    let extracted = extract_event(&single_event, sod, eod).unwrap();
    assert_eq!(extracted.len(), 1);
    assert_eq!(extracted[0].name, "Single Event");

    // Test recurring event
    let mut recurring_event =
        create_test_event("Recurring Event", now.naive_local(), Duration::hours(1));
    recurring_event.add_property("RRULE", "FREQ=DAILY;COUNT=3");
    let extracted = extract_event(&recurring_event, sod, eod).unwrap();
    assert_eq!(extracted.len(), 2);
    assert!(extracted.iter().all(|e| e.name == "Recurring Event"));

    // Test event without end time
    let mut no_end_event = Event::new();
    no_end_event.summary("No End Event");
    no_end_event.starts(now.naive_local());
    assert!(extract_event(&no_end_event, sod, eod).is_err());

    // Test event without start time
    let no_start_event = Event::new();
    assert!(extract_event(&no_start_event, sod, eod).is_err());
}

#[test]
fn test_format_agenda_entry() {
    let now = NaiveDate::from_ymd_opt(2023, 5, 1)
        .unwrap()
        .and_hms_opt(14, 0, 0)
        .unwrap();
    let event = AgendaEntry::new(
        "Test Event".to_string(),
        now + Duration::minutes(30),
        Duration::hours(1),
    );

    assert_eq!(
        format_agenda_entry(DisplayMode::Default, &event, now),
        "Test Event 14:30 (in 30min)"
    );
    assert_eq!(
        format_agenda_entry(DisplayMode::Compact, &event, now),
        "Test Event · 30min"
    );
}

// Integration-like test for the main logic
#[test]
fn test_calendar_processing() {
    let mut calendar = Calendar::new();
    let now = Local::now();

    // Add some test events
    let event1 = create_test_event(
        "Event 1",
        now.naive_local() + Duration::hours(1),
        Duration::hours(2),
    );
    let event2 = create_test_event(
        "Event 2",
        now.naive_local() + Duration::hours(3),
        Duration::hours(1),
    );
    let past_event = create_test_event(
        "Past Event",
        now.naive_local() - Duration::hours(2),
        Duration::hours(1),
    );
    let far_future_event = create_test_event(
        "Far Future Event",
        now.naive_local() + Duration::hours(25),
        Duration::hours(1),
    );

    calendar.push(event1);
    calendar.push(event2);
    calendar.push(past_event);
    calendar.push(far_future_event);

    let formatted_agenda: String = calendar
        .iter()
        .filter_map(|element| match element {
            CalendarComponent::Event(e) => extract_event(
                e,
                now - Duration::hours(HOURS_BEHIND),
                now + Duration::hours(HOURS_AHEAD),
            )
            .ok(),
            _ => None,
        })
        .flatten()
        .sorted_unstable_by_key(|item| item.start)
        .filter(|item| {
            (item.start + item.duration) >= now.naive_local()
                && (now.naive_local() - item.start).num_hours() < 24
        })
        .take(2)
        .map(|item| format_agenda_entry(DisplayMode::Default, &item, now.naive_local()))
        .intersperse(" » ".to_owned())
        .collect();

    // Check that we have two events in the output
    let events: Vec<&str> = formatted_agenda.split(" » ").collect();
    assert_eq!(events.len(), 2);

    // Check that the events are in the correct order
    assert!(events[0].starts_with("Event 1"));
    assert!(events[1].starts_with("Event 2"));

    // Check that past and far future events are not included
    assert!(!formatted_agenda.contains("Past Event"));
    assert!(!formatted_agenda.contains("Far Future Event"));
}
