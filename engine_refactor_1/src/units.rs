pub struct Units {
    hp: usize,
    soft_attack: usize, // attacks/damage are multipliers against defense 
    hard_attack: usize,
    aa_damage: usize,
    // armor: usize, ///resistance to hard attack
    defense: usize,
    movement: usize,
    location: (usize, usize, usize),
    
} 

impl Units {
    // fn take_damage() {
    // }


    fn tank() -> Self {
        Units {
            hp: 100,
            soft_attack: 10,
            hard_attack: 15,
            aa_damage: 5,
            defense: 10,
            movement: 2,
            location: (0,0,0),
        }
    }
}