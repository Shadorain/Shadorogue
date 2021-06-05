use specs::prelude::*;
use super::{CombatStats, WantsToMelee, SufferDamage, Name, gamelog::GameLog};

pub struct MeleeCombatSystem { }

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        );

    fn run (&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, combat_stats, mut inflict_damage) = data;

        for (_ent, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();
                    let dmg = i32::max(0, stats.power - target_stats.defense);
                    if dmg == 0 {
                        log.entries.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.push(format!("{} hit {}, for {} hp.", &name.name, &target_name.name, dmg));
                        SufferDamage::new_dmg(&mut inflict_damage, wants_melee.target, dmg);
                    }
                }
            }
        };
        wants_melee.clear();
    }
}
