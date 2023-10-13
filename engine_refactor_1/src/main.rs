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
mod wgpuimpl;
mod input;
mod gpuprops;
mod tile;
mod units;
use bytemuck::Zeroable;

use chickenwire::{coordinate::cube::Cube, prelude::MultiCoord};
use chickenwire::hexgrid::HexGrid;
use chickenwire::coordinate;

enum Shape {
    FilledCircle,
    OutlinedRectangle
}

const hexgrid_radius:i32 = 10;
const hex_size:f32 = 64.0;
    
const from_x:f32 = 1.0/7.0;
const from_y:f32 = 0.0;
const from_width:f32 = 1.0/7.0; //448 x 64
const from_height:f32 = 1.0;

// move this out eventually
fn create_terrain_hexgrid() -> HexGrid<tile::Tile> {

    let default_tile = tile::Tile::new(tile::Terrain::Coast);

    let mut terrain_hexgrid: HexGrid<tile::Tile> = HexGrid::new_radial(hexgrid_radius as u32, default_tile);

    terrain_hexgrid

}

fn create_unit_hexgrid() -> HexGrid<units::Unit> {
    let unit_hexgrid: HexGrid<units::Unit> = HexGrid::new(chickenwire::hexgrid::Tilt::Flat, chickenwire::hexgrid::Parity::Even, chickenwire::prelude::CoordSys::Cube);

    unit_hexgrid
}

fn terrain_hexgrid_to_sprites(camera:&gpuprops::GPUCamera, hexgrid:&HexGrid<tile::Tile>, sprites: &mut[GPUSprite]) {

    let mut sprite_num = 0;
    // let mut output_sprites:Vec<GPUSprite> = vec![];

    for q in -hexgrid_radius..=hexgrid_radius {
        for r in -hexgrid_radius..=hexgrid_radius {
            for s in -hexgrid_radius..=hexgrid_radius {
                if q + r + s == 0 {

                    let hex = hexgrid.get(coordinate::MultiCoord::force_cube(q, r, s)).unwrap();

                    let mut sprite_idx = 0.0;
                    match hex.terrain {
                        tile::Terrain::Coast => { sprite_idx = 3.0 }
                        tile::Terrain::Plain => { sprite_idx = 4.0 }
                        tile::Terrain::Mountain => { sprite_idx = 0.0 }
                        tile::Terrain::Forest => { sprite_idx = 2.0 }
                        // _ => ();
                    }

                    let (world_x_pos, world_y_pos) = hex_to_xy(camera, hex_size, q as f32,r as f32,s as f32);

                    sprites[sprite_num] = GPUSprite {
                        to_region: [world_x_pos, world_y_pos, hex_size, hex_size],
                        from_region: [sprite_idx*from_x, from_y, from_width, from_height],
                    };

                    sprite_num = sprite_num + 1;

                }
            }
        }
    }
}

// fn unit_hexgrid_to_sprites(camera:&gpuprops::GPUCamera, hexgrid:&HexGrid<tile::Tile>, sprites: &mut[GPUSprite]) {

//     let mut sprite_num = 0;

//     for q in -hexgrid_radius..=hexgrid_radius {
//         for r in -hexgrid_radius..=hexgrid_radius {
//             for s in -hexgrid_radius..=hexgrid_radius {
//                 if q + r + s == 0 {

//                     let hex = hexgrid.get(coordinate::MultiCoord::force_cube(q, r, s)).unwrap();

//                     let mut sprite_idx = 0.0;
//                     match hex.terrain {
//                         tile::Terrain::Coast => { sprite_idx = 3.0 }
//                         tile::Terrain::Plain => { sprite_idx = 4.0 }
//                         tile::Terrain::Mountain => { sprite_idx = 0.0 }
//                         tile::Terrain::Forest => { sprite_idx = 2.0 }
//                         // _ => ();
//                     }

//                     let (world_x_pos, world_y_pos) = hex_to_xy(camera, hex_size, q as f32,r as f32,s as f32);

//                     sprites[sprite_num] = GPUSprite {
//                         to_region: [world_x_pos, world_y_pos, hex_size, hex_size],
//                         from_region: [sprite_idx*from_x, from_y, from_width, from_height],
//                     };

//                     sprite_num = sprite_num + 1;

//                 }
//             }
//         }
//     }
// }

// fn add_unit_to_hexgrid(hexgrid:&HexGrid<tile::Tile>, unit: &Unit){
//     hexgrid.set(unit.location, unit);
// }

// fn move_unit_on_hexgrid(hexgrid:&HexGrid<tile::Tile>, unit: &mut Unit, new_pos: &coordinate::MultiCoord) {
//     let old_pos = unit.location;
//     unit.location = new_pos;
//     hexgrid.set(unit.location, unit);
//     hexgrid.remove(old_pos);
// }

fn units_to_sprites(camera:&gpuprops::GPUCamera, units:&[units::Unit], sprites: &mut[GPUSprite]){
    let mut sprite_num = 0;
    
    units.iter().for_each(|unit| {

        let mut sprite_idx = 0.0;

        match unit.name.as_str() {
            "Tank" => sprite_idx = 5.0,
            _ => sprite_idx = 1.0,
        }


        let (q,r,s) = (unit.location.to_cube().unwrap().x(), unit.location.to_cube().unwrap().y(), unit.location.to_cube().unwrap().z());

        let (world_x_pos, world_y_pos) = hex_to_xy(camera, hex_size, q as f32,r as f32,s as f32);

        sprites[sprite_num] = GPUSprite {
            to_region: [world_x_pos, world_y_pos, hex_size, hex_size],
            from_region: [sprite_idx*from_x, from_y, from_width, from_height],
        };

        sprite_num = sprite_num + 1;
    });
}

fn hex_to_xy(camera:&gpuprops::GPUCamera, full_size:f32, q:f32, r:f32, s:f32) -> (f32, f32) {

    let size:f32 = full_size / 2.0 as f32; //32 px

    //64 wide, 56 tall

    let x:f32 = (size * ((3.0/2.0) * q)) + camera.screen_size[0] / 2.0;
    let y:f32 = (size * (3.0_f32.sqrt()/2.0 * q + 3.0_f32.sqrt() * r)) + camera.screen_size[1] / 2.0;

    (x-size, y-size)
}


fn xy_to_hex(camera:&gpuprops::GPUCamera, full_size:f32, x:f32, y:f32) -> (i32, i32, i32) {

    let size:f32 = full_size / 2.0 as f32; //32 px

    let corrected_x = x - (camera.screen_size[0] / 2.0 as f32);
    let corrected_y = y - (camera.screen_size[1] / 2.0 as f32);

    let q:f32 = ((2.0 as f32 / 3.0 as f32) * corrected_x) / size;
    let r:f32 = ((((-1.0 as f32 / 3.0 as f32) * corrected_x) + ((3.0_f32.sqrt() / 3.0 as f32) * corrected_y))) / size;
    let s:f32 = -q - r;
       
// CHECK THIS
    let mut q_int = q.round() as i32;
    let mut r_int = r.round() as i32;
    let mut s_int = s.round() as i32;

    let q_diff = (q_int - q.round() as i32).abs();
    let r_diff = (r_int - r.round() as i32).abs();
    let s_diff = (s_int - s.round() as i32).abs();

    if q_diff > r_diff && q_diff > s_diff {
        q_int = -r_int-s_int;
    }else if r_diff > s_diff {
        r_int = -q_int-s_int;
    } else {
        s_int = -q_int-r_int;
    }

    (q_int,r_int,s_int)
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut gpu = wgpuimpl::WGPU::new(&window).await;
    let mut sprites = spriterenderer::SpriteRenderer::new(&gpu);

    let (texture0, tex_image) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let (texture1, tex_image) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let (texture2, tex_image) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let tex_image_w = tex_image.width();
    let tex_image_h = tex_image.height();

    let mut terrain_hexgrid = create_terrain_hexgrid();
    let mut unit_hexgrid = create_unit_hexgrid();

  
    let mut player1_units = vec![];
    let mut player2_units = vec![];

    let tank1 = units::Unit::tank(coordinate::MultiCoord::force_cube(0, 0, 0));
    let tank2 = units::Unit::tank(coordinate::MultiCoord::force_cube(5, -1, -4));

    player1_units.push(tank1);
    player1_units.push(tank2);

    let tank3 = units::Unit::tank(coordinate::MultiCoord::force_cube(-7, 0, 7));
    let tank4 = units::Unit::tank(coordinate::MultiCoord::force_cube(-8, 0, 8));

    player2_units.push(tank3);
    player2_units.push(tank4);

    // for unit in player1_units {
    //     add_unit_to_hexgrid(unit_hexgrid, unit);
    // }
    // for unit in player2_units {
    //     add_unit_to_hexgrid(unit_hexgrid, unit);
    // }


        // moving units is move_unit_fn

    sprites.add_sprite_group(&gpu, texture0, vec![GPUSprite::zeroed(); 1024]);   // 0 is terrain hex
    sprites.add_sprite_group(&gpu, texture1, vec![GPUSprite::zeroed(); 1024]);   // 1 is player 1 units
    sprites.add_sprite_group(&gpu, texture2, vec![GPUSprite::zeroed(); 1024]);   // 2 is player 2 units
    // Resverve extra space for each sprite sheet thing. LIke 1024 for the hex map and 1024 for the units, etc.
    // TODO: Make function to calculate size of hexgrid instead of 1024 above. Can also reallocate dymanically


    let mut camera = gpuprops::GPUCamera {
        screen_pos: [0.0, 0.0],
        screen_size: [gpu.config.width as f32, gpu.config.height as f32],
    };
    


    const TILE_NUM : usize = 1024; // usize is the type representing the offset in memory (32 on 32 bit systems, 64 on 64 etc. )
    // gpu.queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
    terrain_hexgrid_to_sprites(&camera, &terrain_hexgrid, sprites.get_sprites_mut(0));
    units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
    units_to_sprites(&camera, &player2_units, sprites.get_sprites_mut(2));
    sprites.refresh_sprites(&gpu, 0, 0..TILE_NUM);
    sprites.refresh_sprites(&gpu, 1, 0..TILE_NUM);
    sprites.refresh_sprites(&gpu, 2, 0..TILE_NUM);


    let mut input = input::Input::default();

    let mut global_tile = Tile::new(tile::Terrain::Plain);
    println!("*******{}",  matches!(global_tile.terrain, tile::Terrain::Plain));


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
                
                // let placeholder_coord = MultiCoord::default();
                
                if input.is_key_pressed(winit::event::VirtualKeyCode::Key1) {
                    global_tile = Tile::new(tile::Terrain::Plain);
                    println!("{}", "PLAINS");
                }
                if input.is_key_pressed(winit::event::VirtualKeyCode::Key4) {
                    global_tile = Tile::new(tile::Terrain::Mountain);
                    println!("{}", "MOUNTAIN");
                }
                if input.is_key_pressed(winit::event::VirtualKeyCode::Key2) {
                    global_tile = Tile::new(tile::Terrain::Coast);
                    println!("{}", "COAST");
                }
                if input.is_key_pressed(winit::event::VirtualKeyCode::Key3) {
                    global_tile = Tile::new(tile::Terrain::Forest);
                    println!("{}", "FOREST");
                }
                if input.is_key_down(winit::event::VirtualKeyCode::W) {
                    camera.screen_pos[1] += 10.0;
                }
                if input.is_key_down(winit::event::VirtualKeyCode::A) {
                    camera.screen_pos[0] -= 10.0;
                }
                if input.is_key_down(winit::event::VirtualKeyCode::S) {
                    camera.screen_pos[1] -= 10.0;
                }
                if input.is_key_down(winit::event::VirtualKeyCode::D) {
                    camera.screen_pos[0] += 10.0;
                }

                if input.is_key_down(winit::event::VirtualKeyCode::P) {
                    player1_units[0].location = coordinate::MultiCoord::force_cube(6, -9, 3);
                    units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
                    println!("{}", "moved")
                }
                if input.is_key_down(winit::event::VirtualKeyCode::O) {
                    player1_units[0].location = coordinate::MultiCoord::force_cube(0, 0, 0);
                    units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
                    println!("{}", "moved")
                }


                if input.is_mouse_down(winit::event::MouseButton::Left) {
                    // TODO screen -> multicord needed
                    let mouse_pos = input.mouse_pos();
                    // Normalize mouse clicks to be 00 at bottom left corner
                    // this stays ase gpu bc mouse coords normalize
                    // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32, ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32); //OG
                    // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32,
                    //                         ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32);
                    
                    let (x_norm, y_norm) = ((mouse_pos.x as f32 + camera.screen_pos[0]),
                                            ((mouse_pos.y as f32 - camera.screen_size[1]) * (-1.0 as f32)) + camera.screen_pos[1]);
                    // println!("{}, {}", x_norm, y_norm);

                    // let (q, r, s) = xy_to_hex(&camera, hex_size, x_norm * camera.screen_size[0] + camera.screen_pos[0], y_norm * camera.screen_size[1] + camera.screen_pos[1]); //OG
                    let (q, r, s) = xy_to_hex(&camera, hex_size, x_norm, y_norm);
                    // expecting inputs in screen space, not 0 to one so we multiply by camera size
                    // for this, if camera is on right, we want tiles to right, but in rendering we want left stuff.
                    //println!("{}, {}, {}", q, r, s);

                    println!("{} {}", x_norm, y_norm);
                    println!("{} {} {}", q, r, s);
                    

                    terrain_hexgrid.update(coordinate::MultiCoord::force_cube(q, r, s), global_tile);

                    terrain_hexgrid_to_sprites(&camera, &terrain_hexgrid, sprites.get_sprites_mut(0));
                }

                
                
                input.next_frame();
                sprites.set_camera(&gpu, &camera);
                
                let length = sprites.get_sprites(0).len(); // maybe only some of them instead of all?
                sprites.refresh_sprites(&gpu, 0, 0..length);
                let length = sprites.get_sprites(1).len();
                sprites.refresh_sprites(&gpu, 1, 0..length);
                let length = sprites.get_sprites(2).len();
                sprites.refresh_sprites(&gpu, 2, 0..length);


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
