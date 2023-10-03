use crate::units::Units;

use crate::gpuprops::GPUSprite;

pub enum Terrain {
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

// impl TerrainModifier {
//     fn new() -> Self {
        
//     } 
// }

pub struct Tile {
    terrain: Terrain,
    units: Vec<Units>,
    // buildings: Vec<Buildings>,
    sprite: GPUSprite,      // Could be a vec of GPU sprites all to render overlapped at a location
}

impl Tile {

    pub fn new(terrain:Terrain) -> Self {


        let mut x_idx = 0.0 as f32;
        let mut y_idx = 0.0 as f32;
        let x_width = (16.0/32.0) as f32;
        let y_width = (16.0/32.0) as f32;

        match terrain {   // Decide which sprite to use
            Terrain::Coast =>       {(x_idx, y_idx) = (0.0, 0.0)},
            Terrain::Plain =>       {(x_idx, y_idx) = (0.0, 1.0)},
            Terrain::Mountain =>    {(x_idx, y_idx) = (1.0, 0.0)},
            Terrain::Forest =>      {(x_idx, y_idx) = (1.0, 1.0)},
            _ => ()
        }

        Self {
            terrain,
            units: Vec::default(),
            sprite: GPUSprite { 
                to_region: [32.0, 32.0, 64.0, 64.0],
                from_region: [x_idx*x_width, y_idx*y_width, x_width, y_width],
            },
        }

    }


    pub fn get_sprite(&self) -> GPUSprite {
        self.sprite
    }
}