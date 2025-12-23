import { EventCalendar } from '@/components/event-calendar'
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
            events={{ url: "/api/calendar.ics", format: 'ics' }}
            addButton={{
                text: 'Add Event',
                click() {
                    alert('add event...')
                }
            }}
        />
    )
}
