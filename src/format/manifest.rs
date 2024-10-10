use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

//TODO: Serialize isn't required, it's just on for testing

//TODO: option for whether or not you want to create extra museum slots or rows

//TODO: figure out how to do prologue jingles

#[derive(Serialize, Deserialize, Debug)]
pub struct ModManifest {
    pub info: ModInfo,
    pub requirements: Requirements,
    pub code: Option<ModCode>,
    pub tickflow: ModTickflows,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModInfo {
    pub name: String,
    pub author: String,
    pub version: String, //TODO: simple versioning
    pub description: Option<String>,
    #[serde(default)]
    pub contributors: Vec<String>,
    pub languages: Vec<RHLanguage>,
}

//TODO: future languages supported
#[derive(Serialize, Deserialize, Debug)]
pub enum RHLanguage {
    #[serde(rename = "jp_jp")]
    Japanese,
    #[serde(rename = "us_en")]
    USEnglish,
    #[serde(rename = "eu_en")]
    EUEnglish,
    #[serde(rename = "eu_fr")]
    French,
    #[serde(rename = "eu_de", alias = "eu_ge")]
    German,
    #[serde(rename = "eu_es")]
    Spanish,
    #[serde(rename = "eu_it")]
    Italian,
    #[serde(rename = "kr_kr")]
    Korean,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Requirements {
    pub version: String, //TODO: simple versioning
    #[serde(flatten)]
    pub dependencies: HashMap<String, String>, //TODO: simple versioning
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModCode {
    //TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModTickflows {
    #[serde(default)]
    pub game: Vec<TickflowGame>,
    #[serde(default)]
    pub land: Vec<TickflowLand>,
    #[serde(default)]
    pub tower: Vec<TickflowTower>,
    #[serde(default)]
    pub gate: Vec<TickflowGate>,
    #[serde(default)]
    pub patch: Vec<TickflowPatch>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShortGame {
    pub name: String,
    pub tickflow: PathBuf,
    pub fs: Option<PathBuf>,

    // extra name overrides
    pub music: Option<PathBuf>,
    pub prologue: Option<PathBuf>,
    pub epilogue: Option<PathBuf>,
    pub text_prefix: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickflowGame {
    #[serde(flatten)]
    pub game: ShortGame,

    pub default_index: u32, //TODO: convert from string?

    #[serde(default)]
    pub fixed: bool, //TODO: maybe disable this?
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickflowLand {
    pub name: String,          // for land assets
    pub fs: Option<PathBuf>,   //TODO: remove?
    pub default_position: u32, //TODO: make enum

    #[serde(default)]
    pub fixed: bool,

    pub game1: ShortGame,
    pub game2: ShortGame,
    pub game3: ShortGame,
    pub game4: ShortGame,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickflowTower {
    pub name: String,          // for tower assets
    pub fs: Option<PathBuf>,   //TODO: remove?
    pub default_position: u32, //TODO: make enum

    #[serde(default)]
    pub fixed: bool,

    pub game1: ShortGame,
    pub game2: ShortGame,
    pub game3: ShortGame,
    pub game4: ShortGame,
    pub remix: ShortGame,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickflowGate {
    pub name: String,          // game will derive everything from the one name
                               // TODO: will this need a second name for overworld assets?
    pub fs: Option<PathBuf>,
    pub default_position: u32, //TODO: make an enum

    #[serde(default)]
    pub fixed: bool,

    // TODO: substruct?
    pub tickflow_easy: PathBuf,
    pub tickflow_medium: PathBuf,
    pub tickflow_hard: PathBuf,
    pub tickflow_endless: PathBuf,
    pub tickflow_practice: PathBuf,

    // extra name overrides
    pub music: Option<PathBuf>,
    pub prologue: Option<PathBuf>,
    pub text_prefix: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TickflowPatch {
    pub tickflow: PathBuf,
    pub pos_us: Option<u32>,
    pub pos_eu: Option<u32>,
    pub pos_kr: Option<u32>,
    pub pos_jp: Option<u32>,
}
