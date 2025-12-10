package web

import "fmt"

var (
	events = map[string][]func([]string){}
)

func EventOn(event string, on func([]string)) {
	events[event] = append(events[event], on)
}

func EventOff(event string, on func([]string)) {
	handlers := events[event]
	for i, h := range handlers {
		if fmt.Sprintf("%p", h) == fmt.Sprintf("%p", on) {
			events[event] = append(handlers[:i], handlers[i+1:]...)
			return
		}
	}
}

func EventEmit(event string, args []string) {
	for _, fn := range events[event] {
		fn(args)
	}
}
