use std::collections::VecDeque;
use std::mem;
use std::ffi::{CString, c_void, CStr};
use libc::{c_char, c_uchar, c_ulong, c_uint, c_long};
use x11::xlib;
use x11::xtest;
use crate::error::{Result, Error};


/// Get Display.
fn open_display() -> Result<*mut xlib::Display> {
    let display = unsafe {
        xlib::XOpenDisplay(std::ptr::null())
    };

    match display.is_null() {
        false => Ok(display),
        _ => Err(Error::X11("XOpenDisplay failed".to_string()))
    }
}


/// Fetch window name by id
fn fetch_window_name(display: *mut xlib::Display, window: c_ulong) -> Option<String> {
    let mut name_ptr = Box::into_raw(Box::new(mem::MaybeUninit::<c_char>::uninit())) as *mut c_char;
    let status = unsafe {
        xlib::XFetchName(
            display,
            window,
            &mut name_ptr as *mut _,
        )
    };

    if status == 0 {
        return None;
    }

    let name = unsafe {
        CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        // std::str::from_utf8_unchecked(std::slice::from_raw_parts(name_ptr as *const u8, strlen(name_ptr)))
    };

    unsafe {
        xlib::XFree(name_ptr as *mut _ as *mut c_void)
    };

    Some(name)
}


/// Get list of children windows.
fn list_of_children(display: *mut xlib::Display, window: c_ulong) -> Vec<c_ulong> {
    let mut count = mem::MaybeUninit::<c_uint>::uninit();
    let mut children_ptr = Box::into_raw(Box::new(mem::MaybeUninit::<c_ulong>::uninit())) as *mut c_ulong;

    let root_id = unsafe {
        xlib::XQueryTree(
            display,
            window,
            mem::MaybeUninit::<c_ulong>::uninit().as_mut_ptr(),
            mem::MaybeUninit::<c_ulong>::uninit().as_mut_ptr(),
            &mut children_ptr as *mut _,
            count.as_mut_ptr(),
        )
    };

    if root_id == 0 {
        return Vec::new();
    }

    let children = unsafe {
        std::slice::from_raw_parts(children_ptr, count.assume_init() as usize)
    }.to_vec();

    unsafe {
        xlib::XFree(children_ptr as *mut _ as *mut c_void);
    };

    children
}


/// Get window id by name
fn find_window_id_by_name(display: *mut xlib::Display, window: c_ulong, name: String) -> Result<c_ulong> {
    let mut deq = VecDeque::from([window]);

    while let Some(current_window) = deq.pop_back() {
        if let Some(window_name) = fetch_window_name(display, current_window) {
            if window_name.eq(&name) {
                return Ok(current_window);
            }
        }

        deq.extend(list_of_children(display, current_window))
    }

    Err(Error::X11("Not found application".to_string()))
}


pub struct App {
    dpy: *mut xlib::Display,
    window: c_ulong,
}


impl App {
    pub fn attach(title: String) -> Result<Self> {
        let dpy = open_display()?;
        let window = find_window_id_by_name(
            dpy,
            unsafe {xlib::XRootWindow(dpy, xlib::XDefaultScreen(dpy))},
            title,
        )?;

        Ok(Self {
            dpy,
            window,
        })
    }

    pub fn activate(&self) {
        unsafe {xlib::XSetInputFocus(self.dpy, self.window, 0, 0)};

        std::thread::sleep(std::time::Duration::new(0, 50));
    }

    pub fn press(&self, key: c_ulong) {
        unsafe {
            let key_code = xlib::XKeysymToKeycode(self.dpy, key) as u32;

            xtest::XTestFakeKeyEvent(self.dpy, key_code, xlib::True, 0);
            xtest::XTestFakeKeyEvent(self.dpy, key_code, xlib::False, 0);
        }
    }

    pub fn move_cursor(&self) {
        unsafe {
            xtest::XTestFakeMotionEvent(self.dpy,0, 100, 200, 3);
        }
    }

    pub fn click(&self) {
        unsafe {
            xtest::XTestFakeButtonEvent(self.dpy, 3, xlib::True, 3);
            xtest::XTestFakeButtonEvent(self.dpy, 3, xlib::False, 3);
        }
    }

    pub fn close(&self) {
        let wm_protocols = CString::new("WM_PROTOCOLS").unwrap();
        let wm_delete_window = CString::new("WM_DELETE_WINDOW").unwrap();

        let mut event = unsafe {
            let mut data = xlib::ClientMessageData::new();
            data.set_long(
                0,
                xlib::XInternAtom(
                    self.dpy,
                    wm_delete_window.as_ptr() as *const c_char,
                    xlib::False,
                ) as c_long
            );
            data.set_long(1, 3);

            xlib::XEvent {
                client_message: xlib::XClientMessageEvent {
                    type_: xlib::ClientMessage,
                    serial: 0,
                    send_event: xlib::False,
                    display: self.dpy,
                    window: self.window,
                    message_type: xlib::XInternAtom(
                        self.dpy, wm_protocols.as_ptr() as *const c_char, xlib::True),
                    format: 32,
                    data,
                }
            }
        };

        unsafe {
            xlib::XSendEvent(self.dpy, self.window, xlib::False, xlib::NoEventMask, &mut event );
        }
    }

    fn key_code(&self, string: &str) -> c_uchar {
        let c_str = CString::new(string).unwrap();
        unsafe {
            xlib::XKeysymToKeycode(
                self.dpy,
                xlib::XStringToKeysym(c_str.as_ptr() as *const c_char),
            )
        }
    }
}


impl Drop for App {
    fn drop(&mut self) {
        unsafe { xlib::XCloseDisplay(self.dpy); }
    }
}
