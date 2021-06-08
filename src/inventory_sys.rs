use specs::prelude::*;
use super::{Map, Name, gamelog::GameLog, WantsToUseItem, ProvidesHealing, CombatStats,
    WantsToPickupItem, WantsToDropItem, Position, InBackpack, Consumable, SufferDamage,
    InflictsDamage, AreaOfEffect, Confusion, Equippable, Equipped, WantsToRemoveEquipment,
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        };
        wants_pickup.clear();
    }
}

pub struct ItemUseSystem { }

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        );

    fn run (&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, map, entities, mut wants_use, names,
            consumables, healing, inflict_damage, mut combat_stats,
            mut suffer_damage, aoe, mut confused, equippable, mut equipped,
            mut backpack) = data;
        
        for (ent,useitem) in (&entities, &wants_use).join() {
            let mut used_item = true;

            /* Targeting */
            let mut targets : Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push(*player_entity); },
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            /* Single target in tile */
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            };
                        },
                        Some(area_effect) => {
                            /* AOE */
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                };
                            };
                        },
                    }
                },
            }
            /* If equippable: equip it and unequip what was in the slot */
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {},
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    /* Remove previous item in slot */
                    let mut to_unequip : Vec<Entity> = Vec::new();
                    for (item_entity,already_equipped,name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequipped {}.", name.name));
                            }
                        }
                    };
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack { owner: target }).expect("Unable to insert backpack entry");
                    };

                    /* Wield the item */
                    equipped.insert(useitem.item, Equipped { owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equipped {}.", names.get(useitem.item).unwrap().name))
                    }
                },
            }
            /* Healing */
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {},
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp+healer.heal_amount);
                            if ent == *player_entity {
                                gamelog.entries.push(format!("You drank the {}, healing {} hp.",
                                        names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                        }
                    };
                }
            }
            /* Damaging */
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {},
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_dmg(&mut suffer_damage, *mob, damage.damage);
                        if ent == *player_entity {
                            let item_name = names.get(useitem.item).unwrap();
                            let mob_name = names.get(*mob).unwrap();
                            gamelog.entries.push(format!("You used {} on {}, inflicting {} hp.",
                                    item_name.name, mob_name.name, damage.damage));
                        }
                        used_item = true;
                    };
                }
            }
            /* Confusion */
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {},
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if ent == *player_entity {
                                let item_name = names.get(useitem.item).unwrap();
                                let mob_name = names.get(*mob).unwrap();
                                gamelog.entries.push(format!("You used {} on {}, inflicting confusion.",
                                        item_name.name, mob_name.name));
                            }
                            used_item = true;
                        };
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion { turns: mob.1 }).expect("Unable to insert status");
            };
            /* if consumable, delete on use */
            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {},
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete Failed");
                    },
                }
            }
        };
        wants_use.clear();
    }
}

pub struct ItemDropSystem { }

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        );

    fn run (&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;
        
        for (ent,to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position { x:0, y:0 };
            {
                let dropped_pos = positions.get(ent).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position { x:dropper_pos.x, y:dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if ent == *player_entity {
                gamelog.entries.push(format!("You dropped the {}.", names.get(to_drop.item).unwrap().name));
            }
        };
        wants_drop.clear();
    }
}

pub struct EquipmentRemoveSystem { }

impl<'a> System<'a> for EquipmentRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveEquipment>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        );

    fn run (&mut self, data : Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack) = data;
        for (ent,to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack.insert(to_remove.item, InBackpack { owner: ent }).expect("Unable to insert backpack");
        };
        wants_remove.clear();
    }
}
