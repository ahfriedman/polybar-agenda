# polybar-agenda
Polybar module to display upcomming calendar events with durations inspired by [taskwarrior-polybar](https://github.com/dakuten/taskwarrior-polybar). 

Unlike exising polybar calendar solutions such as [i3-agenda](https://github.com/rosenpin/i3-agenda), this module runs 100% locally—allowing it to work offline and with negligible performace impacts.

## Example

![image](https://github.com/user-attachments/assets/9cc4bfc0-9229-4b01-bc0a-97f8cb6e2ece)

## Features/What does it do?

This module will read a calendar file (ics), and will display up to the next two events you have for today with 
information about each event separated by "»". Information about events can be displayed in the following formats: 
**Default Display Mode**
For upcomming events: 
```
<event name> <event time> (in <time until event>) 
```
For ongoing events: 
```
<event name> <event time> (<time since start> ago)
```

**Compact Display Mode**
For upcomming events: 
```
<event name> · <time until event> 
```
When an event is in progress:
```
<event name> · <time since start of event>/<time until end of event>
```


This is all designed to work out of the box with system calendars---including recurring events.

## Installation 
Pre-reqs: Cargo/rust is installed (along with polybar or some other similar bar to display the results)
1. Download this repository
2. Navigate to the files in a terminal and run `cargo build -r` from the project's root directory.
3. Add the following to your polybar config:
```
[module/polybar-agenda]
interval = 30
type = custom/script
exec = <path to downloaded repository>/target/release/polybar-agenda <path to calendar to display events from>
format = <label>
format-foreground = #FFF
format-prefix = "󰃭 "
```
For example, if you are running a ubuntu based system using the default calendar app, if you provide the calendar `~/.local/share/evolution/calendar/system/calendar.ics`, you should get system calendar events. 

To switch to the compact display mode, pass the parameter `--display-compact` before specifying the ics file to read.

## Future Directions
- [ ] Read the location field for events and display that if present
- [ ] Allow for number of events to be configured (e.g., to display more than at most 2)
- [ ] Allow for custom time ranges of events to be displayed (e.g., so that way they aren't limited to today's events)
- [ ] Allow time since start of event/time until end of event to be disabled/enabled per event
- [ ] Extend ICS file with field/allow users to use some local configuration to change the formatting of the text per event (e.g., allow events to have different font colors w/o manually adding polybar formatting keys to the event's title)
- [ ] Allow for reading multiple ics files at once and/or all ics files in a directory to make managing multiple calendars easier
- [ ] Revise code to handle edge cases more gracefully and improve the installation process
