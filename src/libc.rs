use core::{
    alloc::Layout,
    cmp::Ordering,
    ffi::*,
    fmt,
    ptr::{null_mut, NonNull},
    slice,
    str::from_utf8,
};

use alloc::format;
use libstd::{
    eprintln,
    fs::{Dir, File, OpenOptions, STDOUT},
    print, println,
    sys::{err::Error, fs::FileDesc},
};
use printf_compat::{argument::Argument, output};

//

#[no_mangle]
pub extern "C" fn fopen(filename: *const c_char, _mode: *const c_char) -> *mut c_void {
    let Some(path) = as_rust_str(filename) else {
        println!("mkdir invalid path");
        return null_mut();
    };

    let path = format!("/{path}");

    match File::open(&path) {
        Ok(f) => unsafe { f.into_inner() }.0 as _,
        Err(err) => {
            match err {
                _ => {
                    println!("FIXME: map error {}", err.as_str());
                }
            };
            unsafe { errno = err.0 as _ };
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "C" fn fprintf(stream: *const c_void, format: *const c_char, mut args: ...) -> c_int {
    vfprintf(stream, format, &mut *args.as_va_list())
}

#[no_mangle]
pub extern "C" fn putc(character: c_int, stream: *const c_void) -> c_int {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn ftell(stream: *const c_void) -> c_long {
    let mut file = unsafe { File::new(FileDesc(stream as usize)) };

    let res = match file.metadata() {
        Ok(meta) => meta,
        Err(err) => {
            match err {
                _ => {
                    println!("FIXME: map error {}", err.as_str());
                }
            };
            unsafe { errno = err.0 as _ };
            return -1;
        }
    };

    unsafe { file.into_inner() };

    res.position as _
}

#[no_mangle]
pub extern "C" fn fwrite(
    ptr: *const c_void,
    size: c_size_t,
    count: c_size_t,
    stream: *const c_void,
) -> c_long {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn remove() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn system(cmd: *const c_char) -> c_int {
    let Some(cmd) = as_rust_str(cmd) else {
        return 1;
    };

    // print zenity msg to console
    println!("run cmd {cmd}");

    0
}

#[no_mangle]
pub extern "C" fn fflush(stream: *const c_void) -> c_int {
    // files are not buffered by default
    0
}

#[no_mangle]
pub extern "C" fn fseek(stream: *const c_void, offset: c_long, origin: c_int) -> c_int {
    if let Err(err) = libstd::sys::seek(FileDesc(stream as usize), offset as _, origin as _) {
        match err {
            _ => {
                println!("FIXME: map error {}", err.as_str());
            }
        };
        unsafe { errno = err.0 as _ };
        -1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn fread(
    ptr: *mut c_void,
    size: c_size_t,
    count: c_size_t,
    stream: *const c_void,
) -> c_size_t {
    let buf = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, size * count) };

    let mut file = unsafe { File::new(FileDesc(stream as usize)) };

    let read = match file.read(buf) {
        Ok(read) => read,
        Err(err) => {
            match err {
                _ => {
                    println!("FIXME: map error {}", err.as_str());
                }
            };
            unsafe { errno = err.0 as _ };
            0
        }
    };

    unsafe { file.into_inner() };

    read / size
}

#[no_mangle]
pub extern "C" fn mkdir(path: *const c_char, mode: u32) -> c_int {
    let Some(path) = as_rust_str(path) else {
        println!("mkdir invalid path");
        return -1;
    };

    if let Err(err) = Dir::open(path) {
        match err {
            _ => {
                println!("FIXME: map error {}", err.as_str());
            }
        };
        unsafe { errno = err.0 as _ };
        -1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(
    stream: *const c_void,
    format: *const c_char,
    // mut args: ...
    mut args: &mut VaListImpl,
) -> c_int {
    let mut file = unsafe { File::new(FileDesc(stream as usize)) };

    let res = printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut file));

    unsafe { file.into_inner() };

    res
}

#[no_mangle]
pub extern "C" fn rename() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn fclose(stream: *const c_void) -> c_int {
    unsafe { File::new(FileDesc(stream as usize)) };
    0
}

#[no_mangle]
pub extern "C" fn sscanf() {
    unimplemented!()
}

#[no_mangle]
unsafe extern "C" fn vsnprintf(
    s: *mut c_char,
    n: c_size_t,
    format: *const c_char,
    mut args: &mut VaListImpl,
    // mut arg: *const c_void,
    // mut args: ...
) -> c_int {
    let mut buffer = unsafe { slice::from_raw_parts_mut(s as *mut u8, n) };
    struct BufferWrite<'a> {
        buf: &'a mut [u8],
        at: usize,
    };

    buffer.fill(0);

    impl<'a> fmt::Write for BufferWrite<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            if self.buf.len() < s.len() + self.at {
                // println!("attempted buffer overflow");
                return Err(fmt::Error);
            }

            self.buf[self.at..self.at + s.len()].copy_from_slice(s.as_bytes());
            self.at += s.len();

            Ok(())
        }
    }

    // let mut test = [0u8; 24];
    // use core::fmt::Write;
    // let mut s = BufferWrite {
    //     buf: &mut test,
    //     at: 0,
    // };
    // println!("{:?}", from_utf8(&test));

    let res = unsafe {
        printf_compat::format(
            format,
            args.as_va_list(),
            output::fmt_write(&mut BufferWrite { buf: buffer, at: 0 }),
        )
    };

    assert!(*buffer.last().unwrap() == 0);

    res
}

#[no_mangle]
pub extern "C" fn realloc(ptr: *mut c_void, size: c_size_t) -> *mut c_void {
    if let Some(alloc) = NonNull::new(ptr as *mut u8) {
        let old_size = unsafe { libstd::alloc::GLOBAL_ALLOC.size(alloc) };
        let new = malloc(size);

        let min = old_size.min(size);
        let old_slice = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, old_size) };
        let new_slice = unsafe { slice::from_raw_parts_mut(new as *mut u8, size) };
        new_slice[..min].copy_from_slice(&old_slice[..min]);

        new
    } else {
        malloc(size)
    }
}

#[no_mangle]
pub extern "C" fn calloc(num: c_size_t, size: c_size_t) -> *mut c_void {
    if num * size == 0 {
        return null_mut();
    }

    let ptr = malloc(num * size);

    if !ptr.is_null() {
        let slice = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, num * size) };
        slice.fill(0);
    }

    ptr
}

#[no_mangle]
pub extern "C" fn puts(str: *const c_char) -> c_int {
    extern "C" {
        fn strlen(str: *const c_char) -> c_size_t;
    }
    let len = unsafe { strlen(str) };

    let str = unsafe { slice::from_raw_parts(str as *const u8, len) };
    let str = from_utf8(str).unwrap();

    println!("{str} {len}");

    0
}

#[no_mangle]
pub extern "C" fn abs() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn fabs() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn isspace() {
    unimplemented!()
}

#[no_mangle]
unsafe extern "C" fn printf(format: *const c_char, mut args: ...) -> c_int {
    // TODO: DIY this c formatting thing

    let stdout = &mut *STDOUT.lock();
    let res =
        unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(stdout)) };
    libstd::io::Write::flush(stdout);

    res

    // let format = as_rust_str(format);
    // TODO: printf is obv kinda useless without the va_args
    // print!("{format}");

    // let mut expects_f = false;
    // for char in format.chars() {
    //     if char != '%' ^ !expects_f {
    //         print!("{char}");
    //     }

    //     match char {
    //         '%' => expects_f = true,
    //         ''
    //     }
    // }

    // args.arg();
}

#[no_mangle]
pub extern "C" fn snprintf() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn putchar(character: c_int) -> c_int {
    if let Some(c) = char::from_u32(character as _) {
        print!("{c}");
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn exit() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn __stack_chk_fail() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn malloc(size: c_size_t) -> *mut c_void {
    unsafe { libstd::alloc::GLOBAL_ALLOC.alloc(size) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn free(ptr: *mut c_void) {
    if let Some(alloc) = NonNull::new(ptr as *mut u8) {
        unsafe { libstd::alloc::GLOBAL_ALLOC.free(alloc) };
    }
}

#[no_mangle]
pub extern "C" fn strcmp(mut lhs: *const c_char, mut rhs: *const c_char) -> c_int {
    strncmp(lhs, rhs, usize::MAX)
}

#[no_mangle]
pub extern "C" fn strncmp(mut lhs: *const c_char, mut rhs: *const c_char, num: c_size_t) -> c_int {
    for i in 0..num {
        let _lhs = unsafe { *lhs };
        let _rhs = unsafe { *rhs };
        lhs = unsafe { lhs.add(1) };
        rhs = unsafe { rhs.add(1) };

        let res = _lhs - _rhs;

        if _lhs == 0 || _rhs == 0 {
            return res as _;
        } else if res != 0 {
            return res as _;
        }
    }

    0
}

pub fn _strcmp_test() {
    let a = "apple\0";
    let b = "apple1\0";
    println!(
        "strcmp({a}, {b}) res: {}",
        strcmp(a.as_ptr() as _, b.as_ptr() as _)
    );
    let a = "apple1\0";
    let b = "apple\0";
    println!(
        "strcmp({a}, {b}) res: {}",
        strcmp(a.as_ptr() as _, b.as_ptr() as _)
    );
    let a = "apple\0";
    let b = "tests\0";
    println!(
        "strcmp({a}, {b}) res: {}",
        strcmp(a.as_ptr() as _, b.as_ptr() as _)
    );
    let a = "apple\0";
    let b = "atest\0";
    println!(
        "strcmp({a}, {b}) res: {}",
        strcmp(a.as_ptr() as _, b.as_ptr() as _)
    );
    let a = "\0";
    let b = "\0";
    println!(
        "strcmp({a}, {b}) res: {}",
        strcmp(a.as_ptr() as _, b.as_ptr() as _)
    );
}

#[no_mangle]
pub extern "C" fn strchr(mut str: *const c_char, character: c_int) -> *const c_char {
    loop {
        let now = unsafe { *str };
        if now as c_int == character || now == 0 {
            return str;
        }

        str = unsafe { str.add(1) };
    }
}

#[no_mangle]
pub extern "C" fn strrchr() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn atoi(str: *const c_char) -> c_int {
    let Some(str) = as_rust_str(str) else {
        return 0;
    };

    let str = str.trim().trim_start_matches(|c| c == '+');
    if str.is_empty() {
        return 0;
    }

    let str = str
        .find(|c: char| !c.is_digit(10))
        .and_then(|last| str.get(..last))
        .unwrap_or(str);

    str.parse().unwrap()
}

#[no_mangle]
pub extern "C" fn strncpy(
    mut dst: *mut c_char,
    mut src: *const c_char,
    num: c_size_t,
) -> *mut c_char {
    let mut end = false;

    for _ in 0..num {
        if unsafe { *src } == 0 {
            end = true;
        }

        if end {
            unsafe {
                *dst = 0;
            }
        } else {
            unsafe {
                *dst = *src;
            }
        }

        dst = unsafe { dst.add(1) };
        src = unsafe { src.add(1) };
    }

    dst
}

#[no_mangle]
pub extern "C" fn toupper(character: c_int) -> c_int {
    (character as u8).to_ascii_uppercase() as _
    // char::from_u32(character as _);
}

#[no_mangle]
pub extern "C" fn strdup(src: *const c_char) -> *mut c_char {
    let len = strlen(src);

    let dst = malloc(len) as *mut c_char;
    strncpy(dst, src, len);

    dst
}

#[no_mangle]
pub extern "C" fn strcasecmp(mut lhs: *const c_char, mut rhs: *const c_char) -> c_int {
    strncasecmp(lhs, rhs, usize::MAX)
}

#[no_mangle]
pub extern "C" fn strncasecmp(
    mut lhs: *const c_char,
    mut rhs: *const c_char,
    num: c_size_t,
) -> c_int {
    for i in 0..num {
        let _lhs = (unsafe { *lhs } as u8).to_ascii_lowercase() as i8;
        let _rhs = (unsafe { *rhs } as u8).to_ascii_lowercase() as i8;
        lhs = unsafe { lhs.add(1) };
        rhs = unsafe { rhs.add(1) };

        let res = _lhs.saturating_sub(_rhs);

        if _lhs == 0 || _rhs == 0 {
            return res as _;
        } else if res != 0 {
            return res as _;
        }
    }

    0
}

//

#[no_mangle]
#[used]
static mut errno: i32 = 0;

#[no_mangle]
#[used]
static mut stderr: usize = 0;

fn strlen(str: *const c_char) -> usize {
    extern "C" {
        fn strlen(str: *const c_char) -> c_size_t;
    }
    unsafe { strlen(str) }
}

#[track_caller]
fn as_rust_str<'a>(str: *const c_char) -> Option<&'a str> {
    let len = strlen(str);

    let str = unsafe { slice::from_raw_parts(str as *const u8, len) };
    match from_utf8(str) {
        Ok(s) => Some(s),
        Err(err) => {
            let valid = from_utf8(&str[..err.valid_up_to()]).unwrap();
            eprintln!("{valid:?} invalid {}\n{err}", str[err.valid_up_to()]);
            None
        }
    }
}
