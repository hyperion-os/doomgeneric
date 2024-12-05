#![no_std]
#![feature(c_variadic, c_size_t, new_uninit, slice_as_chunks, const_mut_refs)]

//

use core::{ffi, ptr::NonNull, slice};

use alloc::{ffi::CString, string::String, vec::Vec};
use crossbeam::queue::SegQueue;
use hyperion_color::Color;
use libstd::{
    env::args,
    eprintln,
    fs::{File, OpenOptions},
    io::{stdin, BufReader},
    println,
    process::ExitCode,
    sync::Mutex,
    sys::{map_file, nanosleep, rename, timestamp, unmap_file, yield_now},
    thread::spawn,
};

use self::libc::{_atoi_test, _strlen_test, _strncasecmp_test, _strncmp_test};

//

extern crate alloc;

mod libc;

//

extern "C" {
    fn doomgeneric_Create(argc: ffi::c_int, argv: *const *const ffi::c_char) -> ffi::c_int;

    fn doomgeneric_Tick() -> ffi::c_int;
}

//

static FB: Mutex<Framebuffer> = Mutex::new(Framebuffer {
    width: 0,
    height: 0,
    pitch: 0,
    buf: None,
});

#[derive(Debug)]
struct Ev {
    key: u8,
    pressed: i32,
}

static KEYS: SegQueue<Ev> = SegQueue::new();
static FBO: Mutex<Option<(File, usize)>> = Mutex::new(None);

//

#[no_mangle]
extern "C" fn DG_Init() {}

#[no_mangle]
pub extern "C" fn exit(status: ffi::c_int) -> ! {
    eprintln!("EXIT {status}");

    if let Some((fbo, fbo_mapped)) = FBO.lock().take() {
        unmap_file(
            fbo.as_desc(),
            NonNull::new(fbo_mapped as *mut ()).unwrap(),
            0,
        )
        .expect("failed to unmap the fb");
    }

    ExitCode::from_raw(status).exit_process()
}

fn lazy_init() {
    if FBO.lock().is_some() {
        return;
    }

    spawn(|| {
        let mut stdin = stdin().lock();
        let mut buf = String::new();
        loop {
            buf.clear();
            if stdin.read_line(&mut buf).is_err() {
                continue;
            }
            if buf.is_empty() {
                eprintln!("EMPTY");
                panic!();
            }
            eprintln!("{buf:?}");

            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            struct KeyboardEventSer {
                // pub scancode: u8,
                state: u8,
                keycode: u8,
                unicode: Option<char>,
            }

            let Ok(ev) = serde_json::from_str::<KeyboardEventSer>(&buf.trim()) else {
                continue;
            };

            let pressed = if ev.state == 0 { 1 } else { 0 };

            // libstd::sys::log!("{:x}:", ev.keycode);

            let key = match ev.keycode {
                // 40 => 0xad, // W - up
                // 61 => 0xa0, // A - strafe left
                // 62 => 0xaf, // S - down
                // 63 => 0xa1, // D - strafe right
                103 => 0xae, // right
                101 => 0xac, // left
                88 => 0xad,  // up
                102 => 0xaf, // down
                84 => 0xa0,  // comma - strafe left
                85 => 0xa1,  // period - strafe right
                96 => 0xa2,  // space - use
                // 93 => 0xa3,       // lctrl - fire
                0 => 27,          // escape
                72 => 13,         // enter
                38 => 9,          // tab
                1 => 0x80 + 0x3b, // f1-12
                2 => 0x80 + 0x3c,
                3 => 0x80 + 0x3d,
                4 => 0x80 + 0x3e,
                5 => 0x80 + 0x3f,
                6 => 0x80 + 0x40,
                7 => 0x80 + 0x41,
                8 => 0x80 + 0x42,
                9 => 0x80 + 0x43,
                10 => 0x80 + 0x44,
                11 => 0x80 + 0x57,
                12 => 0x80 + 0x58,

                17 => b'0',
                18 => b'1',
                19 => b'2',
                20 => b'3',
                21 => b'4',
                22 => b'5',
                23 => b'6',
                24 => b'7',
                25 => b'8',
                26 => b'9',

                30 => 0x7f, // backspace
                34 => 0xff, // pause

                29 => 0x3d, // equals
                28 => 0x2d, // minus

                76 | 87 => 0x80 + 0x36, // r/lshift
                // 100 => 0x80 + 0x1d,     // rctrl
                93 | 100 => 0xa3,       // r/lctrl
                95 | 97 => 0x80 + 0x38, // r/l alt

                60 => 0x80 + 0x3a, // capslock
                // => 0x80+0x45, // numlock
                // => 0x80+0x46, // scrlock
                // => 0x80+0x59, // prtint screen

                //
                32 => 0x80 + 0x47, // home
                54 => 0x80 + 0x4f, // end
                33 => 0x80 + 0x49, // pg down
                55 => 0x80 + 0x51, // pg up
                // => 0x80+0x52, // insert
                53 => 0x80 + 0x53, // delete

                // TODO: keypad keys:
                // => 0,           // 0
                // => 0x80 + 0x4f, // 1, end
                // => 0xaf,        // 2, down
                // => 0x80 + 0x49, // 3, pg down
                // => 0xac,        // 4, left
                // => b'5',        // 5
                // => 0xae,        // 6, right
                // => 0x80 + 0x47, // 7, home
                // => 0xad,        // 8, up
                // => 0x80 + 0x51, // 9, pg up

                // => b'/', // divide
                // => b'+', // plus
                // => b'-', // minus
                // => b'*', // mult
                // => 0, // period
                // => 0x3d, // equals
                // => 13, // enter
                _ => {
                    if let Some(c) = ev.unicode {
                        if c.is_ascii() {
                            c as _
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            };

            // libstd::sys::log!("{:x} -> {:x}", ev.keycode, key);

            KEYS.push(Ev { key, pressed });
        }
    });

    let mut fb = FB.lock();

    let mut info = framebuffer_info();

    let fbo = OpenOptions::new()
        .write(true)
        .open("/dev/fb0")
        .expect("failed to open /dev/fb0");
    let meta = fbo.metadata().expect("failed to read fb file metadata");

    let fbo_mapped = map_file(fbo.as_desc(), None, meta.len, 0).expect("failed to map the fb");

    let buf = unsafe { slice::from_raw_parts_mut(fbo_mapped.as_ptr() as *mut u8, meta.len) };
    // let mut backbuf = Vec::leak(vec![0u8; buf.len()]);
    // info.buf = backbuf;
    info.buf = Some(buf);

    *fb = info;

    // keep the file open

    let mut lock = FBO.lock();
    *lock = Some((fbo, fbo_mapped.as_ptr() as usize));
}

#[no_mangle]
extern "C" fn DG_DrawFrame() {
    lazy_init();

    let mut fb = FB.lock();

    const DOOMGENERIC_RESX: usize = 640;
    const DOOMGENERIC_RESY: usize = 400;

    // const PITCH: usize = DOOMGENERIC_RESX * mem::size_of::<u32>();
    // const W: usize = DOOMGENERIC_RESX * mem::size_of::<u32>();
    // const H: usize = DOOMGENERIC_RESY * mem::size_of::<u32>();

    extern "C" {
        static DG_ScreenBuffer: *mut u32;
    }

    let dg_buf = unsafe { DG_ScreenBuffer };
    let dg_buf =
        unsafe { slice::from_raw_parts(dg_buf as *const _, DOOMGENERIC_RESX * DOOMGENERIC_RESY) };
    // let dg_buf = unsafe { slice::from_raw_parts(dg_buf as *const u8, PITCH * DOOMGENERIC_RESY) };

    for y in 0..DOOMGENERIC_RESY {
        for x in 0..DOOMGENERIC_RESX {
            let px = dg_buf[x + y * DOOMGENERIC_RESX];
            let c = Color::from_u32(px);
            fb.fill(x * 2, y * 2, 2, 2, Color::new(c.b, c.g, c.r));
        }
        // let spot = y * fb.pitch;
        // let dg_buf_slot = y * DOOMGENERIC_RESX;
        // fb.buf[spot..spot + PITCH].copy_from_slice(&dg_buf[dg_buf_slot..dg_buf_slot + PITCH])
    }

    yield_now();
}

#[no_mangle]
extern "C" fn DG_SleepMs(ms: u32) {
    nanosleep(ms as u64 * 1_000_000)
}

#[no_mangle]
extern "C" fn DG_GetTicksMs() -> u32 {
    (timestamp().unwrap() / 1_000_000) as u32
}

#[no_mangle]
extern "C" fn DG_GetKey(_pressed: *mut ffi::c_int, _doom_key: *mut ffi::c_uchar) -> ffi::c_int {
    if let Some(Ev { key, pressed }) = KEYS.pop() {
        // if pressed == 1 {
        //     eprintln!("{key} up");
        // } else {
        //     eprintln!("{key} down");
        // }

        unsafe {
            *_pressed = pressed;
            *_doom_key = key;
        }

        1
    } else {
        0
    }
}

#[no_mangle]
unsafe extern "C" fn DG_SetWindowTitle(title: *const ffi::c_char) {
    let title = unsafe { libc::as_rust_str(title) }.unwrap();
    rename(title).unwrap();
}

//

fn framebuffer_info() -> Framebuffer<'static> {
    let fbo_info = OpenOptions::new().read(true).open("/dev/fb0-info").unwrap();
    let mut fbo_info = BufReader::new(fbo_info);

    let mut buf = String::new();
    fbo_info.read_line(&mut buf).unwrap();
    drop(fbo_info);

    let mut fbo_info_iter = buf.split(':');
    let width = fbo_info_iter.next().unwrap().parse::<usize>().unwrap();
    let height = fbo_info_iter.next().unwrap().parse::<usize>().unwrap();
    let pitch = fbo_info_iter.next().unwrap().parse::<usize>().unwrap();
    // let bpp = fbo_info_iter.next().unwrap().parse::<usize>().unwrap();

    Framebuffer {
        width,
        height,
        pitch,
        buf: None,
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Framebuffer<'a> {
    width: usize,
    height: usize,
    pitch: usize,
    buf: Option<&'a mut [u8]>,
}

impl Framebuffer<'_> {
    fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for yd in y..y + h {
            let spot = x * 4 + yd * self.pitch;
            self.buf.as_mut().unwrap()[spot..spot + 4 * w]
                .as_chunks_mut::<4>()
                .0
                .fill(color.as_arr());
        }
    }
}

//

fn main() {
    _strncmp_test();
    _strncasecmp_test();
    _atoi_test();
    _strlen_test();

    // println!("doomgeneric_Create");

    let argv = args()
        .map(|a| CString::new(a).unwrap())
        .collect::<Vec<CString>>();
    let c_argv = argv.iter().map(|s| s.as_ptr()).collect::<Vec<*const i8>>();
    let c_argv = c_argv.as_ptr();
    let c_argc = argv.len();

    println!("argv: {argv:?}");

    unsafe {
        doomgeneric_Create(c_argc as i32, c_argv);
    }

    // println!("doomgeneric_Tick");

    loop {
        unsafe { doomgeneric_Tick() };
    }
}
