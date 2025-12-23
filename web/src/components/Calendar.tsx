import { EventCalendar } from '@/components/event-calendar'
import type { EventClickData, EventDisplayData, EventInput } from '@fullcalendar/core'
import iCalendarPlugin from '@fullcalendar/icalendar'

export function CalendarApp() {
    return (
        <EventCalendar
            plugins={[iCalendarPlugin]}
            eventDisplay='block'
            firstDay={1} // Week starts on Monday
            eventTimeFormat={{
                hour: '2-digit',
                minute: '2-digit',
                hour12: false,
            }}
            availableViews={['dayGrid', 'list']}
            initialView='dayGrid'
            multiMonthMaxColumns={1}
            duration={{ months: 3 }}
            className='max-w-300 my-10 mx-auto'
            selectable
            nowIndicator
            navLinks
            timeZone='UTC'
            events={{ url: "/api/calendar.ics", format: 'ics', eventDataTransform: eventDataTransform }}
            eventContent={renderEventContent}
            eventClick={eventClick}
            addButton={{
                text: 'Add Event',
                click() {
                    alert('add event...')
                }
            }}
        />
    )
}
function eventClick(data: EventClickData): void {
    console.log('Event clicked: ', data.event.title);
    console.log(data.event);
    //data.jsEvent.preventDefault();
}


function eventDataTransform(input: EventInput): EventInput {
    let props = input.extendedProps || {};
    props.url = input.url;
    input.extendedProps = props;
    input.url = ""; // disable default navigation
    return input;
}


function renderEventContent(eventInfo: EventDisplayData) {
    return (
        <>
            <b>{eventInfo.timeText}</b>
            <i>{eventInfo.event.title}</i>
        </>
    )
}
