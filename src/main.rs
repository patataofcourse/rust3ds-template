#![feature(allocator_api, int_roundings, panic_backtrace_config)]

extern crate barista_ui as ui_lib;

use ctru::{
    console::Console,
    services::{apt::Apt, gfx::Gfx, hid::Hid, ps::Ps, romfs::RomFS},
};
use error::error_applet;
use std::{
    fmt::Display, io::Read, panic::{self, PanicHookInfo}, process
};
use ui_lib::{BaristaUI, Screen};

mod error;

use self::error::{Error, Result};

#[cfg(feature = "audio")]
use ctru::services::ndsp::{self, Ndsp};

#[macro_use]
mod log;

#[cfg(feature = "audio")]
mod audio;

mod constants;
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

//TODO: mutex or reduce all these things to one global state struct
static mut CONFIG: Option<format::saltwater_cfg::Config> = None;

//TODO: just use a mutex please
#[cfg(feature = "audio")]
static mut AUDIO: Option<*const audio::AudioManager> = None;

fn main() {
    let is_citra = unsafe {
        let mut citra_info = 0i64;
        matches!(ctru_sys::svcGetSystemInfo(&mut citra_info, 0x20000, 0), 0)
    };

    if !is_citra {
        panic::set_hook(Box::new(panic_hook));
    } else {
        panic::set_hook(Box::new(citra_panic_hook))
    }

    match run(is_citra) {
        Ok(_) => {}
        Err(c) => {
            let error = match c {
                Error::Ctru(c) => match c {
                    ctru::Error::Os(c) => format!("System error {:#X}", c),
                    ctru::Error::Libc(c) => format!("libc error:\n{}", c),
                    ctru::Error::ServiceAlreadyActive => "Service already active".to_string(),
                    ctru::Error::OutputAlreadyRedirected => "Output already redirected".to_string(),
                    c => format!("Unknown ctru error\n{}", c),
                },
                Error::Io(c) => {
                    format!("IO error: {}", c)
                }
                Error::TomlDe(c) => {
                    format!("TOML deserialize error: {}", c)
                }
                Error::TomlSer(c) => {
                    format!("TOML serialize error: {}", c)
                }
                Error::Other(c) => c,
            };
            if is_citra {
                citra_error(error);
            } else {
                error_applet(error);
            }

            process::exit(1);
        }
    }
}

fn run(is_citra: bool) -> error::Result<()> {
    let apt = Apt::new()?;
    let mut hid = Hid::new()?;
    let gfx = Gfx::new()?;
    let ps = Ps::new()?;
    let romfs = RomFS::new()?;
    let console = Console::new(gfx.bottom_screen.borrow_mut());

    log!(General, "test");

    // Initialize GFX stuff
    let mut ui = BaristaUI::init();

    // TODO: proper mod manifest support rather than... this.
    let mut f = std::fs::File::open("romfs:/sample_mod.toml")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    toml::from_str::<format::manifest::ModManifest>(&s)?;

    ui.set_scene(Screen::Top, scene::top_screen_scene);

    // Init loader
    let versions = launcher::get_available_games();

    let mut game_to_load: Option<GameVer> = None;
    launcher::check_for_plgldr();

    let mods = mod_picker::get_available_mods()?;

    // Init Barista config
    let mut settings = format::barista_cfg::BaristaConfig::from_file("sdmc:/spicerack/cfg.toml")?;
    let mut random = [0u8; 1];
    ps.generate_random_bytes(&mut random)?;
    if !settings.is_new && random == [0x69u8; 1] {
        scene::top_screen::nicole_easter_egg(&mut ui);
    }

    // Init menu
    let mut menu = MenuState::default();
    menu.render(&console, &versions, &[], 0, 0, &settings)?;

    #[allow(unused)]
    let mut audio_player;

    #[allow(unused)]
    let mut ndsp;

    #[allow(unused)]
    #[cfg(not(feature = "audio"))]
    {
        audio_player = ();
        ndsp = ();
    }

    #[cfg(feature = "audio")]
    {
        ndsp = Ndsp::new()?;
        ndsp.set_output_mode(ndsp::OutputMode::Stereo);

        // Music test
        audio_player = audio::AudioManager::new();

        // Initial values for audio player
        audio_player.load("romfs:/audio/strm/Practice.bcstm".to_string());
        audio_player.play();

        unsafe { AUDIO = Some(&audio_player) }
    }

    // Init Saltwater config
    unsafe {
        CONFIG = Some(
            format::saltwater_cfg::Config::from_file("sdmc:/spicerack/bin/saltwater.cfg")
                .unwrap_or_default(),
        );
    }
    // clear mods not in the current folder, save the cfg file after clearing
    config().clear_deleted_mods(&mods);
    config().to_file("sdmc:/spicerack/bin/saltwater.cfg")?;

    let mut page = 0;

    // Main loop
    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();

        ui.render();

        menu.run(&hid, &console, &versions, &mods, &mut page, &mut settings)?;

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
            MenuAction::SaveConfig => {
                config().to_file("sdmc:/spicerack/bin/saltwater.cfg")?;
            }
            MenuAction::SaveSettings => {
                settings.to_file("sdmc:/spicerack/cfg.toml")?;
            }
            MenuAction::ChangeMenu(_)
            | MenuAction::None
            | MenuAction::UpdateScreen
            | MenuAction::ChangePage(_)
            | MenuAction::ChangeIndex(..)
            | MenuAction::ToggleMod
            | MenuAction::ToggleSetting(_) => {}
        }
    }

    drop(console);
    drop(gfx);
    drop(hid);
    drop(romfs);
    #[allow(dropping_copy_types)]
    drop(ndsp);

    if let Some(c) = game_to_load {
        launcher::launch(c, is_citra, &settings)
    }

    Ok(())
}

fn config() -> &'static mut format::saltwater_cfg::Config {
    unsafe { CONFIG.as_mut().expect("Config not initialized") }
}

#[cfg(feature = "audio")]
fn audio<'a>() -> &'a audio::AudioManager {
    unsafe { &*AUDIO.expect("Audio not initialized") }
}

fn panic_hook(info: &PanicHookInfo) {
    error_applet(panic_message(info));

    process::exit(1);
}

fn citra_panic_hook(info: &PanicHookInfo) {
    /* let mut backtrace = backtrace::Backtrace::new();
    backtrace.resolve();
    println!(
        "{:?}",
        backtrace
            .frames()
            .iter()
            .map(|c| c.symbol_address())
            .collect::<Vec<_>>()
    ); */

    citra_error(panic_message(info));

    process::exit(1);
}

fn panic_message(info: &PanicHookInfo) -> String {
    let location_info = if let Some(c) = info.location() {
        format!(" at {}:{}:{}", c.file(), c.line(), c.column())
    } else {
        String::new()
    };

    if let Some(c) = info.payload().downcast_ref::<&str>() {
        format!("panic: {:?}{}\0", c, location_info)
    } else if let Some(c) = info.payload().downcast_ref::<String>() {
        format!("panic: {:?}{}\0", c, location_info)
    } else {
        format!("panic{}\0", location_info)
    }
}

fn citra_error(message: impl Display) {
    let apt = Apt::new().unwrap();
    let gfx = Gfx::new().ok();

    // it's possible GFX is already initialized soooo
    let console = gfx.as_ref().map(|c| Console::new(c.bottom_screen.borrow_mut()));
    unsafe { ctru_sys::consoleClear() }

    println!("{}", message);
    
    while apt.main_loop() {
        unsafe {
            use ctru_sys::*;
            gspWaitForEvent(GSPGPU_EVENT_VBlank0, true);
        }
    }

    drop(console);
}
