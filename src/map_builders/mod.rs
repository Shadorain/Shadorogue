#[allow(unused_imports)]
use std::cell::Cell;

use super::{Map, Rect, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER};
mod simple_map;
#[allow(unused_imports)]
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
#[allow(unused_imports)]
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
#[allow(unused_imports)]
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
#[allow(unused_imports)]
use cellular_automata::CellularAutomataBuilder;
mod drunkard;
#[allow(unused_imports)]
use drunkard::DrunkardsWalkBuilder;
mod maze;
#[allow(unused_imports)]
use maze::MazeBuilder;
mod dla;
#[allow(unused_imports)]
use dla::DLABuilder;
mod voronoi;
#[allow(unused_imports)]
use voronoi::VoronoiBuilder;

mod utils;
use utils::*;
use specs::prelude::*;

mod waveform_collapse;
#[allow(unused_imports)]
use waveform_collapse::*;
mod prefab_builder;
#[allow(unused_imports)]
use prefab_builder::*;
mod room_exploder;
#[allow(unused_imports)]
use room_exploder::*;
mod room_corner_rounding;
#[allow(unused_imports)]
use room_corner_rounding::*;
mod room_sorter;
#[allow(unused_imports)]
use room_sorter::*;
mod room_draw;
#[allow(unused_imports)]
use room_draw::*;
mod door_placement;
#[allow(unused_imports)]
use door_placement::*;

mod rooms_corridors_dogleg;
#[allow(unused_imports)]
use rooms_corridors_dogleg::*;
mod rooms_corridors_bsp;
#[allow(unused_imports)]
use rooms_corridors_bsp::*;
mod rooms_corridors_nearest;
#[allow(unused_imports)]
use rooms_corridors_nearest::*;
mod rooms_corridors_lines;
#[allow(unused_imports)]
use rooms_corridors_lines::*;
mod rooms_corridor_spawner;
#[allow(unused_imports)]
use rooms_corridor_spawner::*;

mod room_based_stairs;
mod room_based_spawner;
mod room_based_starting_position;
mod area_starting_points;
mod distant_exit;
mod cull_unreachable;
mod voronoi_spawning;
#[allow(unused_imports)]
use room_based_stairs::*;
#[allow(unused_imports)]
use room_based_spawner::*;
#[allow(unused_imports)]
use room_based_starting_position::*;
#[allow(unused_imports)]
use area_starting_points::*;
#[allow(unused_imports)]
use distant_exit::*;
#[allow(unused_imports)]
use cull_unreachable::*;
#[allow(unused_imports)]
use voronoi_spawning::*;

pub trait InitialMapBuilder {
    fn build_map (&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map (&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32,
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderMap {
    fn take_snapshot (&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            };
            self.history.push(snapshot);
        }
    }
}

impl BuilderChain {
    pub fn new (new_depth:i32, width:i32, height:i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height,
            },
        }
    }

    pub fn start_with (&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        }
    }

    pub fn with (&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }
    
    pub fn build_map (&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                starter.build_map(rng, &mut self.build_data);
            },
        }

        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        };
    }

    pub fn spawn_entities (&mut self, ecs: &mut World) {
        for ent in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&ent.0, &ent.1));
        };
    }
}

fn random_start_position (rng: &mut rltk::RandomNumberGenerator) -> (XStart, YStart) {
    let x;
    let xroll = rng.roll_dice(1, 3);
    match xroll {
        1 => x = XStart::LEFT,
        2 => x = XStart::CENTER,
        _ => x = XStart::RIGHT
    }

    let y;
    let yroll = rng.roll_dice(1, 3);
    match yroll {
        1 => y = YStart::BOTTOM,
        2 => y = YStart::CENTER,
        _ => y = YStart::TOP
    }
    (x, y)
}

fn random_room_builder (rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new()),
    }

    if build_roll != 3 {
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL)),
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng.roll_dice(1, 2);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            _ => builder.with(BspCorridors::new()),
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {},
        }
    }
    let corridor_roll = rng.roll_dice(1, 4);
    match corridor_roll {
        1 => builder.with(DoglegCorridors::new()),
        2 => builder.with(NearestCorridors::new()),
        3 => builder.with(StraightLineCorridors::new()),
        _ => builder.with(BspCorridors::new()),
    }

    let cspawn_roll = rng.roll_dice(1, 2);
    if cspawn_roll == 1 {
        builder.with(CorridorSpawner::new());
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        },
    }

    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new()),
    }
}

fn random_shape_builder (rng: &mut rltk::RandomNumberGenerator, builder: &mut BuilderChain) {
    let builder_roll = rng.roll_dice(1, 16);
    match builder_roll {
        1  => builder.start_with(CellularAutomataBuilder::new()),
        2  => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3  => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4  => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5  => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6  => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7  => builder.start_with(MazeBuilder::new()),
        8  => builder.start_with(DLABuilder::walk_inwards()),
        9  => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiBuilder::pythagoras()),
        13 => builder.start_with(VoronoiBuilder::manhattan()),
        _  => builder.start_with(PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED)),
    }
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}

pub fn random_builder (new_depth:i32, rng: &mut rltk::RandomNumberGenerator, width:i32, height:i32) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height);
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder),
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
        let (start_x, start_y) = random_start_position(rng);
        builder.with(AreaStartingPosition::new(start_x, start_y));
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }
    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());

    builder
}
