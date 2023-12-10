use core::{
    ffi::*,
    fmt, iter,
    ptr::{self, null_mut, NonNull},
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
use printf_compat::output;

//

#[no_mangle]
pub extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void {
    let Some(path) = as_rust_str(filename) else {
        eprintln!("fopen invalid path");
        return null_mut();
    };
    let Some(mode) = as_rust_str(mode) else {
        eprintln!("fopen invalid mode");
        return null_mut();
    };

    let path = format!("/{path}");

    let mut opts = OpenOptions::new();
    if mode.contains('r') {
        opts.read(true);
    }
    if mode.contains('w') {
        opts.write(true);
        opts.create(true);
    }
    if mode.contains('x') {
        opts.create_new(true);
    }

    match File::open(&path) {
        Ok(f) => unsafe { f.into_inner() }.0 as _,
        Err(err) => {
            eprintln!("fopen syscall error ({path}): {err}");
            match err {
                Error::NOT_A_FILE => {
                    unsafe { errno = 21 };
                }
                Error::NOT_FOUND => {
                    unsafe { errno = 2 };
                }
                _ => {
                    eprintln!("FIXME: fopen map error {}", err.as_str());
                    unsafe { errno = 255 };
                }
            };
            null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn ftell(stream: *const c_void) -> c_long {
    eprintln!("ftell syscall");

    let file = unsafe { File::new(FileDesc(stream as usize)) };

    let res = match file.metadata() {
        Ok(meta) => meta,
        Err(err) => {
            match err {
                _ => {
                    eprintln!("FIXME: ftell map error {}", err.as_str());
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
pub extern "C" fn fflush(_stream: *const c_void) -> c_int {
    // files are not buffered by default
    0
}

#[no_mangle]
pub extern "C" fn fseek(stream: *const c_void, offset: c_long, origin: c_int) -> c_int {
    // eprintln!("fseek syscall");

    if let Err(err) = libstd::sys::seek(FileDesc(stream as usize), offset as _, origin as _) {
        match err {
            _ => {
                eprintln!("FIXME: fseek map error {} {offset} {origin}", err.as_str());
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
    if size == 0 || count == 0 {
        return 0;
    }

    // eprintln!("fread syscall");

    let buf = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, size * count) };

    let file = unsafe { File::new(FileDesc(stream as usize)) };

    let read = match file.read(buf) {
        Ok(read) => read,
        Err(err) => {
            match err {
                _ => {
                    eprintln!("FIXME: fread map error {}", err.as_str());
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
pub extern "C" fn fwrite(
    ptr: *const c_void,
    size: c_size_t,
    count: c_size_t,
    stream: *const c_void,
) -> c_size_t {
    if size == 0 || count == 0 {
        return 0;
    }

    eprintln!("fwrite syscall");

    let buf = unsafe { slice::from_raw_parts(ptr as *const u8, size * count) };

    let file = unsafe { File::new(FileDesc(stream as usize)) };

    let written = match file.write(buf) {
        Ok(n) => n,
        Err(err) => {
            match err {
                _ => {
                    eprintln!("FIXME: fwrite map error {}", err.as_str());
                }
            };
            unsafe { errno = err.0 as _ };
            0
        }
    };

    unsafe { file.into_inner() };

    written as _
}

#[no_mangle]
pub extern "C" fn fclose(stream: *const c_void) -> c_int {
    unsafe { File::new(FileDesc(stream as usize)) };
    0
}

#[no_mangle]
pub extern "C" fn mkdir(path: *const c_char, _mode: u32) -> c_int {
    let Some(path) = as_rust_str(path) else {
        eprintln!("mkdir invalid path");
        return -1;
    };

    if let Err(err) = Dir::open(path) {
        match err {
            _ => {
                eprintln!("FIXME: mkdir map error {}", err.as_str());
            }
        };
        unsafe { errno = err.0 as _ };
        -1
    } else {
        0
    }
}

#[no_mangle]
unsafe extern "C" fn fprintf(stream: *const c_void, format: *const c_char, mut args: ...) -> c_int {
    vfprintf(stream, format, &mut *args.as_va_list())
}

#[no_mangle]
pub extern "C" fn putc(_character: c_int, _stream: *const c_void) -> c_int {
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
    eprintln!("run cmd {cmd}");

    0
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(
    stream: *const c_void,
    format: *const c_char,
    // mut args: ...
    args: &mut VaListImpl,
) -> c_int {
    let mut file = unsafe { File::new(FileDesc(stream as usize)) };

    struct Dbg(File);

    impl fmt::Write for Dbg {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            eprintln!("vfprintf: {s}");
            self.0.write_str(s)
        }
    }

    let mut f = Dbg(file);

    let res = printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut f));

    file = f.0;

    unsafe { file.into_inner() };

    res
}

#[no_mangle]
pub extern "C" fn rename() {
    unimplemented!()
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
    args: &mut VaListImpl,
    // mut arg: *const c_void,
    // mut args: ...
) -> c_int {
    let buffer = unsafe { slice::from_raw_parts_mut(s as *mut u8, n) };
    struct BufferWrite<'a> {
        buf: &'a mut [u8],
        at: usize,
    }

    buffer.fill(0);

    impl<'a> fmt::Write for BufferWrite<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            eprintln!("vsnprintf: {s}");

            let now = self.at;
            self.at += s.len();

            if let Some(end) = self.buf.get_mut(now..) {
                let min = end.len().min(s.len());
                end[..min].copy_from_slice(&s.as_bytes()[..min]);
            }

            // let min = (self.buf.len()).min(self.at + s.len());
            // self.buf[self.at..min].copy_from_slice(&s.as_bytes()[..min - self.at]);
            // self.at += min - self.at;

            // if self.at >= self.buf.len() {
            //     // println!("attempted buffer overflow");
            //     return Err(fmt::Error);
            // }

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
        let new = malloc(size);

        let old_size = unsafe { libstd::alloc::GLOBAL_ALLOC.size(alloc) };
        let min = old_size.min(size);
        let old_slice = unsafe { slice::from_raw_parts(ptr as *mut u8, old_size) };
        let new_slice = unsafe { slice::from_raw_parts_mut(new as *mut u8, size) };
        new_slice[..min].copy_from_slice(&old_slice[..min]);

        eprintln!("realloc old_size:{old_size} size:{size}");

        new
    } else {
        eprintln!("realloc nullptr");
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
pub extern "C" fn abs(n: c_int) -> c_int {
    n.abs()
}

#[no_mangle]
pub extern "C" fn fabs(x: c_double) -> c_double {
    libm::fabs(x)
}

#[no_mangle]
unsafe extern "C" fn printf(format: *const c_char, mut args: ...) -> c_int {
    // TODO: DIY this c formatting thing

    let stdout = &mut *STDOUT.lock();
    let res =
        unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(stdout)) };
    libstd::io::Write::flush(stdout).unwrap();

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
unsafe extern "C" fn snprintf(
    s: *mut c_char,
    n: c_size_t,
    format: *const c_char,
    // mut args: &mut VaListImpl,
    // mut arg: *const c_void,
    mut args: ...
) -> c_int {
    vsnprintf(s, n, format, &mut args.as_va_list())
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
    libstd::alloc::GLOBAL_ALLOC.alloc(size) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if let Some(alloc) = NonNull::new(ptr as *mut u8) {
        unsafe { libstd::alloc::GLOBAL_ALLOC.free(alloc) };
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(lhs: *const c_char, rhs: *const c_char) -> c_int {
    unsafe { strncmp(lhs, rhs, usize::MAX) }
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(lhs: *const c_char, rhs: *const c_char, num: c_size_t) -> c_int {
    let lhs = unsafe { c_str_iter(lhs) };
    let rhs = unsafe { c_str_iter(rhs) };

    for (l, r) in lhs.zip(rhs).take(num) {
        if l != r || l == 0 {
            return l as c_int - r as c_int;
        }
    }

    0
}

pub fn _strncmp_assert(lhs: &str, rhs: &str, n: usize, expected: i32) {
    let val = unsafe { strncmp(lhs.as_ptr() as _, rhs.as_ptr() as _, n) }.signum();
    assert_eq!(
        val, expected,
        "strncmp({lhs}, {rhs}, {n}) => {val}, expected: {expected}"
    );
}

pub fn _strncmp_test() {
    _strncmp_assert("a\0", "a\0", usize::MAX, 0);
    _strncmp_assert("a\0", "a1\0", usize::MAX, -1);
    _strncmp_assert("a1\0", "a\0", usize::MAX, 1);
    _strncmp_assert("\0", "\0", usize::MAX, 0);
    _strncmp_assert("test", "test", 4, 0);
    _strncmp_assert("test1", "test2", 5, -1);
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(lhs: *const c_char, rhs: *const c_char) -> c_int {
    strncasecmp(lhs, rhs, usize::MAX)
}

#[no_mangle]
pub unsafe extern "C" fn strncasecmp(
    lhs: *const c_char,
    rhs: *const c_char,
    num: c_size_t,
) -> c_int {
    let lhs = unsafe { c_str_iter(lhs) };
    let rhs = unsafe { c_str_iter(rhs) };

    for (l, r) in lhs.zip(rhs).take(num) {
        let l = (l as u8).to_ascii_lowercase() as c_int;
        let r = (r as u8).to_ascii_lowercase() as c_int;

        if l != r || l == 0 {
            return l - r;
        }
    }

    0
}

pub fn _strncasecmp_assert(lhs: &str, rhs: &str, n: usize, expected: i32) {
    let val = unsafe { strncasecmp(lhs.as_ptr() as _, rhs.as_ptr() as _, n) }.signum();
    assert_eq!(
        val, expected,
        "strncasecmp({lhs}, {rhs}, {n}) => {val}, expected: {expected}"
    );
}

pub fn _strncasecmp_test() {
    _strncasecmp_assert("a\0", "a\0", usize::MAX, 0);
    _strncasecmp_assert("a\0", "A\0", usize::MAX, 0);
    _strncasecmp_assert("a\0", "a1\0", usize::MAX, -1);
    _strncasecmp_assert("a\0", "A1\0", usize::MAX, -1);
    _strncasecmp_assert("a1\0", "a\0", usize::MAX, 1);
    _strncasecmp_assert("\0", "\0", usize::MAX, 0);
    _strncasecmp_assert("test", "test", 4, 0);
    _strncasecmp_assert("teSt", "tEsT", 4, 0);
    _strncasecmp_assert("test1", "test2", 5, -1);
    _strncasecmp_assert("test1", "Test2", 5, -1);
    _strncasecmp_assert("test", "TEST", 4, 0);
    _strncasecmp_assert("test", "yeet", 0, 0);
}

// iterate all chars in a c string including the null terminator
pub unsafe fn c_str_iter(mut str: *const c_char) -> impl Iterator<Item = c_char> {
    iter::from_fn(move || {
        let c = unsafe { *str };
        str = unsafe { str.byte_add(1) };
        (c != 0).then_some(c)
    })
    .chain([0])
}

#[no_mangle]
pub unsafe extern "C" fn strchr(str: *const c_char, character: c_int) -> *const c_char {
    let character = character as c_char;

    for (i, c) in c_str_iter(str).enumerate() {
        if c == character {
            return unsafe { str.add(i) };
        }
    }

    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn strrchr() {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "C" fn atoi(str: *const c_char) -> c_int {
    let mut iter = unsafe { c_str_iter(str) }
        .skip_while(|&c| isspace(c as _) != 0)
        .peekable();

    let mut neg = false;
    match iter.peek().unwrap() {
        0x2d => {
            // b'-'
            iter.next();
            neg = true;
        }
        0x2b => {
            // b'+'
            iter.next();
        }
        _ => {}
    }

    let mut res = 0;
    for digit in iter.take_while(|&c| isdigit(c as _) != 0) {
        res = 10 * res + b'0' as c_int - digit as c_int;
    }

    if neg {
        res
    } else {
        -res
    }

    // let Some(str) = as_rust_str(str) else {
    //     return 0;
    // };

    // let str = str.trim().trim_start_matches(|c| c == '+');
    // if str.is_empty() {
    //     return 0;
    // }

    // let str = str
    //     .find(|c: char| !c.is_digit(10))
    //     .and_then(|last| str.get(..last))
    //     .unwrap_or(str);

    // str.parse().unwrap()
}

fn _atoi_assert(lhs: &str, expected: i32) {
    let val = unsafe { atoi(lhs.as_ptr() as *const c_char) };
    assert_eq!(val, expected, "atoi({lhs}) => {val}, expected: {expected}");
}

pub fn _atoi_test() {
    _atoi_assert("\0", 0);
    _atoi_assert("  \0", 0);
    _atoi_assert("  1\0", 1);
    _atoi_assert("  1  \0", 1);
    _atoi_assert("  654  \0", 654);
    _atoi_assert("  654  ", 654);
    _atoi_assert(" 3d\0", 3);
    _atoi_assert("-3d\0", -3);
    _atoi_assert("a-3d\0", 0);
    _atoi_assert("+3d\0", 3);
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
pub extern "C" fn isdigit(c: c_int) -> c_int {
    (b'0' as c_int..=b'9' as c_int).contains(&c) as c_int
}

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    (c == 0x20 || c == 0x0c || c == 0x0a || c == 0x0d || c == 0x09 || c == 0x0b) as c_int
}

#[no_mangle]
pub extern "C" fn islower(c: c_int) -> c_int {
    (b'a' as c_int..=b'z' as c_int).contains(&c) as c_int
}

#[no_mangle]
pub extern "C" fn isupper(c: c_int) -> c_int {
    (b'A' as c_int..=b'Z' as c_int).contains(&c) as c_int
}

#[no_mangle]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if islower(c) != 0 {
        c & !0x20
    } else {
        c
    }

    // (character as u8).to_ascii_uppercase() as _
    // char::from_u32(character as _);
}

#[no_mangle]
pub extern "C" fn strdup(src: *const c_char) -> *mut c_char {
    let len = strlen(src);

    let dst = malloc(len + 1) as *mut c_char;
    strncpy(dst, src, len);

    assert_eq!(unsafe { *dst.byte_add(len) }, 0);

    dst
}

//

#[no_mangle]
#[used]
static mut errno: i32 = 0;

#[no_mangle]
#[used]
static mut stderr: usize = 0;

//

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
