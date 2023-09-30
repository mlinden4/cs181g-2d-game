use std::borrow::Cow;
use image;
use wgpu;
use winit;
use bytemuck;
use std::ops::Range;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

enum Shape {
    FilledCircle,
    FilledRectangle,
    OutlinedRectangle
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct GPUSprite {
    to_region: [f32;4],
    from_region: [f32;4]
}


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct GPUCamera {
    screen_pos: [f32;2],
    screen_size: [f32;2],
}



mod input;


#[allow(dead_code)]
struct WGPU {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl WGPU {
    async fn new(window:&Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Bump up the limits to require the availability of storage buffers.
                    limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];


        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
    
        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
        }
    }

    fn resize(&mut self, size:winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    // fn render(&self, rend: impl FnMut(&mut wgpu::RenderPass, &mut wgpu::CommandEncoder)) {
    //     // ... All the 3d drawing code/render pipeline/queue/frame stuff goes here ...
    //     let frame = self.surface
    //         .get_current_texture()
    //         .expect("Failed to acquire next swap chain texture");
    //     let view = frame
    //         .texture
    //         .create_view(&wgpu::TextureViewDescriptor::default());
    //     let mut encoder =
    //         self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    //     {
    //         let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //             label: None,
    //             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //                 view: &view,
    //                 resolve_target: None,
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
    //                     store: true,
    //                 },
    //             })],
    //             depth_stencil_attachment: None,
    //         });
    //     }
        
    // }
}   


struct SpriteGroup {
    texture: wgpu::Texture,
    buffer: wgpu::Buffer,
    sprites: Vec<GPUSprite>,
    texture_bind_group: wgpu::BindGroup,
    sprite_bind_group: wgpu::BindGroup,
}

struct SpriteRenderer {
    pipeline:wgpu::RenderPipeline,
    sprite_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    buffer_camera: wgpu::Buffer,
    groups:Vec<SpriteGroup>
}

impl SpriteRenderer {

    fn new(gpu:&WGPU) -> Self {

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

    fn add_sprite_group(&mut self, gpu:&WGPU, texture:wgpu::Texture, sprites:Vec<GPUSprite>) -> usize {

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

    fn set_camera(&mut self, gpu:&WGPU, camera:&GPUCamera) {
        gpu.queue.write_buffer(&self.buffer_camera, 0, bytemuck::bytes_of(camera));
    }

    // Refresh a slice of sprites
    fn refresh_sprites(&mut self, gpu:&WGPU, which:usize, range:Range<usize>) {
        gpu.queue.write_buffer(&self.groups[which].buffer, range.start as u64, bytemuck::cast_slice(&self.groups[which].sprites[range]));
    }

    // Get a slice of sprites
    fn get_sprites_mut(&mut self, which:usize) -> &mut [GPUSprite] {
        &mut self.groups[which].sprites
    }

    fn get_sprites(&mut self, which:usize) -> &[GPUSprite] {
        &self.groups[which].sprites
    }

    fn render<'s, 'pass>(&'s self, rpass:&mut wgpu::RenderPass<'pass>) 
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

async fn run(event_loop: EventLoop<()>, window: Window) {
    
    let mut gpu = WGPU::new(&window).await;
    let mut sprites = SpriteRenderer::new(&gpu);

    let (texture, tex_image) = load_texture("content/king.png", Some("king image"), &gpu.device, &gpu.queue).expect("Couldn't load king img");
    let tex_image_w = tex_image.width();
    let tex_image_h = tex_image.height();

    let mut my_sprites:Vec<GPUSprite> = vec![
        GPUSprite {
            to_region: [32.0, 32.0, 64.0, 64.0],
            from_region: [0.0, 16.0/32.0, 16.0/32.0, 16.0/32.0],
        },
        GPUSprite {
            to_region: [32.0, 128.0, 64.0, 64.0],
            from_region: [16.0/32.0, 16.0/32.0, 16.0/32.0, 16.0/32.0],
        },
        GPUSprite {
            to_region: [128.0, 32.0, 64.0, 64.0],
            from_region: [0.0, 16.0/32.0, 16.0/32.0, 16.0/32.0],
        },
        GPUSprite {
            to_region: [128.0, 128.0, 64.0, 64.0],
            from_region: [16.0/32.0, 16.0/32.0, 16.0/32.0, 16.0/32.0],
        },
    ];

    sprites.add_sprite_group(&gpu, texture, my_sprites);

    

    let camera = GPUCamera {
        screen_pos: [0.0, 0.0],
        // Consider using config.width and config.height instead,
        // it's up to you whether you want the window size to change what's visible in the game
        // or scale it up and down
        screen_size: [1024.0, 768.0],
    };
    



    // gpu.queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));



    let mut input = input::Input::default();



    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        // let _ = (&gpu.instance, &gpu.adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                gpu.resize(size);
                // gpu.surface.configure(&gpu.device, &gpu.config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {

                // if input.is_key_down(winit::event::VirtualKeyCode::Key1) {
                //     my_sprites[0].to_region[0] -= 4.0;
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::Key4) {
                //     my_sprites[0].to_region[0] += 4.0;
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::Key2) {
                //     my_sprites[0].to_region[1] += 4.0;
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::Key3) {
                //     my_sprites[0].to_region[1] -= 4.0;
                // }

                
                
                
                input.next_frame();
                sprites.set_camera(&gpu, &camera);
                let length = sprites.get_sprites(0).len();
                sprites.refresh_sprites(&gpu, 0, 0..length);
 

                // ... All the 3d drawing code/render pipeline/queue/frame stuff goes here ...
                let frame = gpu.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    // gpu.render(|rpass, _encoder| {
                    //     rpass.set_pipeline(render_pipeline);
                    //     rpass.set_bind_group(0, &sprite_bind_group, &[]);
                    //     rpass.set_bind_group(1, &texture_bind_group, &[]);
                    //     // draw two triangles per sprite, and sprites-many sprites.
                    //     // this uses instanced drawing, but it would also be okay
                    //     // to draw 6 * sprites.len() vertices and use modular arithmetic
                    //     // to figure out which sprite we're drawing, instead of the instance index.
                    //     rpass.draw(0..6, 0..(sprites.len() as u32));
                    // });
                    sprites.render(&mut rpass);
                }
                
                

                gpu.queue.submit(Some(encoder.finish()));
                frame.present();

                // (3)
                // And we have to tell the window to redraw!
                window.request_redraw();

                // Leave now_keys alone, but copy over all changed keys
                input.next_frame();
            },
            // WindowEvent->KeyboardInput: Keyboard input!
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event: WindowEvent::KeyboardInput {
                    input:key_ev,
                    ..
                },
                ..
            } => {
                input.handle_key_event(key_ev);
            },
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                input.handle_mouse_button(state, button);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                input.handle_mouse_move(position);
            }
            _ => (),
            // Event::WindowEvent {
            //     event: WindowEvent::CloseRequested,
            //     ..
            // } => *control_flow = ControlFlow::Exit,
            // _ => {}
        }
    });
}



// AsRef means we can take as parameters anything that cheaply converts into a Path,
// for example an &str.
fn load_texture(path:impl AsRef<std::path::Path>, label:Option<&str>,
                device:&wgpu::Device, queue:&wgpu::Queue
    ) -> Result<(wgpu::Texture, image::RgbaImage), image::ImageError> {
    // This ? operator will return the error if there is one, unwrapping the result otherwise.
    let img = image::open(path.as_ref())?.to_rgba8();

    let (width, height) = img.dimensions();

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    Ok((texture,img))
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(imageproc::drawing::Blend(window.canvas())))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
