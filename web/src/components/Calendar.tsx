import { useState, useCallback } from 'react'
import { EventCalendar } from '@/components/event-calendar'
import type { EventClickData, EventInput } from '@fullcalendar/core'
import type { EventApi } from '@fullcalendar/core'
import iCalendarPlugin from '@fullcalendar/icalendar'
import { EventDialog } from './EventDialog'

const plugins = [iCalendarPlugin]
const eventTimeFormat = {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
} as const
const availableViews = ['dayGrid', 'list']
const duration = { months: 3 }
const events = { url: "/api/calendar.json", format: 'json', eventDataTransform: eventDataTransform }
const addButton = {
    text: 'Add Event',
    click() {
        alert('add event...')
    }
}

export function CalendarApp() {
    const [selectedEvent, setSelectedEvent] = useState<EventApi | null>(null)

    const handleEventClick = useCallback((data: EventClickData): void => {
        setSelectedEvent(data.event)
    }, [])

    return (
        <>
            <EventCalendar
                plugins={plugins}
                eventDisplay='block'
                firstDay={1} // Week starts on Monday
                eventTimeFormat={eventTimeFormat}
                availableViews={availableViews}
                initialView='dayGrid'
                multiMonthMaxColumns={1}
                duration={duration}
                className='max-w-300 my-10 mx-auto'
                selectable
                nowIndicator
                navLinks
                timeZone='UTC'
                events={events}
                eventClick={handleEventClick}
                addButton={addButton}
            />

            {selectedEvent && (
                <EventDialog
                    onOpenChange={(open) => !open && setSelectedEvent(null)}
                    event={selectedEvent}
                />
            )}
        </>
    )
}

function eventDataTransform(input: EventInput): EventInput {
    input.color = input['X-COLOR'] || ''
    return input;
}
