use std::mem;
use std::ffi::{CString, c_void, CStr};
use std::collections::{VecDeque, HashSet};
use libc::{c_char, c_uchar, c_ulong, c_uint};
use rand::Rng;
use x11::keysym;
use x11::xlib;
use x11::xtest;
use lfuzz::error::{Result, Error};


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
fn find_window_id_by_name(display: *mut xlib::Display, window: c_ulong, name: &str) -> Result<c_ulong> {
    let mut deq = VecDeque::from([window]);

    while let Some(current_window) = deq.pop_back() {
        if let Some(window_name) = fetch_window_name(display, current_window) {
            if window_name.eq(name) {
                return Ok(current_window);
            }
        }

        deq.extend(list_of_children(display, current_window))
    }

    Err(Error::X11("Not found application".to_string()))
}


struct App {
    dpy: *mut xlib::Display,
    window: c_ulong,
}


impl App {
    fn attach(title: &str) -> Result<Self> {
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

    fn activate(&self) {
        unsafe {xlib::XSetInputFocus(self.dpy, self.window, 0, 0)};

        std::thread::sleep(std::time::Duration::new(0, 50));
    }

    fn press(&self, key: c_ulong) {
        unsafe {
            let key_code = xlib::XKeysymToKeycode(self.dpy, key) as u32;

            xtest::XTestFakeKeyEvent(self.dpy, key_code, xlib::True, 0);
            xtest::XTestFakeKeyEvent(self.dpy, key_code, xlib::False, 0);
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


fn fuzz(app: App, blacklist: HashSet<c_uint>, pred: u32) {
    let mut count = 0u32;
    let mut rng = rand::thread_rng();

    loop {
        let rand_value = rng.gen::<u16>() as u32;
        if blacklist.contains(&rand_value) {
            continue;
        }

        std::thread::sleep(std::time::Duration::new(0, 500));
        count += 1;

        println!("key press {:#x}", rand_value);
        app.press(rand_value as c_ulong);

        if count == pred {
            break;
        }
    }
}


fn black_list() -> HashSet<c_uint>{
    [
        keysym::XK_Super_L,
        keysym::XK_Win_L,
        keysym::XK_Win_R,
        keysym::XK_F1,
    ].iter().cloned().collect()
}


fn main() {
    let app = App::attach("Calculator").expect("Something went wrong");
    println!("Window id {:?}", app.window);

    app.activate();

    let mut blacklist = black_list();
    for key in 0x4a1..=0xffff {
        blacklist.insert(key);
    }

    fuzz(app, blacklist, 100u32);
}
