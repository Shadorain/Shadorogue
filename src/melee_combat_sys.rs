use specs::prelude::*;
use super::{CombatStats, WantsToMelee, SufferDamage, Name, gamelog::GameLog,
    MeleePowerBonus, DefenseBonus, Equipped};

pub struct MeleeCombatSystem { }

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        );

    fn run (&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, combat_stats,
            mut inflict_damage, melee_power_bonus, defense_bonus, equipped) = data;

        for (ent, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity,power_bonus,equipped_by) in (&entities, &melee_power_bonus, &equipped).join() {
                    if equipped_by.owner == ent {
                        offensive_bonus += power_bonus.power;
                    }
                };
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();
                    let mut defensive_bonus = 0;
                    for (_item_entity,defense_bonus,equipped_by) in (&entities, &defense_bonus, &equipped).join() {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    };

                    let dmg = i32::max(0, (stats.power+offensive_bonus) - (target_stats.defense+defensive_bonus));
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
