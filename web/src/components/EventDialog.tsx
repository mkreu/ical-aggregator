import type { EventApi } from '@fullcalendar/core'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'

interface EventDialogProps {
    event: EventApi
    onOpenChange: (open: boolean) => void
}

export function EventDialog({ event, onOpenChange }: EventDialogProps) {
    const formatDateTime = (date: Date | null) => {
        if (!date) return ''
        return date.toLocaleString('en-US', {
            weekday: 'short',
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
            hour12: false,
        })
    }

    return (
        <Dialog open={true} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{event.title || 'Event Details'}</DialogTitle>
                    <DialogDescription>
                        {event.start && (
                            <div className="mt-4 space-y-2">
                                <div>
                                    <strong>Start:</strong> {formatDateTime(event.start)}
                                </div>
                                {event.end && (
                                    <div>
                                        <strong>End:</strong> {formatDateTime(event.end)}
                                    </div>
                                )}
                                {event.extendedProps?.description && (
                                    <div className="mt-4">
                                        <strong>Description:</strong>
                                        <p className="mt-1 text-foreground">{event.extendedProps.description}</p>
                                    </div>
                                )}
                                {event.extendedProps['LOCATION'] && (
                                    <div>
                                        <strong>Location:</strong> {event.extendedProps['LOCATION']}
                                    </div>
                                )}
                                {event.extendedProps['DESCRIPTION'] && (
                                    <div dangerouslySetInnerHTML={{ __html: event.extendedProps['DESCRIPTION'] }} />
                                )}
                            </div>
                        )}
                    </DialogDescription>
                </DialogHeader>
                <DialogFooter showCloseButton />
            </DialogContent>
        </Dialog>
    )
}
