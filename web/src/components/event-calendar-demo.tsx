import { EventCalendar } from '@/components/event-calendar'
import iCalendarPlugin from '@fullcalendar/icalendar'

export function EventCalendarDemo() {
  return (
    <EventCalendar
      plugins={[iCalendarPlugin]}
      eventDisplay='block'
      className='max-w-300 my-10 mx-auto'
      editable
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
