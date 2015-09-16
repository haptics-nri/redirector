extern crate libc;
use libc::{c_char, c_int, c_void, DIR};
use std::ffi::{CString, CStr, NulError};
use std::mem;
use std::str;
use std::process::exit;

extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[cfg(test)] macro_rules! test_println { ($($t:tt)*) => { println!($($t)*) } }
#[cfg(not(test))] macro_rules! test_println { ($($t:tt)*) => (()) }

macro_rules! attempt {
    ($code:expr) => {
        match $code {
            Ok(v) => v,
            Err(..) => exit(1)
        }
    }
}

fn with_c_str<S, O, F>(s: S, f: F) -> Result<O, NulError>
    where S: AsRef<str>,
          F: FnOnce(*const c_char) -> O
{
    let raii = try!(CString::new(s.as_ref()));
    Ok(f(raii.as_ptr()))
}

fn redirect(path: &str) -> String {
    if path.starts_with("/home/optoforce/newSRC/lib/fonts") {
        format!("{}{}", "/usr/share/fonts/X11/Type1", &path[32..])
    } else {
        path.to_string()
    }
}

macro_rules! get_fn {
    ($name:expr) => {{
        let fnptr = attempt!(unsafe {
            with_c_str($name, |s| dlsym(mem::transmute(-1i64), s))
        });
        test_println!("C {} =\t{:?}", $name, fnptr);
        unsafe { mem::transmute(fnptr) }
    }}
}

// notation:
//   abn: arg-before-name
//   abt: arg-before-type
//   aan: arg-after-type
//   aat: arg-after-type
//   "before" and "after" the arg-to-replace
// syntax is non-ideal, but see rust-lang/rust #26444
macro_rules! intercept {
    ($name:ident([$($abn:ident: $abt:ty),*]
                 $replace:ident
                 [$($aan:ident: $aat:ty),*]) -> $ret:ty) => {
        #[no_mangle]
        pub fn $name($($abn:$abt,)*
                     $replace: *const c_char
                     $(,$aan:$aat)*) -> $ret
        {
            let real: extern fn($($abt,)*
                                *const c_char
                                $(,$aat)*) -> $ret
                = get_fn!(stringify!($name));

            let requested = str::from_utf8(
                unsafe {
                    CStr::from_ptr($replace)
                }.to_bytes()).unwrap();
            let used = redirect(requested);

            attempt!(with_c_str(used, |s| real($($abn,)* s $(,$aan)*)))
        }
    }
}

intercept!(__xstat64([ver: c_int] path [buf: *mut libc::stat]) -> c_int);
intercept!(opendir([] name []) -> *const DIR);
intercept!(open([] pathname [flags: c_int]) -> c_int);

#[cfg(test)]
fn call_stat(file: &str) {
    let mut buf: libc::stat = unsafe { mem::zeroed() };
    println!("Rust stat =\t{:p}", &__xstat64);
    println!("stat buf =\t{:p}", &buf);
    println!("stat: {}", attempt!(with_c_str(file,
                                             |s| __xstat64(1, s, &mut buf))));
    println!("");
    println!("  File: ‘{}’", file);
    println!("  Size: {}\t\tBlocks: {}\t\tIO Block: {} \t\t{}",
             buf.st_size,
             buf.st_blocks,
             buf.st_blksize,
             match buf.st_rdev {
                 0 => "regular file".to_string(),
                 r => format!("rdev={}", r)
             });

    println!("Device: {:x}h/{}d\tInode: {}\t\tLinks: {}",
             buf.st_dev,
             buf.st_dev,
             buf.st_ino,
             buf.st_nlink);

    println!("Access: {:o}\t\tUid: {}\t\t\tGid: {}",
             buf.st_mode,
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
