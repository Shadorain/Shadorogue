use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use specs::prelude::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};
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
pub struct Player { }

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Monster { }

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct Name {
    pub name : String,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BlocksTile { }

#[derive(Component, Clone, ConvertSaveload)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct WantsToMelee {
    pub target : Entity,
}

#[derive(Component, Clone, ConvertSaveload, Debug)]
pub struct SufferDamage {
    pub amount : Vec<i32>,
}
impl SufferDamage {
    pub fn new_dmg (store: &mut WriteStorage<SufferDamage>, victim: Entity, amount:i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

/* Serialization */
pub struct SerializeMe;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct SerializationHelper {
    pub map : super::map::Map,
}

/* Items */
#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct Item { }

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct Consumable { }

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
pub enum EquipmentSlot { Melee, Shield }

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Equippable {
    pub slot : EquipmentSlot,
}

#[derive(Component, Clone, ConvertSaveload)]
pub struct Equipped {
    pub owner : Entity,
    pub slot : EquipmentSlot,
}

#[derive(Component, Clone, ConvertSaveload)]
pub struct MeleePowerBonus {
    pub power : i32,
}

#[derive(Component, Clone, ConvertSaveload)]
pub struct DefenseBonus {
    pub defense : i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToRemoveEquipment {
    pub item : Entity,
}
