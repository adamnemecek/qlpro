#[macro_use]
extern crate objc;

mod keycodes;
pub use keycodes::*;

use core_foundation::{
    runloop::CFRunLoop,
    // *,
};

use core_foundation::string::{
    kCFStringEncodingUTF8,
    // CFString,
    // CFStringGetBytes,
    CFStringGetCStringPtr,
    // CFStringGetLength,
    CFStringRef,
};

use std::process::{
    Child,
    Command,
};

fn quick_look(path: &str) -> Child {
    Command::new("/usr/bin/qlmanage").args(&["-p", path]).spawn().unwrap()
}

fn open(path: &str) -> Child {
    Command::new("/usr/bin/open").args(&[path]).spawn().unwrap()
}

use core_graphics::event::{
    CGEvent,
    CGEventFlags,
    CGEventTap,
    CGEventTapLocation,
    CGEventTapOptions,
    CGEventTapPlacement,
    CGEventType,
    EventField,
};

// use core_graphics::event::CGKeyCode;

fn keycode(e: &CGEvent) -> KeyCode1 {
    let c = e.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    let z = crate::KeyCode1::from_constant(c as i16);
    z
}

fn listen(f: impl Fn(&CGEvent) -> () + 'static) -> Result<CGEventTap<'static>, ()> {
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        vec![CGEventType::KeyDown],
        move |_, _, ev| {
            f(ev);
            None
        },
    )?;

    let source = tap.mach_port.create_runloop_source(0)?;
    let r = CFRunLoop::get_current();
    r.add_source(&source, unsafe { core_foundation::runloop::kCFRunLoopCommonModes });
    tap.enable();

    Ok(tap)
}

// pub trait NSWorkspace {

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

fn front_most_application() -> &'static str {
    use cocoa::base::id;
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        let bundle_id: CFStringRef = msg_send![front_app, bundleIdentifier];
        // bundleID
        to_string(bundle_id)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]

enum Action {
    Next,
    Prev,
    Open,
    Exit,
}

fn paths() -> Option<Vec<std::path::PathBuf>> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        return None;
    }

    let cur_dir = std::env::current_dir().unwrap();

    Some(
        paths
            .iter()
            .map(|x| {
                let mut p = cur_dir.clone();
                p.push(x);
                p
            })
            .collect(),
    )
}

fn action(e: &CGEvent) -> Action {
    let flags = e.get_flags();

    let cmd = flags.contains(CGEventFlags::CGEventFlagCommand);
    let kc = keycode(e);
    match kc {
        KeyCode1::P => Action::Prev,
        KeyCode1::O | KeyCode1::Return => Action::Open,
        KeyCode1::N => Action::Next,
        KeyCode1::Q => Action::Exit,
        KeyCode1::W => Action::Exit,
        _ => unimplemented!(),
    }
}

struct App {
    p: Child,
}

impl App {
    pub fn new(paht: Vec<std::path::PathBuf>) -> Self {
        
        // Self {
        //     p: 
        // }
        todo!()
    }

    pub fn handle(&self,e: &CGEvent) {
        //
        let a = action(e);
        match a {
            Action::Next=> todo!(),
            Action::Prev=> todo!(),
            Action::Open=> todo!(),
            Action::Exit=> todo!(),
        }
        println!("{:?}", a);

    }
}

fn main() {
    // for e in  {
    //     println!("{}", e);
    // }
    let paths = paths().unwrap();

    let app = App::new(paths);

    let tap =
        listen(move |e| {
            app.handle(e);
            // if front_most_application() == "com.apple.quicklook.qlmanage" {
            // }
        })
        .unwrap();

    // quick_look("/Users/adamnemecek/adjoint/papers/Zhang2017.pdf");
    // use core_foundation::base::msg_sen
    // macro
    let q = front_most_application();
    println!("{}", q);

    // MyEnum::A;
    CFRunLoop::run_current();
}
