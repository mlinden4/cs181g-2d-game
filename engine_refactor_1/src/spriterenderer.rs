use std::ops::Range;
use std::borrow::Cow;

use crate::wgpuimpl::WGPU;
use crate::gpuprops::GPUCamera;
use crate::gpuprops::GPUSprite;

pub struct SpriteGroup {
    texture: wgpu::Texture,
    buffer: wgpu::Buffer,
    sprites: Vec<GPUSprite>,
    texture_bind_group: wgpu::BindGroup,
    sprite_bind_group: wgpu::BindGroup,
}

pub struct SpriteRenderer {
    pipeline:wgpu::RenderPipeline,
    sprite_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    buffer_camera: wgpu::Buffer,
    groups:Vec<SpriteGroup>
}

impl SpriteRenderer {

    pub fn new(gpu:&WGPU) -> Self {

        // Load the shaders from disk
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let texture_bind_group_layout =
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                // This bind group's first entry is for the texture and the second is for the sampler.
                entries: &[
                    // The texture binding
                    wgpu::BindGroupLayoutEntry {
                        // This matches the binding number in the shader
                        binding: 0,
                        // Only available in the fragment shader
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // It's a texture binding
                        ty: wgpu::BindingType::Texture {
                            // We can use it with float samplers
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            // It's being used as a 2D texture
                            view_dimension: wgpu::TextureViewDimension::D2,
                            // This is not a multisampled texture
                            multisampled: false,
                        },
                        // This is not an array texture, so it has None for count
                        count: None,
                    },
                    // The sampler binding
                    wgpu::BindGroupLayoutEntry {
                        // This matches the binding number in the shader
                        binding: 1,
                        // Only available in the fragment shader
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // It's a sampler
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        // No count
                        count: None,
                    },
                ],
        });

        let sprite_bind_group_layout =
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // The camera binding
                wgpu::BindGroupLayoutEntry {
                    // This matches the binding in the shader
                    binding: 0, //Was 0
                    // Available in vertex shader
                    visibility: wgpu::ShaderStages::VERTEX,
                    // It's a buffer
                    ty: wgpu::BindingType::Buffer {
                        // Specifically, a uniform buffer
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    // No count, not a buffer array binding
                    count: None,
                },
                // The sprite buffer binding
                wgpu::BindGroupLayoutEntry {
                    // This matches the binding in the shader
                    binding: 1,
                    // Available in vertex shader
                    visibility: wgpu::ShaderStages::VERTEX,
                    // It's a buffer
                    ty: wgpu::BindingType::Buffer {
                        // Specifically, a storage buffer
                        ty: wgpu::BufferBindingType::Storage{read_only:true},
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    // No count, not a buffer array binding
                    count: None,
                },
            ],
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&sprite_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(gpu.config.format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let buffer_camera = gpu.device.create_buffer(&wgpu::BufferDescriptor{
            label: None,
            size: std::mem::size_of::<GPUCamera>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        Self {
            pipeline,
            sprite_bind_group_layout,
            texture_bind_group_layout,
            buffer_camera,
            groups: Vec::default(),
        }
    }

    pub fn add_sprite_group(&mut self, gpu:&WGPU, texture:wgpu::Texture, sprites:Vec<GPUSprite>) -> usize {

        let tex_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let tex_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor::default());  

        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor{
            label: None,
            size: bytemuck::cast_slice::<_,u8>(&sprites).len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let sprite_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.sprite_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.buffer_camera.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding()
                }
            ],
        });
            

        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
            entries: &[
                // One for the texture, one for the sampler
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex_sampler),
                },
            ],
        });
            
        


        gpu.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&sprites));
        
        self.groups.push(SpriteGroup{
            texture,
            buffer,
            sprites,
            texture_bind_group,
            sprite_bind_group,
        });

        self.groups.len() -1
    }

    pub fn clear_sprite_groups(&mut self){
        self.groups = Vec::default();
    }

    pub fn set_sprite_group(&mut self, which:usize, new_sprites:Vec<GPUSprite>) {
        self.groups[which].sprites = new_sprites;
    }

    pub fn set_camera(&mut self, gpu:&WGPU, camera:&GPUCamera) {
        gpu.queue.write_buffer(&self.buffer_camera, 0, bytemuck::bytes_of(camera));
    }

    // Refresh a slice of sprites
    pub fn refresh_sprites(&mut self, gpu:&WGPU, which:usize, range:Range<usize>) {
        gpu.queue.write_buffer(&self.groups[which].buffer, range.start as u64, bytemuck::cast_slice(&self.groups[which].sprites[range]));
    }

    // Get a slice of sprites
    pub fn get_sprites_mut(&mut self, which:usize) -> &mut [GPUSprite] {
        &mut self.groups[which].sprites
    }

    pub fn get_sprites(&mut self, which:usize) -> &[GPUSprite] {
        &self.groups[which].sprites
    }

    pub fn render<'s, 'pass>(&'s self, rpass:&mut wgpu::RenderPass<'pass>) 
        where 's: 'pass,
    {
        rpass.set_pipeline(&self.pipeline);

        for group in self.groups.iter() {

            rpass.set_bind_group(0, &group.sprite_bind_group, &[]);
            rpass.set_bind_group(1, &group.texture_bind_group, &[]);
            // // draw two triangles per sprite, and sprites-many sprites.
            // // this uses instanced drawing, but it would also be okay
            // // to draw 6 * sprites.len() vertices and use modular arithmetic
            // // to figure out which sprite we're drawing, instead of the instance index.
            rpass.draw(0..6, 0..(group.sprites.len() as u32));

        }

        
    }

}