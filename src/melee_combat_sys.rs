use specs::prelude::*;
use super::{Attributes, Skills, Skill, WantsToMelee, Name, SufferDamage, gamelog::GameLog,
    particle_sys::ParticleBuilder, Position, HungerClock, HungerState, Pools, skill_bonus,
    Equipped, MeleeWeapon, WeaponAttribute, EquipmentSlot, Wearable, NaturalAttackDefense
};

pub struct MeleeCombatSystem { }

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, NaturalAttackDefense>,
        ReadExpect<'a, Entity>,
    );

    fn run (&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, attributes, skills,
            mut inflict_damage, mut particle_builder, positions, hunger_clock,
            pools, mut rng, equipped_items, melee_weapons, wearables, natural,
            player_entity) = data;

        for (ent, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in
            (&entities, &wants_melee, &names, &attributes, &skills, &pools).join()
        {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();
                let mut weapon_info = MeleeWeapon {
                    attribute: WeaponAttribute::Might,
                    hit_bonus: 0,
                    dmg_n_dice: 1,
                    dmg_die_type: 4,
                    dmg_bonus: 0,
                };

                if let Some(nat) = natural.get(ent) {
                    if !nat.attacks.is_empty() {
                        let attack_idx = if nat.attacks.len() == 1 { 0 }
                            else { rng.roll_dice(1, nat.attacks.len() as i32) as usize - 1 };
                            weapon_info.hit_bonus = nat.attacks[attack_idx].hit_bonus;
                            weapon_info.dmg_n_dice = nat.attacks[attack_idx].dmg_n_dice;
                            weapon_info.dmg_die_type = nat.attacks[attack_idx].dmg_die_type;
                            weapon_info.dmg_bonus = nat.attacks[attack_idx].dmg_bonus;
                    }
                }

                for (wielded,melee) in (&equipped_items, &melee_weapons).join() {
                    if wielded.owner == ent && wielded.slot == EquipmentSlot::Melee {
                        weapon_info = melee.clone();
                    }
                };

                let natural_roll = rng.roll_dice(1, 20);
                let attribute_hit_bonus = if weapon_info.attribute == WeaponAttribute::Might {
                    attacker_attributes.might.bonus
                } else { attacker_attributes.quickness.bonus };
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = weapon_info.hit_bonus; /* TODO */
                let mut status_hit_bonus = 0;
                if let Some(hc) = hunger_clock.get(ent) {
                    if hc.state == HungerState::WellFed {
                        status_hit_bonus += 1;
                    }
                }
                let modified_hit_roll = natural_roll+attribute_hit_bonus+skill_hit_bonus+weapon_hit_bonus+status_hit_bonus;

                let mut armor_item_bonus_f = 0.0;
                for (wielded,armor) in (&equipped_items, &wearables).join() {
                    if wielded.owner == wants_melee.target {
                        armor_item_bonus_f += armor.armor_class;
                    }
                };
                let base_armor_class = match natural.get(wants_melee.target) {
                    None => 10,
                    Some(nat) => nat.armor_class.unwrap_or(10),
                };
                let armor_quickness_bonus = target_attributes.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armor_item_bonus = armor_item_bonus_f as i32;
                let armor_class = base_armor_class+armor_quickness_bonus+armor_skill_bonus+armor_item_bonus;

                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    let base_dmg = rng.roll_dice(weapon_info.dmg_n_dice, weapon_info.dmg_die_type);
                    let attr_dmg_bonus = attacker_attributes.might.bonus;
                    let skill_dmg_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_dmg_bonus = weapon_info.dmg_bonus;
                    let dmg = i32::max(0, base_dmg+attr_dmg_bonus+skill_hit_bonus+skill_dmg_bonus+weapon_dmg_bonus);
                    SufferDamage::new_dmg(&mut inflict_damage, wants_melee.target, dmg, ent == *player_entity);
                    log.entries.push(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, dmg));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                } else if natural_roll == 1 {
                    log.entries.push(format!("{} considers attacking {}, but misjudges the timing.", &name.name, &target_name.name));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::BLUE),
                            rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                } else {
                    log.entries.push(format!("{} attacks {}, but cannot connect.", &name.name, &target_name.name));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::CYAN),
                            rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                    }
                }
            }
        };
        wants_melee.clear();
    }
}
