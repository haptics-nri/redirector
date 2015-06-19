extern crate libc;
use libc::{c_char, c_int, c_void, DIR};
use std::ffi::{CString, CStr};
use std::mem;
use std::str;

extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[cfg(test)] macro_rules! test_println { ($($t:tt)*) => { println!($($t)*) } }
#[cfg(not(test))] macro_rules! test_println { ($($t:tt)*) => (()) }

fn redirect(path: &str) -> String {
    if path.starts_with("/home/optoforce/newSRC/lib/fonts") {
        format!("{}{}", "/usr/share/fonts/X11/Type1", &path[32..])
    } else {
        path.to_string()
    }
}

macro_rules! get_fn {
    ($name:expr) => {{
        let fnptr = unsafe { dlsym(mem::transmute(-1i64), CString::new($name).unwrap().as_ptr()) };
        test_println!("C {} =\t{:?}", $name, fnptr);
        unsafe { mem::transmute(fnptr) }
    }}
}

/*
// notation:
//   abn: arg-before-name
//   abt: arg-before-type
//   aan: arg-after-type
//   aat: arg-after-type
//   "before" and "after" the arg-to-replace
macro_rules! intercept {
    ($name:ident($($abn:ident: $abt:ty),*; $replace:ident; $($aan:ident: $aat:ty),*) -> $ret:ty) => {
        #[no_mangle]
        pub fn $name($($abn:$abt,)* $replace: *const c_char $(,$aan:$aat)*) -> $ret {
            let real: extern fn($($abt,)* *const c_char $(,$aat)*) -> $ret = get_fn!(stringify!($name));

            let requested = str::from_utf8(unsafe { CStr::from_ptr($replace) }.to_bytes()).unwrap();
            let used = redirect(requested);

            real($($abn,)* $replace $(,$aan)*)
        }
    }
}

intercept!(__xstat64(ver: c_int; path; buf: *mut libc::stat) -> c_int);
intercept!(opendir(; name ;) -> *const DIR);
*/

#[no_mangle]
pub fn __xstat64(ver: c_int, path: *const c_char, buf: *mut libc::stat) -> c_int {
    let stat_fn: extern fn(c_int, *const c_char, *mut libc::stat) -> c_int = get_fn!("__xstat64");

    let requested_path = str::from_utf8(unsafe { CStr::from_ptr(path) }.to_bytes()).unwrap();
    let used_path = redirect(requested_path);

    test_println!("request __xstat64({}, \"{}\", {:?})", ver, requested_path, buf);
    test_println!("calling __xstat64({}, \"{}\", {:?})", ver, used_path, buf);

    stat_fn(ver, CString::new(used_path).unwrap().as_ptr(), buf)
}

#[no_mangle]
pub fn opendir(name: *const c_char) -> *const DIR {
    let opendir_fn: extern fn(*const c_char) -> *const DIR = get_fn!("opendir");

    let requested_path = str::from_utf8(unsafe { CStr::from_ptr(name) }.to_bytes()).unwrap();
    let used_path = redirect(requested_path);

    test_println!("request opendir(\"{}\")", requested_path);
    test_println!("calling opendir(\"{}\")", used_path);

    opendir_fn(CString::new(used_path).unwrap().as_ptr())
}

#[no_mangle]
pub fn open(pathname: *const c_char, flags: c_int) -> c_int {
    let open_fn: extern fn(*const c_char, c_int) -> c_int = get_fn!("open");

    let requested_path = str::from_utf8(unsafe { CStr::from_ptr(pathname) }.to_bytes()).unwrap();
    let used_path = redirect(requested_path);

    test_println!("request open(\"{}\", {})", requested_path, flags);
    test_println!("calling open(\"{}\", {})", used_path, flags);

    open_fn(CString::new(used_path).unwrap().as_ptr(), flags)
}

#[cfg(test)]
fn call_stat(file: &str) {
    let mut buf: libc::stat = unsafe { mem::zeroed() };
    println!("Rust stat =\t{:p}", &__xstat64);
    println!("stat buf =\t{:p}", &buf);
    println!("stat: {}", __xstat64(1, CString::new(file).unwrap().as_ptr(), &mut buf));
    println!("");
    println!("  File: ‘{}’", file);
    println!("  Size: {}\t\tBlocks: {}\t\tIO Block: {} \t\t{}", buf.st_size,
                                                                buf.st_blocks,
                                                                buf.st_blksize,
                                                                match buf.st_rdev {
                                                                    0 => "regular file".to_string(),
                                                                    r => format!("rdev={}", r)
                                                                });
    println!("Device: {:x}h/{}d\tInode: {}\t\tLinks: {}", buf.st_dev,
                                                          buf.st_dev,
                                                          buf.st_ino,
                                                          buf.st_nlink);
    println!("Access: {:o}\t\tUid: {}\t\t\tGid: {}", buf.st_mode,
                                               buf.st_uid,
                                               buf.st_gid);
    println!("Access: {}.{}", buf.st_atime, buf.st_atime_nsec);
    println!("Modify: {}.{}", buf.st_mtime, buf.st_mtime_nsec);
    println!("Change: {}.{}", buf.st_ctime, buf.st_ctime_nsec);
    println!("");
}

#[test]
fn test_stat() {
    call_stat("/etc/fstab");
    call_stat("/home/optoforce/newSRC/lib/fonts");
}

