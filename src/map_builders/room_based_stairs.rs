use super::{MetaMapBuilder, BuilderMap, TileType};

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map (&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
    #[allow(dead_code)]
    pub fn new () -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs{})
    }

    fn build (&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let stairs_pos = rooms[rooms.len()-1].center();
            let stairs_idx = build_data.map.xy_idx(stairs_pos.0, stairs_pos.1);
            build_data.map.tiles[stairs_idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("Room based stairs only works after rooms have been created");
        }
    }
}
