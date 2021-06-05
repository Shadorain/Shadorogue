use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{min, max};
use super::{Player, State, Map, Viewshed, RunState, CombatStats, WantsToMelee,
    Position, Item, gamelog::GameLog, WantsToPickupItem, TileType, Monster};

pub fn try_move_player (delta_x: i32, delta_y: i32, ecs: &mut World) {
    let players = ecs.write_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (ent, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x+delta_x < 1 || pos.x+delta_x > map.width-1 || pos.y+delta_y < 1 || pos.y+delta_y > map.height-1 { return; }
        let dest_idx = map.xy_idx(pos.x+delta_x, pos.y+delta_y);
        
        for potential_target in map.tile_content[dest_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                    /* Attack it */
                    wants_to_melee.insert(ent, WantsToMelee { target: *potential_target }).expect("Add target failed");
                    return; /* Prevent move after attacking */
            }
        };

        if !map.blocked[dest_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            viewshed.dirty = true;

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    };
}

fn get_item (ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item : Option<Entity> = None;
    for (item_ent,_item,pos) in (&entities, &items, &positions).join() {
        if pos.x == player_pos.x && pos.y == player_pos.y {
            target_item = Some(item_ent);
        }
    };
    
    match target_item {
        None => gamelog.entries.push("There is nothing here to pickup.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        },
    }

}

pub fn try_next_level (ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

fn skip_turn (ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {},
                Some(_) => { can_heal = false; }
            }
        };
    };

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp+1, player_hp.max_hp);
    }
    RunState::PlayerTurn
}

pub fn player_input (gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput } /* null */
        Some(key) => match key {
            /* Normal Movement */
            VirtualKeyCode::Left  
            | VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right
            | VirtualKeyCode::L => try_move_player(1,  0, &mut gs.ecs),
            VirtualKeyCode::Up
            | VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down
            | VirtualKeyCode::J => try_move_player(0,  1, &mut gs.ecs),

            /* Diagonals */
            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player( 1, -1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player( 1,  1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1,  1, &mut gs.ecs),

            /* Action Keys */
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
            /* Level Change */
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            },

            /* Save & Quit */
            VirtualKeyCode::Escape => return RunState::SaveGame,

            _ => { return RunState::AwaitingInput }
        },
    }
    RunState::PlayerTurn
}