use rltk::{ RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point };
use serde::{Serialize, Deserialize};
use std::cmp::{min, max};
use specs::prelude::*;
use super::Rect;

pub const MAP_WIDTH : usize  = 80;
pub const MAP_HEIGHT : usize = 50;
pub const MAP_COUNT : usize  = MAP_HEIGHT * MAP_WIDTH;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall, Floor, DownStairs,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles  : Vec<TileType>,
    pub rooms  : Vec<Rect>,
    pub width  : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub depth : i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>,
}

impl Map {
    pub fn xy_idx (&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_horizontal_tunnel (&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1, x2) ..= max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        };
    }

    fn apply_vertical_tunnel (&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1, y2) ..= max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        };
    }

    fn apply_room_to_map (&mut self, room : &Rect) {
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        };
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked (&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        };
    }

    pub fn clear_content_index (&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        };
    }
    
    /// Generates new map that gives random rooms and corridors to join them
    pub fn new_map_rooms_and_corridors (new_depth : i32) -> Map {
        let mut rng = RandomNumberGenerator::new();
        let mut map = Map { 
            tiles  : vec![TileType::Wall; MAP_COUNT],
            rooms  : Vec::new(),
            width  : MAP_WIDTH as i32,
            height : MAP_HEIGHT as i32,
            revealed_tiles : vec![false; MAP_COUNT],
            visible_tiles  : vec![false; MAP_COUNT],
            blocked  : vec![false; MAP_COUNT],
            tile_content  : vec![Vec::new(); MAP_COUNT],
            depth: new_depth,
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE  : i32 = 6;
        const MAX_SIZE  : i32 = 10;

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range (0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        };
        let stairs_position = map.rooms[map.rooms.len()-1].center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions (&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx:usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        /* Normal Directions */
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        /* Diagonals */
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)) };
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx+w)+1, 1.45)) };
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx-w)-1, 1.45)) };
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)) };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

pub fn draw_map (ecs: &World, ctx : &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let bg = RGB::from_f32(0., 0., 0.);

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        /* Render tile dependent on it's type */
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }, TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0., 1.0, 0.);
                }, TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0., 1.0, 0.);
                },
            }
            if !map.visible_tiles[idx] { fg = fg.to_greyscale() }
            ctx.set(x, y, fg, bg, glyph);
        }
        /* Move coords */
        x += 1;
        if x > MAP_WIDTH as i32-1 {
            x = 0;
            y += 1;
        }
    };
}
// Treelikes {{{
// /// Generates a map with solid bounds and 400 randomly placed walls.
// pub fn new_map_test () -> Vec<TileType> {
//     let mut map = vec![TileType::Floor; 80*50];

//     /* Boundary Walls */
//     for x in 0..80 {
//         map[xy_idx(x, 0)] = TileType::Wall;
//         map[xy_idx(x, 49)] = TileType::Wall;
//     }
//     for y in 0..50 {
//         map[xy_idx(0, y)] = TileType::Wall;
//         map[xy_idx(79, y)] = TileType::Wall;
//     }

//     let mut rng = rltk::RandomNumberGenerator::new();

//     for _i in 0..400 {
//         let x = rng.roll_dice(1, 79);
//         let y = rng.roll_dice(1, 49);
//         let idx = xy_idx(x, y);
//         if idx != xy_idx(40, 25) {
//             map[idx] = TileType::Wall;
//         }
//     }
//     map
// }
/* }}} */
