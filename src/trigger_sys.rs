use specs::prelude::*;
use super::{EntryTrigger, EntityMoved, Position, Hidden, Map, Name, gamelog::GameLog,
    InflictsDamage, ParticleBuilder, SufferDamage, SingleActivation, TeleportTo,
    Entity, ApplyTeleport
};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, SingleActivation>,
        ReadStorage<'a, TeleportTo>,
        WriteStorage<'a, ApplyTeleport>,
        WriteExpect<'a, Entity>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, mut hidden, names,
            entities, mut log, inflicts_damage, mut particle_builder, mut suffer_damage,
            single_activation, teleporters, mut apply_teleport, player_entity) = data;

        /* Iterate entities that moved and their final position */
        let mut remove_entities : Vec<Entity> = Vec::new();
        for (ent,mut _moved,pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            crate::spatial::for_each_tile_content (idx, |entity_id| {
                if ent != entity_id {
                    let maybe_trigger = entry_trigger.get(entity_id);
                    match maybe_trigger {
                        None => {},
                        Some(_trigger) => {
                            /* triggered it */
                            let name = names.get(entity_id);
                            if let Some(name) = name {
                                log.entries.push(format!("{} triggers!", &name.name));
                            }

                            hidden.remove(entity_id);

                            /* If damage */
                            let dmg = inflicts_damage.get(entity_id);
                            if let Some(dmg) = dmg {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0);
                                SufferDamage::new_dmg(&mut suffer_damage, ent, dmg.damage, false);
                            }

                            /* If teleporter */
                            if let Some(teleport) = teleporters.get(entity_id) {
                                if (teleport.player_only && ent == *player_entity) || !teleport.player_only {
                                    apply_teleport.insert(ent, ApplyTeleport {
                                        dest_x : teleport.x,
                                        dest_y : teleport.y,
                                        dest_depth : teleport.depth
                                    }).expect("Unable to insert");
                                }
                            }

                            /* If only used once */
                            let sa = single_activation.get(entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(entity_id);
                            }
                        },
                    }
                }
            });
        };
        /* Removes single activation traps */
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        };
        entity_moved.clear();
    }
}
