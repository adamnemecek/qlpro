mod keycodes;
// use core_graphics::sys::CGEvent;
pub use keycodes::*;

use core_foundation::{
    runloop::CFRunLoop,
    // *,
};

use std::process::{
    Command,
    Stdio,
};

fn quick_look(path: &str) {
    let z = Command::new("/usr/bin/qlmanage").args(&["-p", path]).spawn().unwrap();
}

fn open(path: &str) {
    let z = Command::new("/usr/bin/open").args(&[path]).spawn().unwrap();
}

use core_graphics::event::{
    CGEvent,
    CGEventTap,
    CGEventTapLocation,
    CGEventTapOptions,
    CGEventTapPlacement,
    CGEventType,
    EventField,
};

use core_graphics::event::CGKeyCode;

fn keycode(e: &CGEvent) -> i64 {
    e.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
}

fn listen(f: impl Fn(CGEvent) -> () + 'static) -> Result<CGEventTap<'static>, ()> {
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        move |_, _, ev| {
            f(ev.to_owned());
            Some(ev.to_owned())
        },
    )?;

    let source = tap.mach_port.create_runloop_source(0)?;
    let r = CFRunLoop::get_current();
    r.add_source(&source, unsafe { core_foundation::runloop::kCFRunLoopCommonModes });
    tap.enable();

    Ok(tap)
}

// pub trait NSWorkspace {

// }
#[macro_use]
extern crate objc;

unsafe fn front_most_application() {
    use cocoa::base::{id};
    // use objc::class;
    let workspace: id =  msg_send![class!(NSWorkspace), sharedWorkspace];
    let frontApp: id =  msg_send![workspace, frontmostApplication] ;
    println!("{:?}", frontApp);
}

fn main() {
    // let tap = listen(|e| println!("{}", keycode(&e))).unwrap();

    // quick_look("/Users/adamnemecek/adjoint/papers/Zhang2017.pdf");
    // use core_foundation::base::msg_sen
    // macro
    unsafe { front_most_application() };
    
    // MyEnum::A;
    CFRunLoop::run_current();
}
