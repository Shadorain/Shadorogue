use std::sync::Mutex;
use serde::Deserialize;

mod rawmaster;
pub use rawmaster::*;

mod item_structs;
mod mob_structs;
mod prop_structs;
mod spawn_table_structs;
mod loot_structs;
mod faction_structs;
use item_structs::*;
use mob_structs::*;
use prop_structs::*;
use spawn_table_structs::*;
use loot_structs::*;
pub use faction_structs::*;

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_tables: Vec<LootTable>,
    pub faction_table: Vec<FactionInfo>,
}

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

pub fn load_raws () {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");
    let raw_data = rltk::embedding::EMBED.lock()
        .get_resource("../../raws/spawns.json".to_string()).unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string");

    let decoder: Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");
    rltk::console::log(format!("{:?}", decoder));

    RAWS.lock().unwrap().load(decoder);
}
