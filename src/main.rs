use ctru::{
    gfx::{Gfx, Screen},
    console::Console,
    services::apt::Apt,
    services::hid::{Hid, KeyPad},
};
use ctru_sys::{
    C3D_RenderTarget,
    GFX_TOP,
    GFX_LEFT,
};
use ui::SpriteSheet;

mod launcher;

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
    ui::init();
    unsafe {
        screen = ctru_sys::C2D_CreateScreenTarget(GFX_TOP, GFX_LEFT);
    }
    let bg_sheet = SpriteSheet::from_file("romfs:/bg.t3x").expect("No spritesheet bg.t3x!");
    let bg = bg_sheet.get_sprite(0).unwrap();
    let fg = bg_sheet.get_sprite(1).unwrap();

    let barista_sheet = SpriteSheet::from_file("romfs:/barista.t3x").expect("No spritesheet barista.t3x!");
    let barista = barista_sheet.get_sprite(0).unwrap();
    let nicole = barista_sheet.get_sprite(1).unwrap();

    let sign_sheet = SpriteSheet::from_file("romfs:/sign.t3x").expect("No spritesheet barista.t3x!");
    let sign = sign_sheet.get_sprite(0).unwrap();
    let sign_text = sign_sheet.get_sprite(1).unwrap();

    let versions = launcher::get_available_games();

    println!("Welcome to Barista!");
    //println!(" - Press A to boot Saltwater");
    println!(" - Press Start to exit");
    
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
            C2D_TargetClear(screen, 0xFFFFFFFF);
            ctru_sys::C2D_Flush();
            ctru_sys::C3D_FrameDrawOn(screen);
            ctru_sys::C2D_SceneSize(
                (*screen).frameBuf.width.into(),
                (*screen).frameBuf.height.into(),
                (*screen).linked,
            );
        }
        bg.draw(0, 0, 1.0, 1.0, 0.0, 0.0);
        barista.draw(255, 70, 1.0, 1.0, 0.0, 0.0);
        //nicole.draw(174, 17, 1.0, 1.0, 0.0, 0.0);
        fg.draw(0, 188, 1.0, 1.0, 0.0, 0.0);
        sign.draw(30, 150, 1.0, 1.0, 0.0, 0.0);
        sign_text.draw(30, 150, 1.0, 1.0, 0.0, 0.0);
        unsafe {
            ctru_sys::C3D_FrameEnd(0);
        }
    }
}