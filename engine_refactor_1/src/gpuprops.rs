#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GPUSprite {
    pub to_region: [f32;4],
    pub from_region: [f32;4]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GPUCamera {
    pub screen_pos: [f32;2],
    pub screen_size: [f32;2],
}