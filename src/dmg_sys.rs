use specs::prelude::*;
use super::{CombatStats, SufferDamage, Player, Name, gamelog::GameLog, RunState, Position, Map};

pub struct DamageSystem { }

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>
        );

    fn run (&mut self, data : Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities) = data;
        for (ent,mut stats,dmg) in (&entities, &mut stats, &damage).join() {
            stats.hp -= dmg.amount.iter().sum::<i32>();
            let pos = positions.get(ent);
            if let Some(pos) = pos {
                let idx = map.xy_idx(pos.x, pos.y);
                map.bloodstains.insert(idx);
            }
        };
        damage.clear();
    }
}

pub fn delete_the_dead (ecs : &mut World) {
    let mut dead : Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();
        for (ent, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
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

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    };
}
