mod keycodes;
use core_foundation::base::{
    Boolean,
    CFIndex,
    CFRange,
};
// use core_graphics::sys::CGEvent;
pub use keycodes::*;

use core_foundation::{
    runloop::CFRunLoop,
    // *,
};

use core_foundation::string::{
    kCFStringEncodingUTF8,
    CFString,
    CFStringGetBytes,
    CFStringGetCStringPtr,
    CFStringGetLength,
    CFStringRef,
};

use std::borrow::Borrow;
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

// fn to_string(string_ref: CFStringRef) -> String {
//     // reference: https://github.com/servo/core-foundation-rs/blob/355740/core-foundation/src/string.rs#L49
//     unsafe {
//         let char_ptr = CFStringGetCStringPtr(string_ref, kCFStringEncodingUTF8);
//         if !char_ptr.is_null() {
//             let c_str = std::ffi::CStr::from_ptr(char_ptr);
//             return String::from(c_str.to_str().unwrap());
//         }

//         let char_len = CFStringGetLength(string_ref);

//         let mut bytes_required: CFIndex = 0;
//         CFStringGetBytes(
//             string_ref,
//             CFRange {
//                 location: 0,
//                 length: char_len,
//             },
//             kCFStringEncodingUTF8,
//             0,
//             false as Boolean,
//             std::ptr::null_mut(),
//             0,
//             &mut bytes_required,
//         );

//         // Then, allocate the buffer and actually copy.
//         let mut buffer = vec![b'\x00'; bytes_required as usize];

//         let mut bytes_used: CFIndex = 0;
//         CFStringGetBytes(
//             string_ref,
//             CFRange {
//                 location: 0,
//                 length: char_len,
//             },
//             kCFStringEncodingUTF8,
//             0,
//             false as Boolean,
//             buffer.as_mut_ptr(),
//             buffer.len() as CFIndex,
//             &mut bytes_used,
//         );

//         return String::from_utf8_unchecked(buffer);
//     }
// }

fn to_string(string_ref: CFStringRef) -> &'static str {
    // reference: https://github.com/servo/core-foundation-rs/blob/355740/core-foundation/src/string.rs#L49
    unsafe {
        let char_ptr = CFStringGetCStringPtr(string_ref, kCFStringEncodingUTF8);
        assert!(!char_ptr.is_null());
        let c_str = std::ffi::CStr::from_ptr(char_ptr);
        c_str.to_str().unwrap()
    }
}

unsafe fn front_most_application() {
    use cocoa::base::id;
    // use objc::class;
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    let front_app: id = msg_send![workspace, frontmostApplication];
    let bundle_id: CFStringRef = msg_send![front_app, bundleIdentifier];
    // bundleID
    let s = to_string(bundle_id);
    println!("{:?}", s);
    // let z = CFString::from_static_string("com.apple.quicklook.qlmanage");
    // println!("{:?}", &z == *bundle_id);
    // let s: std::borrow::Cow<'_, str> = bundleID.into();
    // println!("{:?}", bundleID.as_str());
    // let z = bundleID.as_ref().unwrap();
    // let q= z.to_owned();
    // let t =z.borrow();
    //    let qq: std::borrow::Cow<'static, str> = z.into();
    // let r = std::borrow::Cow::from(bundleID);
    // let o = bundleID.to_owned();
    // let i = o.to_raw_parts();
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
