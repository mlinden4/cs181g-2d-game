use crate::units::Units;

enum Terrain {
    Coast,
    Plain,
    Mountain,
    Forest
}

struct TerrainModifier {
    terrain_type: Terrain,
    movement: usize,
    // attack: usize,
    defense: usize,
}

impl TerrainModifier {
    fn new() -> Self {
        
    } 
}

struct Tile {
    terrain: Terrain,
    units: Vec<Units>,
    // buildings: Vec<Buildings>,
    
}