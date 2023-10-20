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
use std::time::Instant;

use chickenwire::{coordinate::cube::Cube, prelude::MultiCoord};
use chickenwire::hexgrid::HexGrid;
use chickenwire::coordinate;

mod gamemap;

enum GameState {    //The bool indicates if the state needs to be initialized (true for yes)
    MainMenu(bool),
    MapCreator(bool),
    WarGame(bool),
    GameOver(bool, usize) //Usize indiciate which player won (1 or 2)
}

fn initalizeMapCreator(gpu:&wgpuimpl::WGPU, camera:&mut gpuprops::GPUCamera, hexgrid:&mut HexGrid<tile::Tile>, 
    texture:wgpu::Texture, sprites:&mut spriterenderer::SpriteRenderer, global_tile: &mut tile::Tile) {

    sprites.add_sprite_group(&gpu, texture, vec![GPUSprite::zeroed(); 1024]);   // 0 is terrain hex
    // Resverve extra space for each sprite sheet thing. LIke 1024 for the hex map and 1024 for the units, etc.
    // TODO: Make function to calculate size of hexgrid instead of 1024 above. Can also reallocate dymanically

    const TILE_NUM : usize = 1024;
    gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
    sprites.refresh_sprites(&gpu, 0, 0..TILE_NUM);

    *global_tile = Tile::new(tile::Terrain::Forest);

}

fn updateMapCreator(gpu:&wgpuimpl::WGPU, input:&mut input::Input, camera:&mut gpuprops::GPUCamera, hexgrid:&mut HexGrid<tile::Tile>, 
    sprites:&mut spriterenderer::SpriteRenderer, global_tile: &mut tile::Tile, game_state:&mut GameState) {
    
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key1) {
        *global_tile = Tile::new(tile::Terrain::Plain);
        println!("{}", "PLAINS");
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key4) {
        *global_tile = Tile::new(tile::Terrain::Mountain);
        println!("{}", "MOUNTAIN");
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key2) {
        *global_tile = Tile::new(tile::Terrain::Coast);
        println!("{}", "COAST");
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key3) {
        *global_tile = Tile::new(tile::Terrain::Forest);
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

    if input.is_key_pressed(winit::event::VirtualKeyCode::P) {
        *game_state = GameState::WarGame(true);
    }

    // if input.is_key_down(winit::event::VirtualKeyCode::P) {
    //     player1_units[0].location = coordinate::MultiCoord::force_cube(6, -9, 3);
    //     gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
    //     println!("{}", "moved")
    // }
    // if input.is_key_down(winit::event::VirtualKeyCode::O) {
    //     player1_units[0].location = coordinate::MultiCoord::force_cube(0, 0, 0);
    //     gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
    //     println!("{}", "moved")
    // }

    if input.is_key_pressed(winit::event::VirtualKeyCode::M) {
        gamemap::save_hexgrid(&hexgrid);
    }

    if input.is_key_pressed(winit::event::VirtualKeyCode::L) {
        gamemap::load_hexgrid(hexgrid);
        gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
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
        let (q, r, s) = gamemap::xy_to_hex(&camera, x_norm, y_norm);
        // expecting inputs in screen space, not 0 to one so we multiply by camera size
        // for this, if camera is on right, we want tiles to right, but in rendering we want left stuff.
        //println!("{}, {}, {}", q, r, s);

        println!("{} {}", x_norm, y_norm);
        println!("{} {} {}", q, r, s);
        

        hexgrid.update(coordinate::MultiCoord::force_cube(q, r, s), *global_tile);

        gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
    }
    
    
    sprites.set_camera(&gpu, &camera);
    
    // Only update the hexmap, do not include units
    let length = sprites.get_sprites(0).len(); 
    sprites.refresh_sprites(&gpu, 0, 0..length);
    // let length = sprites.get_sprites(1).len();
    // sprites.refresh_sprites(&gpu, 1, 0..length);
    // let length = sprites.get_sprites(2).len();
    // sprites.refresh_sprites(&gpu, 2, 0..length);

}

fn initalizeWarGame(gpu:&wgpuimpl::WGPU, camera:&mut gpuprops::GPUCamera, hexgrid:&mut HexGrid<tile::Tile>, 
    texture0:wgpu::Texture, texture1:wgpu::Texture, texture2:wgpu::Texture, sprites:&mut spriterenderer::SpriteRenderer, 
    player1_units:&mut Vec<units::Unit>, player2_units:&mut Vec<units::Unit>) {

    let mut p1_units = vec![];
    let mut p2_units = vec![];

    let tank1 = units::Unit::tank(coordinate::MultiCoord::force_cube(0, 0, 0));
    let tank2 = units::Unit::tank(coordinate::MultiCoord::force_cube(5, -1, -4));

    p1_units.push(tank1);
    p1_units.push(tank2);

    let tank3 = units::Unit::tank(coordinate::MultiCoord::force_cube(-7, 0, 7));
    let tank4 = units::Unit::tank(coordinate::MultiCoord::force_cube(-8, 0, 8));

    p2_units.push(tank3);
    p2_units.push(tank4);

    *player1_units = p1_units;
    *player2_units = p2_units;


    sprites.add_sprite_group(&gpu, texture0, vec![GPUSprite::zeroed(); 1024]);   // 0 is terrain hex
    sprites.add_sprite_group(&gpu, texture1, vec![GPUSprite::zeroed(); 1024]);   // 1 is player 1 units
    sprites.add_sprite_group(&gpu, texture2, vec![GPUSprite::zeroed(); 1024]);   // 2 is player 2 units
    // Resverve extra space for each sprite sheet thing. LIke 1024 for the hex map and 1024 for the units, etc.
    // TODO: Make function to calculate size of hexgrid instead of 1024 above. Can also reallocate dymanically

    const TILE_NUM : usize = 1024; // usize is the type representing the offset in memory (32 on 32 bit systems, 64 on 64 etc. )
    // gpu.queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
    gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
    gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
    gamemap::units_to_sprites(&camera, &player2_units, sprites.get_sprites_mut(2));
    sprites.refresh_sprites(&gpu, 0, 0..TILE_NUM);
    sprites.refresh_sprites(&gpu, 1, 0..TILE_NUM);
    sprites.refresh_sprites(&gpu, 2, 0..TILE_NUM);

}


fn updateWarGame(gpu:&wgpuimpl::WGPU, input:&mut input::Input, camera:&mut gpuprops::GPUCamera, hexgrid:&mut HexGrid<tile::Tile>, 
    sprites:&mut spriterenderer::SpriteRenderer, player1_units:&mut[units::Unit], player2_units:&mut[units::Unit], game_state:&mut GameState) {
    
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

    if input.is_key_pressed(winit::event::VirtualKeyCode::P) {
        *game_state = GameState::MapCreator(true);
    }




    if input.is_key_pressed(winit::event::VirtualKeyCode::Z) {
        player1_units[0].location = coordinate::MultiCoord::force_cube(6, -9, 3);
        gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
        println!("{}", "moved")
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::X) {
        player1_units[0].location = coordinate::MultiCoord::force_cube(0, 0, 0);
        gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
        println!("{}", "moved")
    }

    // if input.is_key_pressed(winit::event::VirtualKeyCode::M) {
    //     gamemap::save_hexgrid(&hexgrid);
    // }

    if input.is_key_pressed(winit::event::VirtualKeyCode::L) {
        gamemap::load_hexgrid(hexgrid);
        gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
    }

    


    // if input.is_mouse_down(winit::event::MouseButton::Left) {
    //     // TODO screen -> multicord needed
    //     let mouse_pos = input.mouse_pos();
    //     // Normalize mouse clicks to be 00 at bottom left corner
    //     // this stays ase gpu bc mouse coords normalize
    //     // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32, ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32); //OG
    //     // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32,
    //     //                         ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32);
        
    //     let (x_norm, y_norm) = ((mouse_pos.x as f32 + camera.screen_pos[0]),
    //                             ((mouse_pos.y as f32 - camera.screen_size[1]) * (-1.0 as f32)) + camera.screen_pos[1]);
    //     // println!("{}, {}", x_norm, y_norm);

    //     // let (q, r, s) = xy_to_hex(&camera, hex_size, x_norm * camera.screen_size[0] + camera.screen_pos[0], y_norm * camera.screen_size[1] + camera.screen_pos[1]); //OG
    //     let (q, r, s) = gamemap::xy_to_hex(&camera, x_norm, y_norm);
    //     // expecting inputs in screen space, not 0 to one so we multiply by camera size
    //     // for this, if camera is on right, we want tiles to right, but in rendering we want left stuff.
    //     //println!("{}, {}, {}", q, r, s);

    //     println!("{} {}", x_norm, y_norm);
    //     println!("{} {} {}", q, r, s);
        

    //     hexgrid.update(coordinate::MultiCoord::force_cube(q, r, s), *global_tile);

    //     gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
    // }
    
    sprites.set_camera(&gpu, &camera);
    
    let length = sprites.get_sprites(0).len(); // maybe only some of them instead of all?
    sprites.refresh_sprites(&gpu, 0, 0..length);
    let length = sprites.get_sprites(1).len();
    sprites.refresh_sprites(&gpu, 1, 0..length);
    let length = sprites.get_sprites(2).len();
    sprites.refresh_sprites(&gpu, 2, 0..length);

}


async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut gpu = wgpuimpl::WGPU::new(&window).await;
    let mut sprites = spriterenderer::SpriteRenderer::new(&gpu);

    //let (texture0, tex_image) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    // let tex_image_w = tex_image.width();
    // let tex_image_h = tex_image.height();
    
    let (texture0, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let (texture1, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    let (texture2, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
    
    let mut input = input::Input::default();
    let mut hexgrid = gamemap::create_hexgrid();

    let mut camera = gpuprops::GPUCamera {
        screen_pos: [0.0, 0.0],
        screen_size: [gpu.config.width as f32, gpu.config.height as f32],
    };

    // TIMING
    let mut acc = 0.0_f32;
    let mut prev_t = Instant::now();
    const SIM_DT : f32 = 1.0/60.0; // 60 simulation steps per second

    
    

    

    

    

    // Special Global Variables
    let mut game_state = GameState::MapCreator(true);

    // Main Menu

    // Map Creator
    let mut global_tile = Tile::new(tile::Terrain::Plain);

    // War Game
    let mut player1_units = vec![];
    let mut player2_units = vec![];

    // Game Over
    

    
    
    


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
                    match game_state {
                        GameState::MainMenu(needs_initialization) => {
                            if needs_initialization { 
                                /*initialize*/ 
                                game_state = GameState::MainMenu(false) 
                            }
                            // Handle main menu
                        }
                        GameState::MapCreator(needs_initialization) => {
                            if needs_initialization { 
                                // println!("Initializing");
                                let (texture_sheet, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                initalizeMapCreator(&gpu, &mut camera, &mut hexgrid, texture_sheet, &mut sprites, &mut global_tile);
                                game_state = GameState::MapCreator(false) 
                            }
                            println!("Updating Map");
                            updateMapCreator(&gpu, &mut input, &mut camera, &mut hexgrid, &mut sprites, &mut global_tile, &mut game_state);
                        }
                        GameState::WarGame(needs_initialization) => {
                            if needs_initialization { 
                                let (texture_sheet0, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                let (texture_sheet1, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                let (texture_sheet2, _) = load_texture("content/Game1Sheet.png", Some("Game1Sheet image"), &gpu.device, &gpu.queue).expect("Couldn't load Game1Sheet img");
                                initalizeWarGame(&gpu, &mut camera, &mut hexgrid, texture_sheet0, texture_sheet1, texture_sheet2, &mut sprites, &mut player1_units, &mut player2_units);
                                game_state = GameState::WarGame(false) 
                            }
                            println!("Updating War");
                            updateWarGame(&gpu, &mut input, &mut camera, &mut hexgrid, &mut sprites, &mut player1_units, &mut player2_units, &mut game_state);
                        }
                        GameState::GameOver(needs_initialization, winner) => {
                            if needs_initialization { 
                                /*initialize*/ 
                                game_state = GameState::GameOver(false, 0) 
                            }
                            // Handle main menu
                        }
                    }

                    // NOTE: This is when you should swap "new" keys and "old" keys for input handling!
                    // Otherwise you'll see several frames in a row where a key was just pressed/released.
                    
                    input.next_frame();

                    acc -= SIM_DT;
                }
                
                
                {
                // if input.is_key_pressed(winit::event::VirtualKeyCode::Key1) {
                //     global_tile = Tile::new(tile::Terrain::Plain);
                //     println!("{}", "PLAINS");
                // }
                // if input.is_key_pressed(winit::event::VirtualKeyCode::Key4) {
                //     global_tile = Tile::new(tile::Terrain::Mountain);
                //     println!("{}", "MOUNTAIN");
                // }
                // if input.is_key_pressed(winit::event::VirtualKeyCode::Key2) {
                //     global_tile = Tile::new(tile::Terrain::Coast);
                //     println!("{}", "COAST");
                // }
                // if input.is_key_pressed(winit::event::VirtualKeyCode::Key3) {
                //     global_tile = Tile::new(tile::Terrain::Forest);
                //     println!("{}", "FOREST");
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::W) {
                //     camera.screen_pos[1] += 10.0;
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::A) {
                //     camera.screen_pos[0] -= 10.0;
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::S) {
                //     camera.screen_pos[1] -= 10.0;
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::D) {
                //     camera.screen_pos[0] += 10.0;
                // }

                // if input.is_key_down(winit::event::VirtualKeyCode::P) {
                //     player1_units[0].location = coordinate::MultiCoord::force_cube(6, -9, 3);
                //     gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
                //     println!("{}", "moved")
                // }
                // if input.is_key_down(winit::event::VirtualKeyCode::O) {
                //     player1_units[0].location = coordinate::MultiCoord::force_cube(0, 0, 0);
                //     gamemap::units_to_sprites(&camera, &player1_units, sprites.get_sprites_mut(1));
                //     println!("{}", "moved")
                // }

                // if input.is_key_down(winit::event::VirtualKeyCode::M) {
                //     gamemap::save_hexgrid(&hexgrid);
                // }

                // if input.is_key_down(winit::event::VirtualKeyCode::L) {
                //     gamemap::load_hexgrid(&mut hexgrid);
                //     gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
                // }

                


                // if input.is_mouse_down(winit::event::MouseButton::Left) {
                //     // TODO screen -> multicord needed
                //     let mouse_pos = input.mouse_pos();
                //     // Normalize mouse clicks to be 00 at bottom left corner
                //     // this stays ase gpu bc mouse coords normalize
                //     // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32, ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32); //OG
                //     // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32,
                //     //                         ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32);
                    
                //     let (x_norm, y_norm) = ((mouse_pos.x as f32 + camera.screen_pos[0]),
                //                             ((mouse_pos.y as f32 - camera.screen_size[1]) * (-1.0 as f32)) + camera.screen_pos[1]);
                //     // println!("{}, {}", x_norm, y_norm);

                //     // let (q, r, s) = xy_to_hex(&camera, hex_size, x_norm * camera.screen_size[0] + camera.screen_pos[0], y_norm * camera.screen_size[1] + camera.screen_pos[1]); //OG
                //     let (q, r, s) = gamemap::xy_to_hex(&camera, x_norm, y_norm);
                //     // expecting inputs in screen space, not 0 to one so we multiply by camera size
                //     // for this, if camera is on right, we want tiles to right, but in rendering we want left stuff.
                //     //println!("{}, {}, {}", q, r, s);

                //     println!("{} {}", x_norm, y_norm);
                //     println!("{} {} {}", q, r, s);
                    

                //     hexgrid.update(coordinate::MultiCoord::force_cube(q, r, s), global_tile);

                //     gamemap::hexgrid_to_sprites(&camera, &hexgrid, sprites.get_sprites_mut(0));
                // }

                
                
                // input.next_frame();
                // sprites.set_camera(&gpu, &camera);
                
                // let length = sprites.get_sprites(0).len(); // maybe only some of them instead of all?
                // sprites.refresh_sprites(&gpu, 0, 0..length);
                // let length = sprites.get_sprites(1).len();
                // sprites.refresh_sprites(&gpu, 1, 0..length);
                // let length = sprites.get_sprites(2).len();
                // sprites.refresh_sprites(&gpu, 2, 0..length);
                }


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
