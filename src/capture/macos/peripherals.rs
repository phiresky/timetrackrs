use super::super::pc_common::{KEYSTROKES, MOUSE_CLICKS};
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopRun};
use core_graphics::event::{
    CGEvent, CGEventMask, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType::{self, KeyDown, LeftMouseDown, RightMouseDown}
};
use std::sync::atomic::Ordering;

/// Captures Keystrokes and Mouse Clicks, then increments `KEYSTROKES`/`MOUSE_CLICKS`.
pub fn capture_peripherals() {
    unsafe {
        let current = CFRunLoop::get_current();

        let event_tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![KeyDown, LeftMouseDown, RightMouseDown],
            callback,
        )
        .unwrap();

        let loop_source = event_tap.mach_port.create_runloop_source(0).unwrap();

        current.add_source(&loop_source, kCFRunLoopCommonModes);

        event_tap.enable();

        CFRunLoop::run_current();
    }
}

fn callback(_proxy: CGEventTapProxy, event_type: CGEventType, _event: &CGEvent) -> Option<CGEvent> {
    match event_type {
        LeftMouseDown | RightMouseDown => {
            MOUSE_CLICKS.fetch_add(1, Ordering::Relaxed);
        }
        KeyDown => {
            KEYSTROKES.fetch_add(1, Ordering::Relaxed);
        }
        _ => (),
    };

    None
}
