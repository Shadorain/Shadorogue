use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use specs::prelude::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rltk::RGB;

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Quips {
    pub available: Vec<String>,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Vendor {
    pub categories: Vec<String>,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BlocksTile {}

#[derive(Component, Clone, ConvertSaveload)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct Attributes {
    pub might: Attribute,
    pub fitness: Attribute,
    pub quickness: Attribute,
    pub intelligence: Attribute,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum Skill { Melee, Defense, Magic }

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct Skills {
    pub skills: HashMap<Skill, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub level: i32,
    pub total_weight: f32,
    pub total_initiative_penalty: f32,
    pub gold: f32,
    pub god_mode: bool,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct LootTable {
    pub table: String,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub name: String,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct WantsToApproach {
    pub idx: i32,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct WantsToFlee {
    pub indices: Vec<usize>,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct Chasing {
    pub target: Entity,
}

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize, Debug)]
pub enum Movement {
    Static,
    Random,
    RandomWaypoint { path: Option<Vec<usize>> },
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct MoveMode {
    pub mode: Movement,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct ApplyMove {
    pub dest_idx: usize,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct ApplyTeleport {
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_depth: i32,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct TownPortal {}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct TeleportTo {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub player_only: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SufferDamage {
    pub amount: Vec<(i32, bool)>,
}

impl SufferDamage {
    pub fn new_dmg (store: &mut WriteStorage<SufferDamage>, victim: Entity, amount:i32, from_player: bool) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push((amount, from_player));
        } else {
            let dmg = SufferDamage { amount : vec![(amount, from_player)] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct LightSource {
    pub color: RGB,
    pub range: i32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Initiative {
    pub current: i32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MyTurn {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentChanged {}

/* Serialization */
pub struct SerializeMe;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SerializationHelper {
    pub map : super::map::Map,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct DMSerializationHelper {
    pub map : super::map::MasterDungeonMap,
}

/* Items */
#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct Item {
    pub initiative_penalty: f32,
    pub weight_lbs: f32,
    pub base_value: f32,
}

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize, Debug)]
pub enum MagicItemClass { Common, Rare, Legendary }

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct MagicItem {
    pub class: MagicItemClass,
    pub naming: String,
}

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Debug)]
pub struct InBackpack {
    pub owner : Entity,
}

#[derive(Component, ConvertSaveload, Debug)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity,
}

#[derive(Component, ConvertSaveload, Debug)]
pub struct WantsToUseItem { 
    pub item : Entity,
    pub target : Option<rltk::Point>,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem { 
    pub item : Entity,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct ProvidesHealing { 
    pub heal_amount : i32,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct Ranged { 
    pub range : i32,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct InflictsDamage { 
    pub damage : i32,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct AreaOfEffect { 
    pub radius : i32,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct Confusion { 
    pub turns : i32,
}

/* Equipment */
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Shield, Head, Torso, Legs, Feet, Hands }

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Equippable {
    pub slot : EquipmentSlot,
}

#[derive(Component, Clone, ConvertSaveload)]
pub struct Equipped {
    pub owner : Entity,
    pub slot : EquipmentSlot,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToRemoveEquipment {
    pub item : Entity,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute { Might, Quickness }

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub dmg_n_dice: i32,
    pub dmg_die_type: i32,
    pub dmg_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot: EquipmentSlot,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NaturalAttack {
    pub name: String,
    pub dmg_n_dice: i32,
    pub dmg_die_type: i32,
    pub dmg_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct NaturalAttackDefense {
    pub armor_class: Option<i32>,
    pub attacks: Vec<NaturalAttack>,
}

/* Particles */
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct ParticleLifetime {
    pub lifetime_ms : f32,
}

/* Hunger */
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct HungerClock {
    pub state : HungerState,
    pub duration : i32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ProvidesFood {}

/* Magic Mapper */
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MagicMapper {}

/* Traps */
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Hidden {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EntityMoved {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SingleActivation {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Door {
    pub open: bool,
}
