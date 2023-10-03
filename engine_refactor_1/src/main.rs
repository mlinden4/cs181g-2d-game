use image;
use wgpu;
use winit;
use bytemuck;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use gpuprops::GPUSprite;
mod spriterenderer;
mod wgpuimpl;
mod input;
mod gpuprops;
mod tile;
mod units;


use chickenwire::coordinate::cube::Cube;
use chickenwire::coordinate::*;
use chickenwire::hexgrid::HexGrid;

enum Shape {
    FilledCircle,
    FilledRectangle,
    OutlinedRectangle
}

fn createChickenWire()  -> HexGrid<i32> {
    // let cube_system = Cube::force_from_coords(0, -3, 3);
    HexGrid::new_radial(3, 9)
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut gpu = wgpuimpl::WGPU::new(&window).await;
    let mut sprites = spriterenderer::SpriteRenderer::new(&gpu);

    let (texture, tex_image) = load_texture("content/king.png", Some("king image"), &gpu.device, &gpu.queue).expect("Couldn't load king img");
    let tex_image_w = tex_image.width();
    let tex_image_h = tex_image.height();



    let mut hex_grid = createChickenWire();
    println!("{}", hex_grid.get(chickenwire::coordinate::MultiChordinate::from((0,0,0)).unwrap()));



    let my_tile = tile::Tile::new(tile::Terrain::Mountain);


    let mut my_sprites:Vec<GPUSprite> = vec![
        // GPUSprite {
        //     to_region: [32.0, 32.0, 64.0, 64.0],
        //     from_region: [0.0, 16.0/32.0, 16.0/32.0, 16.0/32.0],
        // },
        my_tile.get_sprite(),
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

    let camera = gpuprops::GPUCamera {
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

                let my_sprites = sprites.get_sprites_mut(0);


                if input.is_key_down(winit::event::VirtualKeyCode::Key1) {
                    my_sprites[0].to_region[0] -= 4.0;
                }
                if input.is_key_down(winit::event::VirtualKeyCode::Key4) {
                    my_sprites[0].to_region[0] += 4.0;
                }
                if input.is_key_down(winit::event::VirtualKeyCode::Key2) {
                    my_sprites[0].to_region[1] += 4.0;
                }
                if input.is_key_down(winit::event::VirtualKeyCode::Key3) {
                    my_sprites[0].to_region[1] -= 4.0;
                }

                
                
                
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
