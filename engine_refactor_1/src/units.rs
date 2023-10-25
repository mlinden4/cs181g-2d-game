use std::ops::Deref;

use crate::tile::TerrainModifier;
use crate::tile::Terrain;
use crate::tile::Tile;
use chickenwire::prelude::MultiCoord;
use chickenwire::coordinate;
use chickenwire::prelude::HexGrid;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::collections::HashMap;


pub struct Itinerary {
    coordinate: coordinate::Cube,
    distance: i32
}

#[derive(PartialEq)]
pub struct Unit {
    pub name: String,
    hp: usize,
    soft_attack: usize,
    hard_attack: usize,
    aa_damage: usize,
    defense: usize,
    pub movement: usize,
    pub remaining_movement: usize,
    terrain_modifiers: [TerrainModifier; 4], // Three terrain modifiers
    pub location: coordinate::MultiCoord
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
        let remaining_movement = movement;

        Unit {
            name,
            hp,
            soft_attack,
            hard_attack,
            aa_damage,
            defense,
            movement,
            remaining_movement,
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
            remaining_movement: 2,
            terrain_modifiers, 
            location: location,
        }
    }

    pub fn inrange(&self, destination: coordinate::MultiCoord, grid: &HexGrid<Tile>) -> bool {
        // do not need to check for occupied, already done in statehandler
        let origin_cube =self.location.to_cube().unwrap();
        let dest_cube = destination.to_cube().unwrap();
        let dist = origin_cube.dist(dest_cube);

        print!("{}", dist);

        if dist <= self.movement as i32 {
            if self.name != "Helicopter" {
                return true;
            } else {
                return !grid.get(destination).unwrap().is_mountain(); // look at 
            }
        }

        false

        // let is_occupied = grid.get(destination).unwrap().is_occupied();
        // if !is_occupied {
        //     let origin_cube =self.location.to_cube().unwrap();
        //     let dest_cube = destination.to_cube().unwrap();
        //     let dist = origin_cube.dist(dest_cube);
        //     dist <= self.movement as i32
        // } else {
        //     false
        // }
        // will eventually include dijkstra and terrain cost movement, will do a loop and bfs
        // also check to see if another unit is in tile, return false 
        // is distance to dest and compare with movement
    }

    pub fn bfs(&self, grid: &HexGrid<Tile>, enemy_units : Vec<Unit>, allied_units : Vec<Unit>) -> Vec<(Itinerary)> {
        let origin_cube = self.location.to_cube().unwrap();
        
        let mut visited = HashSet::new(); // To keep track of visited coordinates
        let mut queue = VecDeque::new(); // Queue for BFS traversal
        let mut distances = HashMap::new();
    
        queue.push_back(origin_cube);
        visited.insert(origin_cube);
        distances.insert(origin_cube, 0);
    
        let mut result = Vec::new();
    
        while !queue.is_empty() {
            // Dequeue the current coordinate from the front of the queue
            let current: chickenwire::prelude::Cube = queue.pop_front().unwrap();
            let current_distance: i32 = distances[&current];
    
            // Process neighbors
            for neighbor in current.neighbors() {
                if !visited.contains(&neighbor) {
                    // You can add conditions here to filter valid neighbors based on movement costs
                    let mut dist: i32 = 1; 
                    // Make if statements for each tile's movement cost
                    let tile = grid.get(MultiCoord::from(neighbor)).unwrap();
                    
                    // TODO, ONLY CALL THIS LINE IF ITS NOT A HELICOPTER
                    if tile.terrain == Terrain::Coast { 
                        dist = 999;
                    }
                    if tile.terrain == Terrain::Plain { 
                        dist = 1;
                    }
                    if tile.terrain == Terrain::Mountain { 
                        dist = 999;
                    }
                    // TODO: SPECIAL ONE FOR TANKS SO FORESTS ARE 2 COST FOR THEM
                    if tile.terrain == Terrain::Forest { 
                        dist = 1;
                    }
                    // Make if statement if there is opponents unit
                    for unit in &enemy_units { 
                        if unit.location == MultiCoord::from(neighbor) {
                            dist = 999;
                        }
                    }
                    
                    let neighbor_distance: i32 = current_distance + dist;

                    if neighbor_distance <= self.remaining_movement.try_into().unwrap() {
                        // Add neighbor to the queue and mark it as visited
                        queue.push_back(neighbor);
                        visited.insert(neighbor);
    
                        // Update the distance to the neighbor in the distances map
                        distances.insert(neighbor, neighbor_distance);
    
                        // Create an Itinerary struct and add it to the result
                        result.push(Itinerary {
                            coordinate: neighbor,
                            distance: neighbor_distance,
                        });
                    }
    
                  
    
                    // Just add the neighbor to the result with distance of dist
                    let itin: Itinerary = Itinerary{coordinate : neighbor, distance : dist};
                    result.push(itin);
                }
            }
        }

        // Update distances to 999 for allied units
        for unit in &allied_units {
            for itinerary in &mut result {
                if itinerary.coordinate == unit.location.to_cube().unwrap() {
                    itinerary.distance = 999;
                }
            }
        }
    
        result
    }
    

    pub fn move_unit(&mut self, destination: coordinate::MultiCoord, grid: &mut HexGrid<Tile>) {
        if self.inrange(destination, grid) {
            // grid.get_mut(self.location).unwrap().set_empty(); // set original tile as empty
            // grid.get_mut(destination).unwrap().set_occupied(); // set new tile to occupied
            self.location = destination
        }
    }

}
 