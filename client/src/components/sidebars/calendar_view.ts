

class CalendarView extends HTMLElement {
    connectedCallback() {
        this.textContent = "Calendar View!"
    }
}

customElements.define('calendar-view', CalendarView)

export default CalendarView;