use crate::units::{Unit, init_units_vec};


pub struct Player{
    name: String,
    units: Vec<Unit>
}

impl Player{
    pub fn new(&self, name:String) -> Self {
        Player {
            name: name.into(),
            units: init_units_vec(),
        }
    }
}

