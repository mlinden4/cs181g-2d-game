use std::ops::Deref;

use crate::tile::TerrainModifier;
use crate::tile::Terrain;
use crate::tile::Tile;
use chickenwire::coordinate;
use chickenwire::prelude::HexGrid;
use chickenwire::prelude::MultiCoord;

pub struct Unit {
    name: String,
    hp: usize,
    soft_attack: usize,
    hard_attack: usize,
    aa_damage: usize,
    defense: usize,
    movement: usize,
    terrain_modifiers: [TerrainModifier; 4], // Three terrain modifiers
    location: coordinate::MultiCoord
}

impl Unit {
    fn new(
        name: String,
        hp: usize,
        soft_attack: usize,
        hard_attack: usize,
        aa_damage: usize,
        defense: usize,
        movement: usize,
        location: coordinate::MultiCoord,
    ) -> Self {
        // Initialize terrain modifiers for each terrain type
        let terrain_modifiers = [
            TerrainModifier {
                terrain_type: Terrain::Plain,
                movement: 1,
                defense: 5,
            },
            TerrainModifier {
                terrain_type: Terrain::Mountain,
                movement: 10000,
                defense: 10,
            },
            TerrainModifier {
                terrain_type: Terrain::Coast,
                movement: 10000,
                defense: 2,
            },
            TerrainModifier {
                terrain_type: Terrain::Forest,
                movement: 2,
                defense: 2,
            },
        ];

        Unit {
            name,
            hp,
            soft_attack,
            hard_attack,
            aa_damage,
            defense,
            movement,
            terrain_modifiers,
            location
        }
    }
    pub fn tank(location: coordinate::MultiCoord) -> Self {
        let terrain_modifiers = [
                TerrainModifier {
                    terrain_type: Terrain::Plain,
                    movement: 1,
                    defense: 5,
                },
                TerrainModifier {
                    terrain_type: Terrain::Mountain,
                    movement: 10000,
                    defense: 10,
                },
                TerrainModifier {
                    terrain_type: Terrain::Coast,
                    movement: 10000,
                    defense: 2,
                },
                TerrainModifier {
                    terrain_type: Terrain::Forest,
                    movement: 2,
                    defense: 2,
                },
            ];
        Unit {
            name: "Tank".into(),
            hp: 100,
            soft_attack: 10,
            hard_attack: 15,
            aa_damage: 5,
            defense: 10,
            movement: 2,
            terrain_modifiers, 
            location: location,
        }
    }

    pub fn inrange(&self, destination: coordinate::MultiCoord, tile_grid: &HexGrid<Tile>, unit_grid: &HexGrid<Unit>) -> bool {
        let is_occupied = unit_grid.contains_coord(destination); // if unit grid contains another unit at coordiante
        let is_mountain = tile_grid.get(destination).unwrap().is_mountain(); // if tile is a mountain
        if !is_occupied && !is_mountain{
            let origin_cube =self.location.to_cube().unwrap();
            let dest_cube = destination.to_cube().unwrap();
            let dist = origin_cube.dist(dest_cube);

            dist <= self.movement as i32
        } else {
            false
        }
        // will eventually include dijkstra and terrain cost movement, will do a loop and bfs
        // also check to see if another unit is in tile, return false 
        // is distance to dest and compare with movement
    }

    pub fn move_unit(mut self, destination: coordinate::MultiCoord, tile_grid: &mut HexGrid<Tile>, unit_grid: &mut HexGrid<Unit>) {
        if self.inrange(destination, tile_grid, unit_grid) {
            // let cur_unit = unit_grid.remove(self.location).unwrap();
            self.location = destination;
            let _ = unit_grid.add(destination, self);
        }
    }

}

// vec of neccesary units 
// TODO: location and add more units
pub fn init_units_vec() -> Vec<Unit> {
    let unit_vec: Vec<Unit> = (0..6).map(|_| Unit::tank(MultiCoord::force_cube(0, 0, 0))).collect();
    unit_vec
}
 