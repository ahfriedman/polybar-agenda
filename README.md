# polybar-agenda
Polybar module to display upcomming calendar events with durations inspired by [taskwarrior-polybar](https://github.com/dakuten/taskwarrior-polybar). 

Unlike exising polybar calendar solutions such as [i3-agenda](https://github.com/rosenpin/i3-agenda), this module runs 100% locally—allowing it to work offline and with negligible performace impacts.

## Example

![image](https://github.com/user-attachments/assets/9cc4bfc0-9229-4b01-bc0a-97f8cb6e2ece)

## Features/What does it do?

This module will read a calendar file (ics), and will display up to the next two events you have for today. This is in the format of either: 

```
<event 1 name> · <time until event 1> (» <event 2 name> · <time until event 2>)?
```
or
```
<event 1 name> · <time since start of event 1>/<time until end of event 1> (» <event 2 name> · <time until event 2>)?
```
depending on if there is a current ongoing event or not. For instance, in the example image above, the calendar event dinner started 48 minutes ago and there is one hour until the end of the event. 
The subsequent event, rest, starts in one hour as well. 

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


## Future Directions
- [ ] Read the location field for events and display that if present
- [ ] Allow for number of events to be configured (e.g., to display more than at most 2)
- [ ] Allow for custom time ranges of events to be displayed (e.g., so that way they aren't limited to today's events)
- [ ] Allow time since start of event/time until end of event to be disabled/enabled per event
- [ ] Extend ICS file with field/allow users to use some local configuration to change the formatting of the text per event (e.g., allow events to have different font colors w/o manually adding polybar formatting keys to the event's title)
- [ ] Allow for reading multiple ics files at once and/or all ics files in a directory to make managing multiple calendars easier
- [ ] Revise code to handle edge cases more gracefully and improve the installation process
