use image;
use tile::Tile;
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
mod textrenderer;
mod wgpuimpl;
mod input;
mod gpuprops;
mod tile;
mod units;
use bytemuck::Zeroable;
use std::time::Instant;

use chickenwire::{coordinate::cube::Cube, prelude::MultiCoord};
use chickenwire::hexgrid::HexGrid;
use chickenwire::coordinate;

use glyphon::Color;
use glyphon::TextArea;
use glyphon::TextBounds;
use glyphon::Resolution;
use glyphon::Attrs;
use glyphon::Family;
use glyphon::Shaping;
use glyphon::Metrics;
use wgpu::MultisampleState;
use glyphon::Buffer;
use glyphon::TextRenderer;
use glyphon::TextAtlas;
use glyphon::SwashCache;
use glyphon::FontSystem;


mod gamemap;
mod statehandler;

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut gpu = wgpuimpl::WGPU::new(&window).await;
    let mut sprites = spriterenderer::SpriteRenderer::new(&gpu);
    let mut text_renders = textrenderer::TextRenderList::new();

    
    let (texture0, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let (texture1, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let (texture2, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    
    let mut input = input::Input::default();
    // let mut hexgrid = gamemap::create_hexgrid();

    let mut camera = gpuprops::GPUCamera {
        screen_pos: [0.0, 0.0],
        screen_size: [gpu.config.width as f32, gpu.config.height as f32],
    };

    // TIMING
    let mut acc = 0.0_f32;
    let mut prev_t = Instant::now();
    const SIM_DT : f32 = 1.0/60.0; // 60 simulation steps per second




    let mut game_state = statehandler::GameState {
        game_mode: statehandler::GameMode::MainMenu(true),
        hexgrid: gamemap::create_hexgrid(),
        player1_units: Vec::new(),
        player2_units: Vec::new(),
        global_tile: Tile::new(tile::Terrain::Plain),
        moving_unit_location: None::<coordinate::MultiCoord>,
    };


    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        // let _ = (&gpu.instance, &gpu.adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                gpu.resize(size);
                // camera.screen_size = size.into();
                // sprites.set_camera(&gpu, &camera);
                // gpu.surface.configure(&gpu.device, &gpu.config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
                // figure out how to rerender the hexgrid to sprites as the camera changes
            },
            Event::MainEventsCleared => {

                // Handle timing
                let elapsed = prev_t.elapsed().as_secs_f32();
                acc += elapsed;
                prev_t = Instant::now();
                while acc >= SIM_DT {
                    
                    // Handle Updating Game
                    match game_state.game_mode {
                        statehandler::GameMode::MainMenu(needs_initialization) => {
                            if needs_initialization { 
                                let (texture_sheet, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                statehandler::initalizeMainMenu(&gpu, &window, &mut text_renders, &mut camera, texture_sheet, &mut sprites, &mut game_state);
                            }
                            statehandler::updateMainMenu(&gpu, &mut input, &mut camera, &mut text_renders, &mut sprites, &mut game_state);
                            // Handle main menu
                        }
                        statehandler::GameMode::MapCreator(needs_initialization) => {
                            if needs_initialization { 
                                // println!("Initializing");
                                let (texture_sheet, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                statehandler::initalizeMapCreator(&gpu, &mut camera, texture_sheet, &mut sprites, &mut game_state);
                            }
                            println!("Updating Map");
                            statehandler::updateMapCreator(&gpu, &mut input, &mut camera, &mut sprites, &mut game_state);
                        }
                        statehandler::GameMode::WarGame(needs_initialization, _) => {
                            if needs_initialization { 
                                let (texture_sheet0, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                let (texture_sheet1, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                let (texture_sheet2, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                // let sprite_steets = vec![texture_sheet0, texture_sheet1, texture_sheet2];
                                statehandler::initalizeWarGame(&gpu, &mut camera, texture_sheet0, texture_sheet1, texture_sheet2, &mut sprites, &mut game_state);
                            }
                            println!("Updating War");
                            statehandler::updateWarGame(&gpu, &mut input, &mut camera, &mut sprites, &mut game_state);
                        }
                        statehandler::GameMode::GameOver(needs_initialization, winner) => {
                            if needs_initialization { 
                                /*initialize*/ 
                                game_state.game_mode = statehandler::GameMode::GameOver(false, 0) 
                            }
                            // Handle main menu
                        }
                    }

                    // NOTE: This is when you should swap "new" keys and "old" keys for input handling!
                    // Otherwise you'll see several frames in a row where a key was just pressed/released.
                    
                    input.next_frame();
                    acc -= SIM_DT;
                }
                
                
                let size = window.inner_size();

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

                    sprites.render(&mut rpass);
                    text_renders.render(&mut rpass);
                    
                }
                
                gpu.queue.submit(Some(encoder.finish()));
                frame.present();
                window.request_redraw();
                text_renders.trim_atlas(); // For text display
                

                // Leave now_keys alone, but copy over all changed keys
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
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
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
    // create_chicken_wire();
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
