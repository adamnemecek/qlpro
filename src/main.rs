#[macro_use]
extern crate objc;

use {
    core_foundation::{
        runloop::CFRunLoop,
        string::{
            kCFStringEncodingUTF8,
            CFStringGetCStringPtr,
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

mod keycodes;
pub use keycodes::*;

fn quick_look(path: &std::path::Path) -> Child {
    // it makes sense to just like trim the last part since it can be passed in as both global and local
    println!("{}", &path.components().last().unwrap().as_os_str().to_str().unwrap());
    Command::new("/usr/bin/qlmanage")
        .args(&["-p", path.to_str().unwrap()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap()
}

fn open(path:  &std::path::Path) -> Child {
    Command::new("/usr/bin/open").args(&[path]).spawn().unwrap()
}

pub trait CGEventExt {
    fn key_code(&self) -> KeyCode;
}

impl CGEventExt for &CGEvent {
    fn key_code(&self) -> KeyCode {
        let c = self.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
        (c as u16).into()
    }
}

fn listen(f: impl Fn(&CGEvent) -> bool + 'static) -> Result<CGEventTap<'static>, ()> {
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        move |_, _, ev| {
            if f(ev) {
                None
            } else {
                Some(ev.to_owned())
            }
        },
    )?;

    let source = tap.mach_port.create_runloop_source(0)?;
    let r = CFRunLoop::get_current();
    r.add_source(&source, unsafe { core_foundation::runloop::kCFRunLoopCommonModes });
    tap.enable();

    Ok(tap)
}

pub trait CFStringExt {
    fn as_str(&self) -> &'static str;
}

impl CFStringExt for CFStringRef {
    fn as_str(&self) -> &'static str {
        // reference: https://github.com/servo/core-foundation-rs/blob/355740/core-foundation/src/string.rs#L49
        unsafe {
            let char_ptr = CFStringGetCStringPtr(*self, kCFStringEncodingUTF8);
            assert!(!char_ptr.is_null());
            let c_str = std::ffi::CStr::from_ptr(char_ptr);
            c_str.to_str().unwrap()
        }
    }
}

fn front_most_application() -> &'static str {
    use cocoa::base::id;
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        let bundle_id: CFStringRef = msg_send![front_app, bundleIdentifier];
        bundle_id.as_str()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]

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

    let Ok(cwd) = std::env::current_dir() else {
        todo!()
    };

    let paths: Vec<_> = paths.iter().map(|path| {
        let mut absolute_path = std::path::PathBuf::from(path);
        if !absolute_path.is_absolute() {
            absolute_path = cwd.join(path);
        }
        absolute_path
    }).collect();

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
        let kc = e.key_code();
        match kc {
            KeyCode::P => Self::Prev.into(),
            KeyCode::N => Self::Next.into(),
            KeyCode::O | KeyCode::Return => Self::Open.into(),
            KeyCode::Q | KeyCode::W => Self::Exit.into(),
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
    cursor: usize,
}

// #[derive(Debug)]
// struct FileLoc(pub String, pub std::path::PathBuf);

impl App {
    pub fn new(paths: Vec<std::path::PathBuf>) -> Self {
        assert!(!paths.is_empty());

        let ql = quick_look(&paths[0]);
        Self { ql, paths, cursor: 0 }
    }

    fn current_path<'a>(&'a self) -> &std::path::Path {
        &self.paths[self.cursor]
    }

    fn move_by(&mut self, delta: Dir) {
        let new_cursor = self.cursor as isize + delta as isize;
        let indices = 0..self.paths.len() as isize;
        if !indices.contains(&new_cursor) {
            return;
        }
        let _ = self.ql.kill();

        self.cursor = new_cursor as _;
        let path = &self.current_path();
        self.ql = quick_look(&path);
    }

    pub fn handle(&mut self, e: &CGEvent) -> bool {
        let is_preview = front_most_application() == "com.apple.quicklook.qlmanage";
        if !is_preview {
            return false;
        }

        let Some(a) = Action::from(e) else {
            return false;
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
        true
    }
}

fn main() {
    // let mut z = std::path::PathBuf::new();
    // z.push("/Users/adamnemecek/adjoint/papers/Zhang2017.pdf");
    // println!("{:?}", z);

    // let c = z.components().last();
    // println!("{:?}", c);
    // return;
    // let p = z.as_path().components();

    let Some(paths) = paths() else {
        println!("Usage: Pass in the list of files");
        return;
    };

    // println!("{:?}", paths);
    //
    let app = std::rc::Rc::new(RefCell::new(App::new(paths)));
    let _tap = listen(move |e| {
        let mut a = app.as_ref().borrow_mut();
        a.handle(e)
    })
    .unwrap();

    CFRunLoop::run_current();
}
