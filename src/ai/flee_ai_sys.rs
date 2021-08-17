use specs::prelude::*;
use crate::{MyTurn, WantsToFlee, Position, Map, ApplyMove};

pub struct FleeAI {}

impl<'a> System<'a> for FleeAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToFlee>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (mut turns, mut wants_flee, positions, mut map, 
            entities, mut apply_move) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (ent, pos, flee, _myturn) in (&entities, &positions, &wants_flee, &turns).join() {
            turn_done.push(ent);
            let my_idx = map.xy_idx(pos.x, pos.y);
            map.populate_blocked();
            let flee_map = rltk::DijkstraMap::new(map.width as usize, map.height as usize, &flee.indices, &*map, 100.0);
            let flee_target = rltk::DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
            if let Some(flee_target) = flee_target {
                if !crate::spatial::is_blocked(flee_target as usize) {
                    apply_move.insert(ent, ApplyMove{ dest_idx : flee_target }).expect("Unable to insert");
                    turn_done.push(ent);
                }
            }
        };
        wants_flee.clear();

        for done in turn_done.iter() {
            turns.remove(*done);
        };
    }
}
