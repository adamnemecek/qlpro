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
    // self.eventTap = CGEvent.tapCreate(
    //     tap: .cgSessionEventTap,
    //     place: .headInsertEventTap,
    //     options: .defaultTap,
    //     eventsOfInterest: CGEventMask(eventMask),
    //     callback: handleEventTap,
    //     userInfo: userInfo
    // )
    // let z = CGEventTapLocation::Session;
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
    // let run_loop_source = event_tap.get_run_loop_source();
    // let run_loop = core_foundation::run_loop::CFRunLoop::get_current();
    // run_loop.add_source(&run_loop_source, core_foundation::string::CFString::new("CGEventTap"));
    // run_loop.run();

    // core_foundation::runloop::CFRunLoopAddSource(
    //     CFRunLoop::get_current(),
    //     run_loop_source.as_CFTypeRef(),
    //     kCFRunLoopDefaultMode,
    // );
    // use core_foundation as cf;
    // let p = unsafe { cf::mach_port::CFMachPortCreateRunLoopSource(cf::base::kCFAllocatorDefault, &tap.mach_port, 0) };
    let a = tap.mach_port.create_runloop_source(0)?;

    use core_foundation as cf;
    let r = CFRunLoop::get_current();
    r.add_source(&a, unsafe { cf::runloop::kCFRunLoopCommonModes });

    tap.enable();

    Ok(tap)
}

fn main() {
    let tap = listen().unwrap();

    CFRunLoop::run_current();
}
