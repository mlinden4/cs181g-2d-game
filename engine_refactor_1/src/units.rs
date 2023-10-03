pub struct Unit {
    hp: usize,
    soft_attack: usize,
    hard_attack: usize,
    aa_damage: usize,
    defense: usize,
    movement: usize,
    terrain_modifiers: [TerrainModifier; 3], // Three terrain modifiers
}

impl Unit {
    fn new(
        hp: usize,
        soft_attack: usize,
        hard_attack: usize,
        aa_damage: usize,
        defense: usize,
        movement: usize,
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

        Units {
            hp,
            soft_attack,
            hard_attack,
            aa_damage,
            defense,
            movement,
            terrain_modifiers,
        }
    }
}
fn tank() -> Self {
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
    Units {
        hp: 100,
        soft_attack: 10,
        hard_attack: 15,
        aa_damage: 5,
        defense: 10,
        movement: 2,
        terrain_modifiers, 
    }
}
}
