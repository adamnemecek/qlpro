use core_foundation::{
    runloop::CFRunLoop,
    // *,
};
use core_graphics::event::{
    CGEventTap,
    CGEventTapLocation,
    CGEventTapOptions,
    CGEventTapPlacement,
    EventField,
    CGEventType,
};

fn listen() -> Result<CGEventTap<'static>, ()> {
    let tap =
        CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::KeyDown],
            |_, _, event| {
                println!(
                    "event {:?}",
                    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                );

                Some(event.to_owned())
            },
        )?;
    let source = tap.mach_port.create_runloop_source(0)?;

    let r = CFRunLoop::get_current();
    r.add_source(&source, unsafe { core_foundation::runloop::kCFRunLoopCommonModes });

    tap.enable();

    Ok(tap)
}

fn main() {
    let tap = listen().unwrap();

    CFRunLoop::run_current();
}
