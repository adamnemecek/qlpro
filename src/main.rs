#[macro_use]
extern crate objc;

mod keycodes;
pub use keycodes::*;

use {
    core_foundation::{
        runloop::CFRunLoop,
        string::{
            kCFStringEncodingUTF8,
            // CFString,
            // CFStringGetBytes,
            CFStringGetCStringPtr,
            // CFStringGetLength,
            CFStringRef,
        },
    },

    core_graphics::event::{
        CGEvent,
        // CGEventFlags,
        CGEventTap,
        CGEventTapLocation,
        CGEventTapOptions,
        CGEventTapPlacement,
        CGEventType,
        EventField,
    },
    std::{
        cell::RefCell,
        process::{
            Child,
            Command,
        },
    },
};

fn quick_look(path: &str) -> Child {
    Command::new("/usr/bin/qlmanage")
        .args(&["-p", path])
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap()
}

fn open(path: &str) -> Child {
    Command::new("/usr/bin/open").args(&[path]).spawn().unwrap()
}

// use core_graphics::event::CGKeyCode;

impl KeyCode1 {
    fn from(e: &CGEvent) -> Self {
        let c = e.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
        Self::from_constant(c as i16)
    }
}

fn listen(f: impl Fn(&CGEvent) -> () + 'static) -> Result<CGEventTap<'static>, ()> {
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        move |_, _, ev| {
            f(ev);
            // Some(ev.to_owned())
            None
        },
    )?;

    let source = tap.mach_port.create_runloop_source(0)?;
    let r = CFRunLoop::get_current();
    r.add_source(&source, unsafe { core_foundation::runloop::kCFRunLoopCommonModes });
    tap.enable();

    Ok(tap)
}


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
    let paths: Vec<_> = paths
        .iter()
        .map(|x| {
            let mut p = cur_dir.clone();
            p.push(x);
            p
        })
        .collect();

    let non: Vec<_> = paths.iter().filter(|x| !x.exists()).collect();

    if !non.is_empty() {
        println!("{:?} don't exist", non);
        return None;
    }
    Some(paths)
}

impl Action {
    fn from(e: &CGEvent) -> Option<Self> {
        // let flags = e.get_flags();
        // let cmd = flags.contains(CGEventFlags::CGEventFlagCommand);
        let kc = KeyCode1::from(e);
        match kc {
            KeyCode1::P => Self::Prev.into(),
            KeyCode1::N => Self::Next.into(),
            KeyCode1::O | KeyCode1::Return => Self::Open.into(),
            KeyCode1::Q => Self::Exit.into(),
            KeyCode1::W => Self::Exit.into(),
            _ => None,
        }
    }
}

#[repr(isize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Dir {
    Prev = -1,
    Next = 1,
}

struct App {
    ql: Child,
    paths: Vec<std::path::PathBuf>,
    index: usize,
}

impl App {
    pub fn new(paths: Vec<std::path::PathBuf>) -> Self {
        assert!(!paths.is_empty());
        let path = &paths[0].to_string_lossy();
        println!("{:?}", path);
        let ql = quick_look(path);
        Self { ql, paths, index: 0 }
    }

    fn current_path<'a>(&'a self) -> std::borrow::Cow<'a, str> {
        self.paths[self.index].to_string_lossy()
    }

    fn move_by(&mut self, delta: Dir) {
        let new_index = self.index as isize + delta as isize;
        let indices = 0..self.paths.len() as isize;
        if !indices.contains(&new_index) {
            return;
        }
        let _ = self.ql.kill();

        self.index = new_index as _;
        let path = &self.current_path();
        println!("{:?}", path);
        self.ql = quick_look(path);
    }

    pub fn handle(&mut self, e: &CGEvent) {
        let is_preview = front_most_application() == "com.apple.quicklook.qlmanage";
        if !is_preview {
            return;
        }

        let Some(a) = Action::from(e) else {
            return;
        };
        match a {
            Action::Next => self.move_by(Dir::Next),
            Action::Prev => self.move_by(Dir::Prev),
            Action::Open => _ = open(&self.current_path()),
            Action::Exit => {
                _ = self.ql.kill();
                std::process::exit(0)
            }
        }
    }
}

fn main() {
    let Some(paths) = paths() else {
        println!("Usage: Pass in the list of pdfs");
        return;
    };

    // let mut app = App::new(paths);
    let app = std::rc::Rc::new(RefCell::new(App::new(paths)));
    let _tap = listen(move |e| {
        let mut a = app.as_ref().borrow_mut();
        a.handle(e);
    })
    .unwrap();

    CFRunLoop::run_current();
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