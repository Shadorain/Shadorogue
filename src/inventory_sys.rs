use specs::prelude::*;
use super::{Map, Name, gamelog::GameLog, WantsToUseItem, ProvidesHealing, Pools,
    WantsToPickupItem, WantsToDropItem, Position, InBackpack, Consumable, SufferDamage,
    InflictsDamage, AreaOfEffect, Confusion, Equippable, Equipped, WantsToRemoveEquipment,
    ParticleBuilder, ProvidesFood, HungerState, HungerClock, MagicMapper, RunState,
    EquipmentChanged, TownPortal
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, EquipmentChanged>,
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names,
            mut backpack, mut dirty) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            dirty.insert(pickup.collected_by, EquipmentChanged{}).expect("Unable to insert");

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
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, TownPortal>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, map, entities, mut wants_use, names,
            consumables, healing, inflict_damage, mut combat_stats,
            mut suffer_damage, aoe, mut confused, equippable, mut equipped,
            mut backpack, mut particle_builder, positions, provides_food,
            mut hunger_clock, magic_mapper, mut runstate, mut dirty, town_portal) = data;
        
        for (ent,useitem) in (&entities, &wants_use).join() {
            dirty.insert(ent, EquipmentChanged{}).expect("Unable to insert");
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
                            crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                        },
                        Some(area_effect) => {
                            /* AOE */
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                crate::spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                                particle_builder.request(tile_idx.x, tile_idx.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('░'), 200.0);
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
            /* Magic Mapper */
            let is_mapper = magic_mapper.get(useitem.item);
            match is_mapper {
                None => {},
                Some(_) => {
                    used_item = true;
                    gamelog.entries.push("The map is revealed to you!".to_string());
                    *runstate = RunState::MagicMapReveal { row: 0 };
                },
            }
            /* Town portal */
            if let Some(_townportal) = town_portal.get(useitem.item) {
                if map.depth == 1 {
                    gamelog.entries.push("You are already in town, so the scroll does nothing.".to_string());
                } else {
                    used_item = true;
                    gamelog.entries.push("You are teleported back to town!".to_string());
                    *runstate = RunState::TownPortal;
                }
            }
            /* Food!!! */
            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {},
                Some(_) => {
                    used_item = true;
                    let target = targets[0];
                    let hc = hunger_clock.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!("You eat the {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }
            /* Healing */
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {},
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hit_points.current = i32::min(stats.hit_points.max, stats.hit_points.current+healer.heal_amount);
                            if ent == *player_entity {
                                gamelog.entries.push(format!("You drank the {}, healing {} hp.",
                                        names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                            used_item = true;
                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::GREEN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('♥'), 200.0);
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
                        SufferDamage::new_dmg(&mut suffer_damage, *mob, damage.damage, true);
                        if ent == *player_entity {
                            let item_name = names.get(useitem.item).unwrap();
                            let mob_name = names.get(*mob).unwrap();
                            gamelog.entries.push(format!("You used {} on {}, inflicting {} hp.",
                                    item_name.name, mob_name.name, damage.damage));
                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::RED), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                            }
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
                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::MAGENTA), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
                                }
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
        WriteStorage<'a, EquipmentChanged>,
        );

    fn run (&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions,
            mut backpack, mut dirty) = data;
        
        for (ent,to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position { x:0, y:0 };
            {
                let dropped_pos = positions.get(ent).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position { x:dropper_pos.x, y:dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty.insert(ent, EquipmentChanged{}).expect("Unable to insert");

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
