use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    UpStairs,
    Road,
    Grass,
    Gravel,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Stalactite,
    Stalagmite,
}

pub fn tile_walkable (tt: TileType) -> bool {
    match tt {
        TileType::Floor | TileType::DownStairs | TileType::UpStairs | TileType::Grass |
            TileType::Road | TileType::ShallowWater | TileType::WoodFloor |
            TileType::Bridge | TileType::Gravel => true,
        _ => false,
    }
}

pub fn tile_opaque (tt: TileType) -> bool {
    match tt {
        TileType::Wall | TileType::Stalactite | TileType::Stalagmite => true,
        _ => false,
    }
}

pub fn tile_cost (tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.9,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}
