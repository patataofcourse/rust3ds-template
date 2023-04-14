// Menu: Let's Get This Done For The First Release Edition
// Wonder if anything from here will be salvageable

use std::path::PathBuf;

use crate::{
    constants::{
        SLOT_NAMES_DEFAULT, SLOT_NAMES_GATE, SLOT_NAMES_INTERNAL, SLOT_NAMES_INTERNAL_GATE,
        SLOT_NAMES_NORETCON,
    },
    format::barista_cfg::{BaristaConfig, SlotTitleMode},
    launcher::GameVer,
    mod_picker,
};
use ctru::{
    console::Console,
    services::hid::{Hid, KeyPad},
};

#[derive(Clone, Debug)]
pub struct MenuState {
    pub sub_menu: SubMenu,
    pub cursor: u32,
    pub action: MenuAction,
    pub hold_controller: HoldController,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct HoldController {
    pub up: Option<u32>,
    pub down: Option<u32>,
    pub left: Option<u32>,
    pub right: Option<u32>,
}

impl HoldController {
    const FIRST_PRESS_TIME: u32 = 15;
    const LOOP_PRESS_TIME: u32 = 4;

    pub fn update(&mut self, keys: KeyPad) {
        if keys.contains(KeyPad::DUP) {
            if let Some(c) = &mut self.up {
                *c += 1;
            } else {
                self.up = Some(0)
            }
        } else {
            self.up = None;
        }
        if keys.contains(KeyPad::DDOWN) {
            if let Some(c) = &mut self.down {
                *c += 1;
            } else {
                self.down = Some(0)
            }
        } else {
            self.down = None;
        }
        if keys.contains(KeyPad::DLEFT) {
            if let Some(c) = &mut self.left {
                *c += 1;
            } else {
                self.left = Some(0)
            }
        } else {
            self.left = None;
        }
        if keys.contains(KeyPad::DRIGHT) {
            if let Some(c) = &mut self.right {
                *c += 1;
            } else {
                self.right = Some(0)
            }
        } else {
            self.right = None;
        }
        if keys.contains(KeyPad::L) {
            log!(General, "{:?}", self);
        }
    }

    pub fn should_click(&self, key: KeyPad) -> bool {
        let check = |t| t == 0 || (t >= Self::FIRST_PRESS_TIME && t % Self::LOOP_PRESS_TIME == 0);

        if key == KeyPad::DUP {
            if let Some(t) = self.up {
                check(t)
            } else {
                false
            }
        } else if key == KeyPad::DDOWN {
            if let Some(t) = self.down {
                check(t)
            } else {
                false
            }
        } else if key == KeyPad::DLEFT {
            if let Some(t) = self.left {
                check(t)
            } else {
                false
            }
        } else if key == KeyPad::DRIGHT {
            if let Some(t) = self.right {
                check(t)
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum SubMenu {
    Main,
    Run,
    Options,
    #[cfg(feature = "audio")]
    Music,
    SetUp(bool),
    #[cfg(debug_assertions)]
    Log,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    // All
    None,
    ChangeMenu(SubMenu),
    Exit,
    UpdateScreen,

    // Run
    Run,

    // Options
    ToggleSetting(u8),
    SaveSettings,

    // SetUp
    ChangePage(bool),
    SaveConfig,
    ToggleMod,
    ChangeIndex(bool, bool),

    // Music
    #[cfg(feature = "audio")]
    ToggleAudio,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            sub_menu: SubMenu::Main,
            cursor: 0,
            action: MenuAction::None,
            hold_controller: HoldController::default(),
        }
    }
}

impl SubMenu {
    const ACTIONS_MAIN: &[MenuAction] = &[
        MenuAction::ChangeMenu(SubMenu::Run),
        MenuAction::ChangeMenu(SubMenu::SetUp(false)),
        #[cfg(feature = "audio")]
        MenuAction::ChangeMenu(SubMenu::Music),
        MenuAction::ChangeMenu(SubMenu::Options),
        MenuAction::Exit,
    ];
    const ACTIONS_RUN: [MenuAction; 1] = [MenuAction::ChangeMenu(SubMenu::Main)];
    const ACTIONS_SETUP: [MenuAction; 3] = [
        MenuAction::ChangePage(false),
        MenuAction::ChangePage(true),
        MenuAction::SaveConfig,
    ];
    #[cfg(feature = "audio")]
    const ACTIONS_MUSIC: [MenuAction; 2] = [
        MenuAction::ToggleAudio,
        MenuAction::ChangeMenu(SubMenu::Main),
    ];
    const ACTIONS_OPTIONS: [MenuAction; 3] = [
        MenuAction::ToggleSetting(0),
        MenuAction::ToggleSetting(1),
        MenuAction::SaveSettings,
    ];

    pub fn actions(&self) -> &[MenuAction] {
        match &self {
            SubMenu::Main => Self::ACTIONS_MAIN,
            SubMenu::Run => &Self::ACTIONS_RUN,
            SubMenu::SetUp(_) => &Self::ACTIONS_SETUP,
            #[cfg(feature = "audio")]
            SubMenu::Music => &Self::ACTIONS_MUSIC,
            SubMenu::Options => &Self::ACTIONS_OPTIONS,
            #[cfg(debug_assertions)]
            SubMenu::Log => &[MenuAction::ChangeMenu(SubMenu::Main)],
        }
    }

    pub fn cursor_option_len(&self, versions: &Vec<GameVer>, mods: &Vec<(String, u16)>) -> u32 {
        (self.actions().len()
            + if let SubMenu::Run = self {
                versions.len()
            } else if let SubMenu::SetUp(_) = self {
                mods.len()
            } else {
                0
            }) as u32
    }
}

impl MenuState {
    pub fn actions(&self) -> &[MenuAction] {
        self.sub_menu.actions()
    }
    pub fn cursor_option_len(&self, versions: &Vec<GameVer>, mods: &Vec<(String, u16)>) -> u32 {
        self.sub_menu.cursor_option_len(versions, mods)
    }

    pub fn run(
        &mut self,
        hid: &Hid,
        console: &Console,
        versions: &Vec<GameVer>,
        mods: &Vec<PathBuf>,
        page: &mut usize,
        settings: &mut BaristaConfig,
    ) {
        self.action = MenuAction::None;

        let mut mod_page = if let SubMenu::SetUp(_) = self.sub_menu {
            mod_picker::show_page(mods, crate::config(), *page)
        } else {
            vec![]
        };

        if hid.keys_down().contains(KeyPad::START) {
            self.action = MenuAction::Exit;
            return;
        }

        self.hold_controller.update(hid.keys_held());

        if self.hold_controller.should_click(KeyPad::DUP) {
            if self.cursor > 0 {
                self.cursor -= 1;
            } else {
                self.cursor = self.cursor_option_len(versions, &mod_page) - 1;
            }
            self.action = MenuAction::UpdateScreen
        } else if self.hold_controller.should_click(KeyPad::DDOWN) {
            if self.cursor < self.cursor_option_len(versions, &mod_page) - 1 {
                self.cursor += 1;
            } else {
                self.cursor = 0;
            }
            self.action = MenuAction::UpdateScreen
        } else if hid.keys_down().contains(KeyPad::B) {
            if let SubMenu::Main = self.sub_menu {
                self.action = MenuAction::Exit;
            } else {
                self.action = self.actions()[self.actions().len() - 1].clone();
            }
        } else if hid.keys_down().contains(KeyPad::A) {
            if let SubMenu::Run = self.sub_menu {
                if self.cursor == self.cursor_option_len(versions, &mod_page) - 1 {
                    self.action = MenuAction::ChangeMenu(SubMenu::Main)
                } else {
                    self.action = MenuAction::Run
                }
            } else if let SubMenu::SetUp(_) = self.sub_menu {
                if self.cursor_option_len(versions, &mod_page) - self.cursor <= 3 {
                    self.action =
                        SubMenu::ACTIONS_SETUP[self.cursor as usize - mod_page.len()].clone();
                } else {
                    self.action = MenuAction::ToggleMod;
                }
            } else {
                self.action = self.actions()[self.cursor as usize].clone()
            }
        }
        #[cfg(debug_assertions)]
        if hid.keys_down().contains(KeyPad::SELECT) {
            self.action = MenuAction::ChangeMenu(SubMenu::Log)
        }
        if let SubMenu::SetUp(c) = &mut self.sub_menu {
            if hid.keys_down().contains(KeyPad::Y) {
                *c = !*c;
                self.action = MenuAction::UpdateScreen
            }
            if hid.keys_down().contains(KeyPad::L) {
                self.action = MenuAction::ChangePage(false)
            } else if hid.keys_down().contains(KeyPad::R) {
                self.action = MenuAction::ChangePage(true)
            } else if self.hold_controller.should_click(KeyPad::DLEFT) {
                if hid.keys_held().contains(KeyPad::X) {
                    self.action = MenuAction::ChangeIndex(false, true)
                } else {
                    self.action = MenuAction::ChangeIndex(false, false)
                }
            } else if self.hold_controller.should_click(KeyPad::DRIGHT) {
                if hid.keys_held().contains(KeyPad::X) {
                    self.action = MenuAction::ChangeIndex(true, true)
                } else {
                    self.action = MenuAction::ChangeIndex(true, false)
                }
            }
        }

        match &self.action {
            MenuAction::Exit | MenuAction::Run | MenuAction::None => return,
            MenuAction::ChangeMenu(c) => {
                if let SubMenu::SetUp(_) = *c {
                    mod_page = mod_picker::show_page(mods, crate::config(), *page);
                }

                self.sub_menu = *c;
                self.cursor = 0;
                *page = 0;
            }
            MenuAction::SaveConfig | MenuAction::SaveSettings => {
                self.sub_menu = SubMenu::Main;
                self.cursor = 0;
                *page = 0;
            }
            MenuAction::ChangePage(c) => {
                if !c && *page > 0 {
                    *page -= 1;
                } else if *c && *page < mod_picker::num_pages(mods) - 1 {
                    *page += 1;
                }
                let old_len = mod_page.len() as u32;
                mod_page = mod_picker::show_page(mods, crate::config(), *page);
                log!(General, "{} {} {}", old_len, mod_page.len(), self.cursor);

                // Make sure the cursor is in-bounds
                if self.cursor < old_len {
                    self.cursor = self.cursor.clamp(0, mod_page.len() as u32 - 1);
                } else {
                    self.cursor = self
                        .cursor
                        .wrapping_add(mod_page.len() as u32)
                        .wrapping_sub(old_len);
                }
            }
            //TODO: properly order stuff in new gate mode (both ChangeIndex and ToggleMod)
            MenuAction::ChangeIndex(i, fast) => {
                if let Some(m) = mod_page.get_mut(self.cursor as usize) {
                    let config = crate::config();
                    if m.1 != u16::MAX {
                        config.btks.remove(&m.1);
                        let mut step: i16 = if *i { 1 } else { -1 };
                        if *fast {
                            step *= 0x10
                        }
                        let mut out = m.1.wrapping_add_signed(step);

                        while !mod_picker::is_valid_slot(out) || config.btks.contains_key(&out) {
                            out = match out.wrapping_add_signed(step) {
                                0x8000..=u16::MAX => 0x113,
                                0x114..=0x7FFF => 0,
                                c => c,
                            }
                        }

                        config.btks.insert(
                            out,
                            mod_picker::get_mod_name(mods, *page, self.cursor as usize),
                        );
                        m.1 = out;
                    }
                }
            }
            MenuAction::ToggleMod => {
                if let Some(m) = mod_page.get_mut(self.cursor as usize) {
                    let config = crate::config();
                    if m.1 == u16::MAX {
                        let mut val = 0;
                        while val <= 0x113 && config.btks.contains_key(&val) {
                            val += 1;
                        }
                        if val <= 0x113 {
                            config.btks.insert(
                                val,
                                mod_picker::get_mod_name(mods, *page, self.cursor as usize),
                            );
                        } else {
                            val = u16::MAX;
                        }
                        m.1 = val;
                    } else {
                        config.btks.remove(&m.1);
                        m.1 = u16::MAX;
                    }
                }
            }
            MenuAction::ToggleSetting(c) => match c {
                0 => {
                    settings.original_gates = !settings.original_gates;
                }
                1 => {
                    settings.slot_titles = match settings.slot_titles {
                        SlotTitleMode::Megamix => SlotTitleMode::Original,
                        SlotTitleMode::Original => SlotTitleMode::Internal,
                        SlotTitleMode::Internal | SlotTitleMode::Infernal => SlotTitleMode::Megamix,
                    }
                }
                _ => {}
            },
            MenuAction::UpdateScreen => {}
            #[cfg(feature = "audio")]
            MenuAction::ToggleAudio => {}
        }
        self.render(
            console,
            versions,
            &mod_page,
            *page,
            mod_picker::num_pages(mods),
            settings,
        )
    }

    pub fn render(
        &mut self,
        console: &Console,
        versions: &Vec<GameVer>,
        mods: &Vec<(String, u16)>,
        page: usize,
        num_pages: usize,
        settings: &BaristaConfig,
    ) {
        console.clear();
        match &self.sub_menu {
            SubMenu::Main => {
                println!("Barista - Main menu");
                println!();
                println!("Controls:");
                println!("- DPad up/down: move cursor");
                println!("- A: choose selected option");
                println!("- B: go to prev menu");
                println!("- Start: exit Barista");
                #[cfg(debug_assertions)]
                println!("- Select: open debug log");
                println!();
                println!(
                    " [{}] Run Saltwater",
                    if self.cursor == 0 { "*" } else { " " }
                );
                println!(
                    " [{}] Set up mods",
                    if self.cursor == 1 { "*" } else { " " }
                );
                #[cfg(feature = "audio")]
                println!(" [{}] Music", if self.cursor == 2 { "*" } else { " " });

                let cursor_increase = if cfg!(feature = "audio") { 1 } else { 0 };

                println!(
                    " [{}] Settings",
                    if self.cursor == 2 + cursor_increase {
                        "*"
                    } else {
                        " "
                    }
                );
                println!(
                    " [{}] Exit Barista",
                    if self.cursor == 3 + cursor_increase {
                        "*"
                    } else {
                        " "
                    }
                );
            }
            SubMenu::Run => {
                println!("Barista - Run Saltwater");
                println!();
                println!("Choose a version to run with Saltwater");
                println!();
                for (vnum, ver) in versions.iter().enumerate() {
                    println!(
                        " [{}] {}",
                        if self.cursor as usize == vnum {
                            "*"
                        } else {
                            " "
                        },
                        ver
                    );
                }
                println!(
                    " [{}] Back",
                    if self.cursor as usize == versions.len() {
                        "*"
                    } else {
                        " "
                    }
                );
            }
            SubMenu::SetUp(c) => {
                println!("Barista - Set up mods");
                println!();
                if mods.is_empty() {
                    println!(
                        "Put some mods in your /spicerack/mods\nfolder in order to load them!"
                    );
                    println!();
                    println!("- [*] Back")
                } else {
                    println!("Choose what mods to load with Saltwater");
                    println!("Disabled mods show index --- instead");
                    println!();
                    println!("Press A to enable or disable mods");
                    println!("L/R buttons or Prev/Next to change page");
                    println!("DPad Left/Right to change index");
                    println!("Hold X to scroll indexes faster");
                    println!();
                    println!("Page {} of {}", page + 1, num_pages);
                    for (i, elmt) in mods.iter().enumerate() {
                        println!(
                            "- [{}] {} {}",
                            if self.cursor == i as u32 { "*" } else { " " },
                            match elmt.1 {
                                u16::MAX => "---".to_string(),
                                c =>
                                    if c >= 0x100 && !settings.original_gates {
                                        format!(
                                            "G{}{}",
                                            if c >= 0x110 { c & 3 } else { (c & 0xFF) >> 2 },
                                            if c >= 0x110 {
                                                "P".to_string()
                                            } else if c & 3 == 3 {
                                                "E".to_string()
                                            } else {
                                                (c & 3).to_string()
                                            }
                                        )
                                    } else {
                                        format!("{:03X}", c)
                                    },
                            },
                            // TODO: slot mode
                            if !*c || elmt.1 == u16::MAX {
                                elmt.0.clone()
                            } else if elmt.1 >= 0x100 {
                                String::from("->")
                                    + *match settings.slot_titles {
                                        SlotTitleMode::Internal => SLOT_NAMES_INTERNAL_GATE,
                                        SlotTitleMode::Megamix | SlotTitleMode::Original => {
                                            SLOT_NAMES_GATE
                                        }
                                        SlotTitleMode::Infernal => ["unimplemented uwu"; 0x14],
                                    }
                                    .get((elmt.1 - 0x100) as usize)
                                    .unwrap_or(&"slot not found")
                            } else {
                                String::from("->")
                                    + *match settings.slot_titles {
                                        // TODO: remove &s when they're all done
                                        SlotTitleMode::Internal => &SLOT_NAMES_INTERNAL,
                                        SlotTitleMode::Megamix => SLOT_NAMES_DEFAULT,
                                        SlotTitleMode::Original => SLOT_NAMES_NORETCON,
                                        SlotTitleMode::Infernal => &["unimplemented uwu"; 0x68],
                                    }
                                    .get(elmt.1 as usize)
                                    .unwrap_or(&"slot not found")
                            }
                        );
                    }
                    println!();
                    println!(
                        "- [{}] Prev",
                        if self.cursor == self.cursor_option_len(versions, mods) - 3 {
                            "*"
                        } else {
                            " "
                        }
                    );
                    println!(
                        "- [{}] Next",
                        if self.cursor == self.cursor_option_len(versions, mods) - 2 {
                            "*"
                        } else {
                            " "
                        }
                    );
                    println!();
                    println!(
                        "- [{}] Back",
                        if self.cursor == self.cursor_option_len(versions, mods) - 1 {
                            "*"
                        } else {
                            " "
                        }
                    );
                }
            }
            #[cfg(feature = "audio")]
            SubMenu::Music => {
                println!("Barista - Music");
                println!();
                println!("Current status: very broken");
                println!();
                println!(
                    " [{}] {}",
                    if self.cursor == 0 { "*" } else { " " },
                    if crate::audio().is_playing() {
                        "Disable"
                    } else {
                        "Enable"
                    }
                );
                println!(" [{}] Back", if self.cursor == 1 { "*" } else { " " })
            }
            SubMenu::Options => {
                println!("Barista - Settings");
                println!();
                println!(
                    " [{}] Use 0x100 format for gates: {}",
                    if self.cursor == 0 { "*" } else { " " },
                    if settings.original_gates { "on" } else { "off" }
                );
                println!(
                    " [{}] Slot title mode: {}",
                    if self.cursor == 1 { "*" } else { " " },
                    match settings.slot_titles {
                        SlotTitleMode::Megamix => "Megamix",
                        SlotTitleMode::Original => "Original",
                        SlotTitleMode::Internal => "Internal",
                        SlotTitleMode::Infernal => "Infernal...?",
                    }
                );
                println!();
                println!(" [{}] Back", if self.cursor == 2 { "*" } else { " " })
            }
            #[cfg(debug_assertions)]
            SubMenu::Log => {
                println!("{}", unsafe { &crate::log::LOG });
            }
        }
    }
}
