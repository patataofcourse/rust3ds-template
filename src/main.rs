use ctru::{
    gfx::{Gfx, Screen},
    console::Console,
    services::apt::Apt,
    services::hid::{Hid, KeyPad},
};
use ctru_sys::{
    C2D_Sprite,
    C2D_SpriteSheet,
    C3D_RenderTarget,
    GFX_TOP,
    GFX_LEFT,
};
use ui::{SpriteSheet, Image};

fn main() {
    let apt = Apt::init().unwrap();
    let hid = Hid::init().unwrap();
    let gfx = Gfx::default();
    let console = Console::init(Screen::Bottom);
    console.select();
    unsafe {
        ctru_sys::romfsMountSelf("romfs\0".as_ptr());
    }
    let screen: *mut C3D_RenderTarget;
    citro2d::init(None, None);
    unsafe {
        screen = ctru_sys::C2D_CreateScreenTarget(GFX_TOP, GFX_LEFT);
    }
    let bg_sheet = SpriteSheet::from_file("romfs:/bg.t3x\0").expect("No spritesheet bg.t3x!");
    let bg = bg_sheet.get_sprite(0).unwrap();
    let fg = bg_sheet.get_sprite(1).unwrap();

    println!("Welcome to Barista!");
    println!("\x1b[29;12HPress Start to exit");
    
    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::KEY_START) {
            break;
        }

        // Render the scene
        unsafe {
            use ctru_sys::*;
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW as u8);
            C2D_TargetClear(screen, citro2d::WHITE);
            citro2d::scene_begin(screen);
        }
        bg.draw(0, 0, 1.0, 1.0, 0.0, 0.0);
        fg.draw(0, 188, 1.0, 1.0, 0.0, 0.0);
        unsafe {
            ctru_sys::C3D_FrameEnd(0);
        }
    }
}