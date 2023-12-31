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
use rand::Rng;


pub struct Itinerary {
    coordinate: coordinate::Cube,
    distance: i32
}

#[derive(PartialEq, Clone)]
pub struct Unit {
    pub name: String,
    pub max_hp: i32,
    pub hp: i32,
    soft_attack: i32,
    hard_attack: i32,
    aa_damage: i32,
    defense: i32,
    pub movement: i32,
    pub remaining_movement: i32,
    terrain_modifiers: [TerrainModifier; 4], // Three terrain modifiers
    pub location: coordinate::MultiCoord,
    pub has_fought: bool
}

impl Unit {
    fn new(
        name: String,
        max_hp: i32,
        hp: i32,
        soft_attack: i32,
        hard_attack: i32,
        aa_damage: i32,
        defense: i32,
        movement: i32,
        location: coordinate::MultiCoord,
        has_fought: bool,
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
            max_hp,
            hp,
            soft_attack,
            hard_attack,
            aa_damage,
            defense,
            movement,
            remaining_movement,
            terrain_modifiers,
            location,
            has_fought,
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
            max_hp: 100,
            hp: 100,
            soft_attack: 15,
            hard_attack: 15,
            aa_damage: 5,
            defense: 10,
            movement: 3,
            remaining_movement: 3,
            terrain_modifiers, 
            location: location,
            has_fought: false
        }
    }
    pub fn helicopter(location: coordinate::MultiCoord) -> Self {
        let terrain_modifiers = [
                TerrainModifier {
                    terrain_type: Terrain::Plain,
                    movement: 1,
                    defense: 5,
                },
                TerrainModifier {
                    terrain_type: Terrain::Mountain,
                    movement: 2,
                    defense: 1,
                },
                TerrainModifier {
                    terrain_type: Terrain::Coast,
                    movement: 1,
                    defense: 5,
                },
                TerrainModifier {
                    terrain_type: Terrain::Forest,
                    movement: 1,
                    defense: 3,
                },
            ];

        Unit {
            name: "Helicopter".into(),
            max_hp: 75,
            hp: 75,
            soft_attack: 15,
            hard_attack: 25,
            aa_damage: 10,
            defense: 10,
            movement: 5,
            remaining_movement: 5,
            terrain_modifiers, 
            location: location,
            has_fought: false
        }
    }
    pub fn infantry(location: coordinate::MultiCoord) -> Self {
        let terrain_modifiers = [
                TerrainModifier {
                    terrain_type: Terrain::Plain,
                    movement: 1,
                    defense: 5,
                },
                TerrainModifier {
                    terrain_type: Terrain::Mountain,
                    movement: 2,
                    defense: 20,
                },
                TerrainModifier {
                    terrain_type: Terrain::Coast,
                    movement: 10000,
                    defense: 2,
                },
                TerrainModifier {
                    terrain_type: Terrain::Forest,
                    movement: 2,
                    defense: 10,
                },
            ];

        Unit {
            name: "Infantry".into(),
            max_hp: 120,
            hp: 120,
            soft_attack: 10,
            hard_attack: 5,
            aa_damage: 20,
            defense: 10,
            movement: 2,
            remaining_movement: 2,
            terrain_modifiers, 
            location: location,
            has_fought: false
        }
    }

    // if true u die
    pub fn move_unit(&mut self, destination: coordinate::MultiCoord, enemy_units : &mut Vec<Unit>, allied_units : Vec<Unit>, grid: &HexGrid<Tile>) -> bool {
        let itin = self.bfs(grid, enemy_units.clone(), allied_units.clone());
        

        // if reachable
        if let Some(result) = itin.iter().find(|itinerary| itinerary.coordinate == destination.to_cube().unwrap()) {
            let match_itin = result;
            // print!("WITHIN ITIN WITHIN ITIN WITHIN ITIN WITHIN ITINWITHIN ITIN WITHIN ITIN WITHIN ITIN WITHIN ITIN\n");
            // if let Some(mut enemy) = enemy_units.iter().find(|enemy: &&Unit| enemy.location == destination) {
            //     print!("ENEMY ENEMY ENEMY ENEMY ENEMY ENEMY\n");
            //     // FIGHT, if returns true, move unit to location
            //     self.fight(&mut enemy);
            // }

            if self.remaining_movement >= match_itin.distance as i32{
                 
                self.location = destination;
                
                self.remaining_movement -= match_itin.distance as i32;
                // print!("\n DIST DIST DIST DIST DIST DIST DIST DIST \n");
            }
            // print!("\n FUCKED FUCKED FUCKED FUCKED FUCKED FUCKED \n {}", match_itin.distance);
            // now check if enemy at location
            if let Some(index) = enemy_units.iter().position(|enemy| enemy.location == destination) {
                // println!("ENEMY ENEMY ENEMY ENEMY ENEMY ENEMY");
            
                // Get a reference to the matching enemy
                let mut enemy = &mut enemy_units[index];
                if !self.has_fought {
                    let (youdie, theydie) = self.fight(enemy);

                    // Remove the matching enemy from the vector
                    if youdie {
                        // print!("you died\n");
                        if theydie { //how????
                            // print!("you died and they died\n");
                            self.hp = 1;
                            self.location = destination;
                            enemy_units.remove(index);
                            return false;
                        }
                        return true;
                    } else { // if you dont die
                        if theydie {
                            // print!("they died\n");
                            enemy_units.remove(index);
                            self.location = destination;
                        }
                        print!("you won\n");
                        print!("your health {}\n", self.hp);
                        return false;
                    }
                }
                

                // if let Some(enemy) = enemy_units.iter_mut().find(|enemy| enemy.location == destination) {
                //     println!("ENEMY ENEMY ENEMY ENEMY ENEMY ENEMY");
                
                //     // Now you have a mutable reference to the matching enemy, and you can modify it.
                //     (result, death) =self.fight(enemy);
                //     if result{
                //         enemy_units.remove(index)
                //     }
            }
        }

        return false;
    }

    pub fn bfs(&self, grid: &HexGrid<Tile>, enemy_units : Vec<Unit>, allied_units : Vec<Unit>) -> Vec<Itinerary> {
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
                    if let Some(tile) = grid.get(MultiCoord::from(neighbor)) {
                        if (self.name != "Helicopter") && (tile.terrain == Terrain::Coast) { 
                            dist = 999;
                        }
                        if tile.terrain == Terrain::Plain { 
                            dist = 1;
                        }
                        if tile.terrain == Terrain::Mountain { 
                            dist = 999;
                        }
                        if tile.terrain == Terrain::Forest { 
                            dist = 1;
                        }
                        
                        //Special movement costs
                        if (self.name == "Infantry") && (tile.terrain == Terrain::Mountain) { 
                            dist = 2;
                        }
                        if (self.name == "Tank") && (tile.terrain == Terrain::Forest) { 
                            dist = 2;
                        }
                        // Make if statement if there is opponents unit
                        for unit in &enemy_units { 
                            if unit.location == MultiCoord::from(neighbor) {
                                dist = 999;
                            }
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
    

    // first bool is if you die, second bool is if enemy dies
    pub fn fight(&mut self, enemy : &mut Unit) -> (bool, bool) {

        // Can no longer move after fighting
        self.remaining_movement = 0;
        self.has_fought = true;
 
        // attacker info
        let mut attack: i32 = self.soft_attack;
        if enemy.name == "Helicopter" {
            attack = self.aa_damage;
        }
        if enemy.name == "Tank" {
            attack = self.hard_attack
        }
        let mut defense = enemy.defense;

        // defender counter info
        let mut eattack: i32 = enemy.soft_attack;
        if self.name == "Helicopter" {
            eattack = enemy.aa_damage;
        }
        if self.name == "Tank" {
            eattack = enemy.hard_attack
        }
        let mut edefense = enemy.defense;

        // Attack damage
        let mut rng = rand::thread_rng();
        // let attack_damage_modifer: i32 = (rng.gen_range(0..100))/100;

        let mut attack_power = attack;
        // if(attack_power < 0) {
        //     attack_power = 5;
        // }
        let damage: i32 = 15 + attack_power as i32;

        // Defense damage
        // let defense_damage_modifer: i32 = (rng.gen_range(0..100))/100;

        let mut defense_power = eattack;
        // if(defense_power < 0) {
        //     defense_power = 0;
        // }
        let oof: i32 = defense_power as i32;

        enemy.hp -= damage as i32;
        self.hp -= oof as i32;

        // println!("def {}, def_mod", defense_power);
        // println!("dmg: {}, oof {}", damage, oof);

        return (self.hp <= 0, enemy.hp <= 0)
    }

}
 