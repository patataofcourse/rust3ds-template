#![feature(allocator_api)]

extern crate barista_ui as ui_lib;

use ctru::{
    console::Console,
    gfx::Gfx,
    services::{apt::Apt, hid::Hid},
};
use error::error_applet;
use std::{
    panic::{self, PanicInfo},
    process,
};
use ui_lib::{BaristaUI, Screen};

mod error;
use self::error::{Error, Result};

#[cfg(feature = "audio")]
mod audio;

mod format;
mod launcher;
mod mod_picker;
mod scene;
use self::{
    launcher::GameVer,
    scene::menu::{MenuAction, MenuState},
};

/// Bindings + safe abstraction for plgldr.c
mod plgldr;

static mut CONFIG: Option<format::saltwater_cfg::Config> = None;

#[cfg(feature = "audio")]
static mut AUDIO: Option<*const audio::AudioManager> = None;

fn main() {
    match run() {
        Ok(_) => {}
        Err(c) => {
            let error = match c {
                Error::CtruError(c) => match c {
                    ctru::Error::Os(c) => format!("System error {:#X}", c),
                    ctru::Error::Libc(c) => format!("libc error:\n{}", c),
                    ctru::Error::ServiceAlreadyActive => format!("Service already active"),
                    ctru::Error::OutputAlreadyRedirected => format!("Output already redirected"),
                    _ => todo!(),
                },
                Error::IoError(c) => {
                    todo!();
                }
                Error::OtherError(c) => c,
            };
            error_applet(error);

            process::exit(1);
        }
    }
}

fn run() -> error::Result<()> {
    ctru::init();
    let apt = Apt::init().unwrap();
    let hid = Hid::init().unwrap();
    let gfx = Gfx::init().unwrap();
    let console = Console::init(gfx.bottom_screen.borrow_mut());
    unsafe {
        assert!(ctru_sys::romfsMountSelf("romfs\0".as_ptr()) == 0);
        assert!(ctru_sys::ndspInit() == 0);
        ctru_sys::ndspSetOutputMode(ctru_sys::NDSP_OUTPUT_STEREO);
    }

    panic::set_hook(Box::new(panic_hook));

    // Initialize GFX stuff
    let mut ui = BaristaUI::init();

    let top_scene = scene::top_screen_scene(&ui);
    ui.set_scene(Screen::Top, &top_scene);

    // Init loader
    let versions = launcher::get_available_games();

    let mut game_to_load: Option<GameVer> = None;
    launcher::check_for_plgldr();

    let mods = mod_picker::get_available_mods()?;

    // Init menu
    let mut menu = MenuState::default();
    menu.render(&console, &versions);

    #[allow(unused)]
    let mut audio_player;

    #[allow(unused)]
    #[cfg(not(feature = "audio"))]
    {
        audio_player = ();
    }

    #[cfg(feature = "audio")]
    {
        // Music test
        audio_player = audio::AudioManager::new();

        // Initial values for audio player
        audio_player.load("romfs:/audio/strm/bartender_construction.bcstm".to_string());
        audio_player.play();

        unsafe { AUDIO = Some(&audio_player) }
    }

    // Init config
    *config_wrapped() = Some(
        format::saltwater_cfg::Config::from_file("/spicerack/bin/saltwater.cfg")
            .unwrap_or_default(),
    );

    // Main loop
    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();

        ui.render();

        menu.run(&hid, &console, &versions);

        match &menu.action {
            MenuAction::Exit => break,
            MenuAction::Run => {
                game_to_load = Some(versions[menu.cursor as usize].clone());
                break;
            }
            #[cfg(feature = "audio")]
            MenuAction::ToggleAudio => {
                if audio_player.is_playing() {
                    audio_player.pause()
                } else {
                    audio_player.play()
                }
            }
            MenuAction::ChangeMenu(_) | MenuAction::None | MenuAction::MoveCursor => {}
        }
    }

    unsafe {
        assert!(ctru_sys::romfsUnmount("romfs\0".as_ptr()) == 0);
        ctru_sys::ndspExit();
    }

    drop(console);
    drop(gfx);
    drop(hid);

    if let Some(c) = game_to_load {
        launcher::launch(c)
    }

    Ok(())
}

fn config() -> &'static mut format::saltwater_cfg::Config {
    unsafe { CONFIG.as_mut().expect("Config not initialized") }
}

fn config_wrapped() -> &'static mut Option<format::saltwater_cfg::Config> {
    unsafe { &mut CONFIG }
}

#[cfg(feature = "audio")]
fn audio<'a>() -> &'a audio::AudioManager {
    unsafe { &*AUDIO.expect("Audio not initialized") }
}

fn panic_hook(info: &PanicInfo) {
    let location_info = if let Some(c) = info.location() {
        format!(" at {}:{}:{}", c.file(), c.line(), c.column())
    } else {
        String::new()
    };

    let msg = if let Some(c) = info.payload().downcast_ref::<&str>() {
        format!("panic: {:?}{}\0", c, location_info)
    } else if let Some(c) = info.payload().downcast_ref::<String>() {
        format!("panic: {:?}{}\0", c, location_info)
    } else {
        format!("panic{}\0", location_info)
    };
    error_applet(msg);

    process::exit(1);
}
