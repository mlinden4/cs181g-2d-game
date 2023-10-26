use crate::units::Unit;
use bytemuck;
use chickenwire::prelude::HexGrid;

use winit::event::VirtualKeyCode;

use crate::gpuprops::GPUSprite;

#[derive(Clone, Copy, PartialEq)]
pub enum Terrain {
    Coast,
    Plain,
    Mountain,
    Forest
}

#[derive(PartialEq, Clone)]
pub struct TerrainModifier {
    pub terrain_type: Terrain,
    pub movement: usize,
    // attack: usize,
    pub defense: usize,
}

// impl TerrainModifier {
//     fn new() -> Self {
        
//     } 
// }


#[derive(Clone, Copy)]
pub struct Tile {
    pub terrain: Terrain,
    pub mountain: bool,
    // units: Vec<Unit>, // create a vec of units, keep track of what tile a unit is on
    // buildings: Vec<Buildings>,
    // sprite: GPUSprite,    Render elsewhere  // Could be a vec of GPU sprites all to render overlapped at a location
}

impl Tile {

    pub fn new(terrain:Terrain) -> Self {

        let mut is_mountain = false;

        let mut x_idx = 0.0 as f32;
        let mut y_idx = 0.0 as f32;
        let x_width = (16.0/32.0) as f32;
        let y_width = (16.0/32.0) as f32;

        match terrain {   // Decide which sprite to use
            Terrain::Coast =>       {(x_idx, y_idx) = (0.0, 0.0)},
            Terrain::Plain =>       {(x_idx, y_idx) = (0.0, 1.0)},
            Terrain::Mountain =>    {(is_mountain, x_idx, y_idx) = (true, 1.0, 0.0)},
            Terrain::Forest =>      {(x_idx, y_idx) = (1.0, 1.0)},
            _ => ()
        }

        Self {
            terrain,
            mountain: is_mountain,
        }

    }

    pub fn is_mountain(&self) -> bool {
        self.mountain
    }

} 