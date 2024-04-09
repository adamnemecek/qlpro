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
        CGEventTap,
        CGEventTapLocation,
        CGEventTapOptions,
        CGEventTapPlacement,
        CGEventType,
        EventField,
    },
    std::process::{
        Child,
        Command,
    },
};

mod keycodes;
pub use keycodes::*;

pub trait CGEventExt {
    fn key_code(&self) -> KeyCode;
}

impl CGEventExt for &CGEvent {
    #[inline]
    fn key_code(&self) -> KeyCode {
        let c: u16 = self.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as _;
        c.into()
    }
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

fn open(path: &std::path::Path) -> Child {
    Command::new("/usr/bin/open").args(&[path]).spawn().unwrap()
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn AXAPIEnabled() -> bool;
    pub fn AXIsProcessTrustedWithOptions(options: core_graphics::display::CFDictionaryRef) -> bool;
    pub static kAXTrustedCheckOptionPrompt: CFStringRef;
}

unsafe fn is_process_trusted() -> bool {
    use {
        core_foundation::{
            base::{
                FromVoid,
                ToVoid,
            },
            dictionary::CFMutableDictionary,
            number::CFNumber,
            string::CFString,
        },
        std::ffi::c_void,
    };

    let mut dict: CFMutableDictionary<CFString, CFNumber> = <_>::default();

    dict.add(
        &CFString::from_void(kAXTrustedCheckOptionPrompt as *const c_void).to_owned(),
        &1i64.into(),
    );

    let app_has_permissions = AXIsProcessTrustedWithOptions(dict.into_untyped().to_void() as *const _);

    app_has_permissions
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

fn front_most_application() -> &'static str {
    use cocoa::base::id;
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        let bundle_id: CFStringRef = msg_send![front_app, bundleIdentifier];
        bundle_id.as_str()
    }
}

fn paths() -> Option<Vec<std::path::PathBuf>> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        return None;
    }

    let Ok(cwd) = std::env::current_dir() else { todo!() };

    let paths: Vec<_> = paths
        .iter()
        .map(|path| {
            let mut absolute_path = std::path::PathBuf::from(path);
            if !absolute_path.is_absolute() {
                absolute_path = cwd.join(path);
            }
            absolute_path
        })
        .collect();

    let non: Vec<_> = paths.iter().filter(|x| !x.exists()).collect();

    if !non.is_empty() {
        println!("{:?} don't exist", non);
        return None;
    }
    Some(paths)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Action {
    Next,
    Prev,
    Open,
    Exit,
    Important,
}

impl Action {
    fn from(e: &CGEvent) -> Option<Self> {
        // let flags = e.get_flags();
        // let cmd = flags.contains(CGEventFlags::CGEventFlagCommand);
        let kc = e.key_code();
        match kc {
            KeyCode::P => Self::Prev,
            KeyCode::N => Self::Next,
            KeyCode::O | KeyCode::Return => Self::Open,
            KeyCode::Q | KeyCode::W => Self::Exit,
            KeyCode::I => Self::Important,
            _ => return None,
        }
        .into()
    }
}

#[repr(isize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Dir {
    Prev = -1,
    Next = 1,
}

fn is_preview_active() -> bool {
    front_most_application() == "com.apple.quicklook.qlmanage"
}

struct App {
    ql: Child,
    paths: Vec<std::path::PathBuf>,
    cursor: usize,
}

impl App {
    pub fn new(paths: Vec<std::path::PathBuf>) -> Self {
        assert!(!paths.is_empty());

        let ql = quick_look(&paths[0]);
        Self { ql, paths, cursor: 0 }
    }

    fn current_path<'a>(&'a self) -> &std::path::Path {
        &self.paths[self.cursor]
    }

    fn move_cursor_by(&mut self, delta: Dir) {
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

    // fn important(&self) {
    // let current = self.current_path().ancestors().last().unwrap().as_os_str().as_;
    // println!("{}", );
    // let last = &self.current_path().components().as_path();
    // use std::os::unix::fs::symlink;
    // symlink(last.into(),last.into());//"_important".into());
    // symlink("/origin_does_not_exist/", link_path).unwrap();
    // }

    pub fn handle(&mut self, e: &CGEvent) -> bool {
        if !is_preview_active() {
            return false;
        }

        let Some(a) = Action::from(e) else {
            return false;
        };
        match a {
            Action::Next => self.move_cursor_by(Dir::Next),
            Action::Prev => self.move_cursor_by(Dir::Prev),
            Action::Open => _ = open(&self.current_path()),
            Action::Important => todo!(),
            Action::Exit => {
                _ = self.ql.kill();
                std::process::exit(0)
            }
        }
        true
    }
}

fn main() {
    let _ = unsafe { is_process_trusted() };
    // println!("premissins {}", a);

    let Some(paths) = paths() else {
        println!("Usage: Pass in the list of files");
        return;
    };

    // println!("{:?}", paths);
    //
    // signal_hook::flag::register(signal_hook::consts::SIGCHLD, Arc::clone(&term));
    //s signal_hook_registry::register(signal_hook_registry::, action)
    let app = std::rc::Rc::new(std::cell::RefCell::new(App::new(paths)));
    let _tap = listen(move |e| {
        let mut a = app.as_ref().borrow_mut();
        a.handle(e)
    })
    .unwrap();

    CFRunLoop::run_current();
}
