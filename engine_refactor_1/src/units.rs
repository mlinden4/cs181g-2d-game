use std::ops::Deref;

use crate::tile::TerrainModifier;
use crate::tile::Terrain;
use crate::tile::Tile;
use chickenwire::coordinate;
use chickenwire::prelude::HexGrid;

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

    pub fn inrange(&self, destination: coordinate::MultiCoord, grid: &HexGrid<Tile>) -> bool {
        let is_occupied = grid.get(destination).unwrap().is_occupied();
        if !is_occupied {
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

    pub fn move_unit(&mut self, destination: coordinate::MultiCoord, grid: &mut HexGrid<Tile>) {
        if self.inrange(destination, grid) {
            grid.get_mut(self.location).unwrap().set_empty(); // set original tile as empty
            grid.get_mut(destination).unwrap().set_occupied(); // set new tile to occupied
            self.location = destination
        }
    }

}
 