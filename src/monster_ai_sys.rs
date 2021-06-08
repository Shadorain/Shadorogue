use specs::prelude::*;
use rltk::Point;
use super::{Viewshed, Monster, Map, Position, WantsToMelee, RunState, Confusion};

pub struct MonsterAI { }

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
    );

    fn run (&mut self, data : Self::SystemData) {
        let (mut map, ppos, player_entity, runstate, entities, mut viewshed, monster,
            mut position, mut wants_to_melee, mut confused) = data;
        if *runstate != RunState::MonsterTurn { return; }

        for (ent,mut viewshed,_monster,mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
            let mut can_act = true;
            
            let is_confused = confused.get_mut(ent);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(ent);
                }
                can_act = false;
            }

            if can_act {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *ppos);
                if dist < 1.5 { /* ATTACK */
                    wants_to_melee.insert(ent, WantsToMelee { target: *player_entity }).expect("Unable to insert attack");
                } else if viewshed.visible_tiles.contains(&*ppos) {
                    /* Path to player */
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(ppos.x, ppos.y),
                        &*map);
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        };
    }
}
