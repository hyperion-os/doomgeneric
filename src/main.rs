#![no_std]
#![feature(c_variadic, c_size_t, new_uninit, slice_as_chunks, const_mut_refs)]

//

use core::{ffi, mem, ptr, slice};

use alloc::string::String;
use hyperion_color::Color;
use libstd::{
    eprintln,
    fs::{File, OpenOptions, Stdin},
    io::{BufReader, Read},
    sync::Mutex,
    sys::{fs::FileDesc, map_file, nanosleep, rename, timestamp, unmap_file, yield_now},
    thread::spawn,
};
use ringbuf::Rb;

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
    buf: &mut [],
});

struct Ev {
    key: u8,
    pressed: i32,
}

static RB: Mutex<Option<ringbuf::HeapRb<Ev>>> = Mutex::new(None);

//

#[no_mangle]
extern "C" fn DG_Init() {
    RB.lock().get_or_insert_with(|| ringbuf::HeapRb::new(256));

    spawn(|| {
        let mut stdin = BufReader::new(unsafe { File::new(Stdin::FD) });

        let mut key = [0u8; 40];

        eprintln!("READING STDIN FOR KEYS");
        let mut buf = String::new();
        while stdin.read_line(&mut buf).is_ok() {
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            struct KeyboardEventSer {
                // pub scancode: u8,
                state: u8,
                keycode: u8,
                unicode: Option<char>,
            }

            // eprintln!("GOT RAW `{}`", buf.trim());
            let Ok(ev) = serde_json::from_str::<KeyboardEventSer>(&buf.trim()) else {
                continue;
            };
            buf.clear();
            eprintln!("GOT {ev:?}");

            let pressed = if ev.state == 0 { 1 } else { 0 };

            let key = match ev.keycode {
                103 => 0xae, // right
                101 => 0xac, // left
                88 => 0xad,  // up
                102 => 0xaf, // down
                // #define KEY_STRAFE_L	0xa0
                // #define KEY_STRAFE_R	0xa1
                96 => 0xa2,       // space - use
                93 => 0xa3,       // lctrl - fire
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

                30 => 0x7f, // backspace
                34 => 0xff, // pause

                29 => 0x3d, // equals
                28 => 0x2d, // minus

                // (0x80+0x36) rshift, idk, mine doesnt work
                100 => 0x80 + 0x1d,     // rctrl
                95 | 97 => 0x80 + 0x38, // r/l alt

                // #define KEY_CAPSLOCK    (0x80+0x3a)
                // #define KEY_NUMLOCK     (0x80+0x45)
                // #define KEY_SCRLCK      (0x80+0x46)
                // #define KEY_PRTSCR      (0x80+0x59)

                // #define KEY_HOME        (0x80+0x47)
                // #define KEY_END         (0x80+0x4f)
                // #define KEY_PGUP        (0x80+0x49)
                // #define KEY_PGDN        (0x80+0x51)
                // #define KEY_INS         (0x80+0x52)
                // #define KEY_DEL         (0x80+0x53)

                // #define KEYP_0          0
                // #define KEYP_1          KEY_END
                // #define KEYP_2          KEY_DOWNARROW
                // #define KEYP_3          KEY_PGDN
                // #define KEYP_4          KEY_LEFTARROW
                // #define KEYP_5          '5'
                // #define KEYP_6          KEY_RIGHTARROW
                // #define KEYP_7          KEY_HOME
                // #define KEYP_8          KEY_UPARROW
                // #define KEYP_9          KEY_PGUP

                // #define KEYP_DIVIDE     '/'
                // #define KEYP_PLUS       '+'
                // #define KEYP_MINUS      '-'
                // #define KEYP_MULTIPLY   '*'
                // #define KEYP_PERIOD     0
                // #define KEYP_EQUALS     KEY_EQUALS
                // #define KEYP_ENTER      KEY_ENTER
                _ => continue,
            };

            RB.lock().as_mut().unwrap().push(Ev { key, pressed });
        }
        eprintln!("READING STDIN FOR KEYS STOPPED");

        // let mut stdin = STDIN.lock();

        // stdin.read();
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
    info.buf = buf;

    *fb = info;

    // unmap_file(fbo.as_desc(), fbo_mapped, 0).expect("failed to unmap the fb");
    // keep the file open
    mem::forget(fbo);
}

#[no_mangle]
extern "C" fn DG_DrawFrame() {
    // eprintln!("DG_DrawFrame");
    // unimplemented!();
    let mut fb = FB.lock();

    const DOOMGENERIC_RESX: usize = 640;
    const DOOMGENERIC_RESY: usize = 400;

    const PITCH: usize = DOOMGENERIC_RESX * mem::size_of::<u32>();
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
            fb.fill(x + 100, y + 100, 1, 1, Color::new(c.b, c.g, c.r));
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
    if let Some(Ev { key, pressed }) = RB.lock().as_mut().unwrap().pop() {
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
        buf: &mut [],
    }
}

#[derive(Debug)]
struct Framebuffer<'a> {
    width: usize,
    height: usize,
    pitch: usize,
    buf: &'a mut [u8],
}

impl Framebuffer<'_> {
    fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for yd in y..y + h {
            let spot = x * 4 + yd * self.pitch;
            self.buf[spot..spot + 4 * w]
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

    unsafe {
        doomgeneric_Create(0, ptr::null());
    }

    // println!("doomgeneric_Tick");

    loop {
        unsafe { doomgeneric_Tick() };
    }
}
