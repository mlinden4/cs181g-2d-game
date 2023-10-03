#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GPUSprite {
    pub to_region: [f32;4],
    pub from_region: [f32;4]
}


impl GPUSprite {
    pub fn contains(&self, x:f32, y:f32) -> bool {

        let x1 = self.to_region[0];
        let x2 = self.to_region[0] + self.to_region[2];
        let y1 = self.to_region[1];
        let y2 = self.to_region[1] + self.to_region[3];

        println!("{}, {}, {}, {}", x1, x2, y1, y2);

        x1 < x && x < x2 && y1 < y && y < y2
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GPUCamera {
    pub screen_pos: [f32;2],
    pub screen_size: [f32;2],
}