use specs::prelude::*;
use crate::{Initiative, Position, MyTurn, Attributes, RunState, Pools};

pub struct InitiativeSystem {}

impl<'a> System<'a> for InitiativeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, rltk::Point>,
        ReadStorage<'a, Pools>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (mut initiatives, positions, mut turns, entities, mut rng, attributes,
            mut runstate, player, player_pos, pools) = data;

        if *runstate != RunState::Ticking { return; }
        turns.clear();

        for (ent, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;
            if initiative.current < 1 {
                let mut myturn = true;
                initiative.current = 6 + rng.roll_dice(1, 6);

                if let Some(attr) = attributes.get(ent) {
                    initiative.current -= attr.quickness.bonus;
                }

                if let Some(pools) = pools.get(ent) {
                    initiative.current += f32::floor(pools.total_initiative_penalty) as i32;
                }

                if ent == *player {
                    *runstate = RunState::AwaitingInput;
                } else {
                    let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, rltk::Point::new(pos.x, pos.y));
                    if dist > 20.0 { myturn = false; }
                }
                if myturn {
                    turns.insert(ent, MyTurn{}).expect("Unable to insert turn");
                }
            }
        };
    }
}
