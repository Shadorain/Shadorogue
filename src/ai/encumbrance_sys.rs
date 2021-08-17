use specs::prelude::*;
use crate::{EquipmentChanged, Item, InBackpack, Equipped, Pools, Attributes, gamelog::GameLog};
use std::collections::HashMap;

pub struct EncumbranceSystem {}

impl<'a> System<'a> for EncumbranceSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>
    );

    fn run (&mut self, data: Self::SystemData) {
        let (mut equip_dirty, entities, items, backpacks, wielded, mut pools,
            attributes, player, mut gamelog) = data;
        if equip_dirty.is_empty() { return; }

        let mut to_update: HashMap<Entity, (f32, f32)> = HashMap::new();
        for (ent, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(ent, (0.0, 0.0));
        };
        equip_dirty.clear();

        /* Total of equipped items */
        for (item, equipped) in (&items, &wielded).join() {
            if to_update.contains_key(&equipped.owner) {
                let totals = to_update.get_mut(&equipped.owner).unwrap();
                totals.0 += item.weight_lbs;
                totals.1 += item.initiative_penalty;
            }
        };

        /* Total of carried items */
        for (item, carried) in (&items, &backpacks).join() {
            if to_update.contains_key(&carried.owner) {
                let totals = to_update.get_mut(&carried.owner).unwrap();
                totals.0 += item.weight_lbs;
                totals.1 += item.initiative_penalty;
            }
        };

        /* Apply to Pools */
        for (ent, (weight, initiative)) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*ent) {
                pool.total_weight = *weight;
                pool.total_initiative_penalty = *initiative;

                if let Some(attr) = attributes.get(*ent) {
                    let carry_capacity_lbs = (attr.might.base + attr.might.modifiers) * 15;
                    if pool.total_weight as i32 > carry_capacity_lbs {
                        /* Overburdened */
                        pool.total_initiative_penalty += 4.0;
                        if *ent == *player {
                            gamelog.entries.push("You are overburdened, and suffering an initiative penalty.".to_string());
                        }
                    }
                }
            }
        };
    }
}
