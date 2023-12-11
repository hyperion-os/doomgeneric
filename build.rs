const DOOM_C_FILES: &[&str] = &[
    "doomgeneric/am_map.c",
    "doomgeneric/d_event.c",
    "doomgeneric/d_items.c",
    "doomgeneric/d_iwad.c",
    "doomgeneric/d_loop.c",
    "doomgeneric/d_main.c",
    "doomgeneric/d_mode.c",
    "doomgeneric/d_net.c",
    "doomgeneric/doomdef.c",
    "doomgeneric/doomgeneric.c",
    "doomgeneric/doomstat.c",
    "doomgeneric/dstrings.c",
    "doomgeneric/dummy.c",
    "doomgeneric/f_finale.c",
    "doomgeneric/f_wipe.c",
    "doomgeneric/g_game.c",
    "doomgeneric/gusconf.c",
    "doomgeneric/hu_lib.c",
    "doomgeneric/hu_stuff.c",
    "doomgeneric/i_cdmus.c",
    "doomgeneric/i_endoom.c",
    "doomgeneric/i_input.c",
    "doomgeneric/i_joystick.c",
    "doomgeneric/i_scale.c",
    "doomgeneric/i_sound.c",
    "doomgeneric/i_system.c",
    "doomgeneric/i_timer.c",
    "doomgeneric/i_video.c",
    "doomgeneric/icon.c",
    "doomgeneric/info.c",
    "doomgeneric/m_argv.c",
    "doomgeneric/m_bbox.c",
    "doomgeneric/m_cheat.c",
    "doomgeneric/m_config.c",
    "doomgeneric/m_controls.c",
    "doomgeneric/m_fixed.c",
    "doomgeneric/m_menu.c",
    "doomgeneric/m_misc.c",
    "doomgeneric/m_random.c",
    "doomgeneric/memio.c",
    "doomgeneric/mus2mid.c",
    "doomgeneric/p_ceilng.c",
    "doomgeneric/p_doors.c",
    "doomgeneric/p_enemy.c",
    "doomgeneric/p_floor.c",
    "doomgeneric/p_inter.c",
    "doomgeneric/p_lights.c",
    "doomgeneric/p_map.c",
    "doomgeneric/p_maputl.c",
    "doomgeneric/p_mobj.c",
    "doomgeneric/p_plats.c",
    "doomgeneric/p_pspr.c",
    "doomgeneric/p_saveg.c",
    "doomgeneric/p_setup.c",
    "doomgeneric/p_sight.c",
    "doomgeneric/p_spec.c",
    "doomgeneric/p_switch.c",
    "doomgeneric/p_telept.c",
    "doomgeneric/p_tick.c",
    "doomgeneric/p_user.c",
    "doomgeneric/r_bsp.c",
    "doomgeneric/r_data.c",
    "doomgeneric/r_draw.c",
    "doomgeneric/r_main.c",
    "doomgeneric/r_plane.c",
    "doomgeneric/r_segs.c",
    "doomgeneric/r_sky.c",
    "doomgeneric/r_things.c",
    "doomgeneric/s_sound.c",
    "doomgeneric/sha1.c",
    "doomgeneric/sounds.c",
    "doomgeneric/st_lib.c",
    "doomgeneric/st_stuff.c",
    "doomgeneric/statdump.c",
    "doomgeneric/tables.c",
    "doomgeneric/v_video.c",
    "doomgeneric/w_checksum.c",
    "doomgeneric/w_file.c",
    "doomgeneric/w_file_stdc.c",
    "doomgeneric/w_main.c",
    "doomgeneric/w_wad.c",
    "doomgeneric/wi_stuff.c",
    "doomgeneric/z_zone.c",
];

fn main() {
    println!("cargo:rustc-link-arg=-no-pie");

    cc::Build::new()
        .compiler("x86_64-elf-gcc")
        .files(DOOM_C_FILES)
        // .files(&["doomgeneric/test.c"])
        .warnings(false)
        .extra_warnings(false)
        .flag("-w") // TODO: fix the warnings in doomgeneric
        .flag("-nostdlib")
        // .flag("-nolibc")
        // .flag("-ffreestanding")
        // .flag("-fomit-frame-pointer")
        // .flag("-O0")
        .include("./include")
        .compile("doomgeneric");
}