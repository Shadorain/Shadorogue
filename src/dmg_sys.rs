use specs::prelude::*;
use super::{Pools, SufferDamage, Player, Name, gamelog::GameLog, RunState, Position, Map,
    InBackpack, Equipped, LootTable, Attributes, ParticleBuilder};
use crate::gamesys::{player_hp_at_level, mana_at_level};

pub struct DamageSystem { }

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, ParticleBuilder>,
        ReadExpect<'a, rltk::Point>,
    );

    fn run (&mut self, data : Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities, player, attributes,
            mut log, mut particles, player_pos) = data;
        let mut xp_gain = 0;
        let mut gold_gain = 0.0f32;

        for (ent,mut stats,damage) in (&entities, &mut stats, &damage).join() {
            for dmg in damage.amount.iter() {
                if !stats.god_mode {
                    stats.hit_points.current -= dmg.0;
                }
                let pos = positions.get(ent);
                if let Some(pos) = pos {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.bloodstains.insert(idx);
                }
                if stats.hit_points.current < 1 && dmg.1 {
                    xp_gain += stats.level * 100;
                    gold_gain += stats.gold;
                    if let Some(pos) = pos {
                        let idx = map.xy_idx(pos.x, pos.y);
                        crate::spatial::remove_entity(ent, idx);
                    }
                }
            };
        };
        if xp_gain != 0 || gold_gain != 0.0 {
            let mut player_stats = stats.get_mut(*player).unwrap();
            let player_attributes = attributes.get(*player).unwrap();
            player_stats.xp += xp_gain;
            player_stats.gold += gold_gain;
            if player_stats.xp >= player_stats.level * 1000 {
                /* Gone up a level! */
                player_stats.level += 1;
                player_stats.hit_points.max = player_hp_at_level(player_attributes.fitness.base +
                    player_attributes.fitness.modifiers, player_stats.level);
                player_stats.hit_points.current = player_stats.hit_points.max;
                player_stats.mana.max = mana_at_level(player_attributes.intelligence.base +
                    player_attributes.intelligence.modifiers, player_stats.level);
                player_stats.mana.current = player_stats.mana.max;
                log.entries.push(format!("Congratulations!!! You are now level {}!", player_stats.level));
                for i in 0..3 {
                    if player_pos.y - i > 1 {
                        particles.request(player_pos.x, player_pos.y-i, rltk::RGB::named(rltk::GOLD), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('???'), 1000.0);
                        particles.request(player_pos.x-1, player_pos.y-i+1, rltk::RGB::named(rltk::GOLD), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('???'), 1000.0);
                        particles.request(player_pos.x+1, player_pos.y-i+1, rltk::RGB::named(rltk::GOLD), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('???'), 1000.0);
                    }
                };
            }
        }
        damage.clear();
    }
}

pub fn delete_the_dead (ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
        for (ent, stats) in (&entities, &combat_stats).join() {
            if stats.hit_points.current < 1 {
                let player = players.get(ent);
                match player {
                    None => {
                        let victim_name = names.get(ent);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", &victim_name.name));
                        }
                        dead.push(ent) 
                    },
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        };
    }

    /* Drop everything held by dead person */
    let mut to_spawn: Vec<(String, Position)> = Vec::new();
    {
        let mut to_drop: Vec<(Entity, Position)> = Vec::new();
        let entities = ecs.entities();
        let mut equipped = ecs.write_storage::<Equipped>();
        let mut carried = ecs.write_storage::<InBackpack>();
        let mut positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();
        let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
        for victim in dead.iter() {
            let pos = positions.get(*victim);
            for (ent, equipped) in (&entities, &equipped).join() {
                if equipped.owner == *victim {
                    let pos = positions.get(*victim);
                    if let Some(pos) = pos {
                        to_drop.push((ent, pos.clone()));
                    }
                }
            };
            for (ent, backpack) in (&entities, &carried).join() {
                if backpack.owner == *victim {
                    let pos = positions.get(*victim);
                    if let Some(pos) = pos {
                        to_drop.push((ent, pos.clone()));
                    }
                }
            };

            if let Some(table) = loot_tables.get(*victim) {
                let drop_finder = crate::raws::get_item_drop(&crate::raws::RAWS.lock().unwrap(),
                    &mut rng, &table.table);
                if let Some(tag) = drop_finder {
                    if let Some(pos) = pos {
                        to_spawn.push((tag, pos.clone()));
                    }
                }
            }
        };

        for drop in to_drop.iter() {
            equipped.remove(drop.0);
            carried.remove(drop.0);
            positions.insert(drop.0, drop.1.clone()).expect("Unable to insert position");
        };
    }

    for drop in to_spawn.iter() {
        crate::raws::spawn_named_item(&crate::raws::RAWS.lock().unwrap(), ecs,
            &drop.0, crate::raws::SpawnType::AtPosition { x : drop.1.x, y : drop.1.y });
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    };
}
