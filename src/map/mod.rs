use rltk::{ BaseMap, Algorithm2D, Point };
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

mod tiletype;
pub use tiletype::{TileType, tile_walkable, tile_opaque, tile_cost};
mod themes;
pub use themes::*;
pub mod dungeon;
pub use dungeon::*;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,
    pub name: String,
    pub outdoors: bool,
    pub light: Vec<rltk::RGB>,
}

impl Map {
    pub fn xy_idx (&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !crate::spatial::is_blocked(idx)
    }

    pub fn populate_blocked (&mut self) {
        crate::spatial::populate_blocked_from_map(self);
    }

    pub fn clear_content_index (&mut self) {
        crate::spatial::clear();
    }

    /// Generates new empty map
    pub fn new <S: ToString>(new_depth:i32, width:i32, height:i32, name: S) -> Map {
        let map_tile_count = (width*height) as usize;
        crate::spatial::set_size(map_tile_count);
        Map { 
            tiles: vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            name: name.to_string(),
            outdoors: true,
            light: vec![rltk::RGB::from_f32(0.0, 0.0, 0.0); map_tile_count],
        }
    }
}
    
impl Algorithm2D for Map {
    fn dimensions (&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        let idx_u = idx as usize;
        if idx_u > 0 && idx_u < self.tiles.len() {
            tile_opaque(self.tiles[idx_u]) || self.view_blocked.contains(&idx_u)
        } else {
            true
        }
    }

    fn get_available_exits(&self, idx:usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;
        let tt = self.tiles[idx as usize];

        /* Normal Directions */
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, tile_cost(tt))) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, tile_cost(tt))) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, tile_cost(tt))) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, tile_cost(tt))) };

        /* Diagonals */
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, tile_cost(tt) * 1.45)) };
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, tile_cost(tt) * 1.45)) };
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, tile_cost(tt) * 1.45)) };
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, tile_cost(tt) * 1.45)) };
        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
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
