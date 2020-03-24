extern crate inotify;

use inotify::{EventMask, Inotify, WatchMask};
use std::ffi::{CString};
use std::os::raw::{c_char, c_int};
use std::string::String;

#[repr(C)]
pub enum INotifyCEventMask {
    CREATE,
    DELETE,
    MODIFY,
    UNHANDLED,
}

#[repr(C)]
pub struct INotifyCEvent {
    pub name: String,
    pub mask: INotifyCEventMask,
}

#[repr(C)]
pub struct ResultINotifyEventCTransport {
    pub is_ok: bool,
    pub err_msg: *mut c_char,
    pub err_len: c_int,
    pub event_array: *mut [INotifyCEvent],
    pub event_len: c_int,
}

#[repr(C)]
pub struct INotifyC {
    pub ptr: *mut Inotify,
}

#[repr(C)]
pub struct ResultINotifyCTransport {
    pub is_ok: bool,
    pub err_msg: *mut c_char,
    pub err_len: c_int,
    pub inotify: INotifyC,
}

impl From<Inotify> for INotifyC {
    fn from(inote: Inotify) -> INotifyC {
        INotifyC {
            ptr: Box::into_raw(Box::new(inote)),
        }
    }
}

impl ResultINotifyEventCTransport {
    // TODO cleanup this type signaure.
    // An ugly type signaure...
    fn new(events: std::vec::Vec<INotifyCEvent>) -> ResultINotifyEventCTransport {
        let s = events.as_slice();
        ResultINotifyEventCTransport {
            is_ok: true,
            err_msg: std::ptr::null_mut(),
            err_len: 0,
            event_array: Box::into_raw(Box::new(s[..])),
            event_len: events.len() as c_int,
        }
    }

    fn new_err(msg: &str) -> ResultINotifyEventCTransport {
        ResultINotifyEventCTransport {
            is_ok: false,
            err_msg: CString::new(msg).unwrap().into_raw(),
            err_len: msg.len() as c_int,
            event_array: Box::into_raw(Box::new([])),
            event_len: 0,
        }
    }
}

impl ResultINotifyCTransport {
    fn new(inote: Inotify) -> ResultINotifyCTransport {
        ResultINotifyCTransport {
            is_ok: true,
            err_msg: std::ptr::null_mut(),
            err_len: 0,
            inotify: INotifyC {
                ptr: Box::into_raw(Box::new(inote)),
            },
        }
    }

    fn new_err(msg: &str) -> ResultINotifyCTransport {
        ResultINotifyCTransport {
            is_ok: false,
            err_msg: CString::new(msg).unwrap().into_raw(),
            err_len: msg.len() as c_int,
            inotify: INotifyC {
                ptr: std::ptr::null_mut(),
            },
        }
    }
}

/// Allows us to convert from the rust style inotify::Event to a
/// C-FFI safe INotifyCEventMask.
impl From<EventMask> for INotifyCEventMask {
    fn from(m: inotify::EventMask) -> INotifyCEventMask {
        match m {
            EventMask::CREATE => INotifyCEventMask::CREATE,
            EventMask::DELETE => INotifyCEventMask::DELETE,
            EventMask::MODIFY => INotifyCEventMask::MODIFY,
            _ => INotifyCEventMask::UNHANDLED,
        }
    }
}

/// Allows us to convert from the rust style inotify::Event to a
/// C-FFI safe INotifyCEvent.
impl From<inotify::Event<String>> for INotifyCEvent {
    fn from(e: inotify::Event<String>) -> INotifyCEvent {
        INotifyCEvent {
            name: e.name.unwrap_or("none".to_string()),
            mask: INotifyCEventMask::from(e.mask),
        }
    }
}

/// C-FFI Functions
/// Initialize the passed inotify instance to watch `path`.
#[no_mangle]
pub unsafe extern "C" fn inotify_init(c_path: *mut c_char) -> ResultINotifyCTransport {
    if c_path.is_null() {
        return ResultINotifyCTransport::new_err("Nullptr passed as path");
    }

    let raw = CString::from_raw(c_path).into_string().unwrap();
    let path = std::path::Path::new(&raw);

    if let Ok(mut inotify) = Inotify::init() {
        let w = inotify.add_watch(
            path,
            WatchMask::CREATE | WatchMask::MODIFY | WatchMask::DELETE,
        );

        if w.is_err() {
            return ResultINotifyCTransport::new_err("Failed to watch directory");
        }

        return ResultINotifyCTransport::new(inotify);
    }

    return ResultINotifyCTransport::new_err("Failed to initialize inotify");
}

/// Blocking read on the inotify instance.
#[no_mangle]
pub unsafe extern "C" fn inotify_read_blocking(
    inotify_c: INotifyC,
) -> ResultINotifyEventCTransport {
    if inotify_c.ptr.is_null() {
        return ResultINotifyEventCTransport::new_err("Inotify instance is a nullptr.");
    }

    let mut inotify = Box::from_raw(inotify_c.ptr);
    let mut buffer = [0u8; 4096];
    let inotify_events = inotify.read_events_blocking(&mut buffer);

    if inotify_events.is_err() {
        return ResultINotifyEventCTransport::new_err("Error while reading events from inotify");
    }

    let events = inotify_events
        .unwrap()
        .filter(|e| {
            e.mask.contains(
                !EventMask::ISDIR | EventMask::CREATE | EventMask::MODIFY | EventMask::DELETE,
            )
        })
        .map(|e| {
            // TODO clean up.
            let s = e.name.unwrap().to_str().unwrap();
            INotifyCEvent {
                name: s.to_owned(),
                mask: INotifyCEventMask::from(e.mask),
            }
        }).collect::<Vec<_>>();

    ResultINotifyEventCTransport::new(events)
}

/// Release any resources related to our inotify result instance.
#[no_mangle]
pub unsafe extern "C" fn inotify_destroy(inotify_c: INotifyC) {
    if inotify_c.ptr.is_null() {
        return;
    }

    // TODO is this the right way to do this?
    drop(inotify_c.ptr);
}

#[no_mangle]
pub unsafe extern "C" fn inotify_destroy_error(err: *mut c_char) {
    if err.is_null() {
        return;
    }

    let c_string = CString::from_raw(err);

    drop(c_string);
}
