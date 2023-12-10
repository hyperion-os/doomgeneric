#![no_std]
#![feature(c_variadic, c_size_t, new_uninit)]

//

use core::{ffi, ptr};

use self::libc::{_atoi_test, _strncasecmp_test, _strncmp_test};

//

extern crate alloc;

mod libc;

//

extern "C" {
    fn doomgeneric_Create(argc: ffi::c_int, argv: *const *const ffi::c_char) -> ffi::c_int;

    fn doomgeneric_Tick() -> ffi::c_int;
}

//

#[no_mangle]
extern "C" fn DG_Init() {}

#[no_mangle]
extern "C" fn DG_DrawFrame() {
    unimplemented!()
}

#[no_mangle]
extern "C" fn DG_SleepMs(_ms: u32) {
    unimplemented!()
}

#[no_mangle]
extern "C" fn DG_GetTicksMs() -> u32 {
    unimplemented!()
}

#[no_mangle]
extern "C" fn DG_GetKey(_pressed: *mut ffi::c_int, _doom_key: *mut ffi::c_uchar) -> ffi::c_int {
    unimplemented!()
}

#[no_mangle]
extern "C" fn DG_SetWindowTitle(_title: *const ffi::c_char) {
    unimplemented!()
}

//

fn main() {
    _strncmp_test();
    _strncasecmp_test();
    _atoi_test();

    // println!("doomgeneric_Create");

    unsafe {
        doomgeneric_Create(0, ptr::null());
    }

    // println!("doomgeneric_Tick");

    loop {
        unsafe { doomgeneric_Tick() };
    }
}
