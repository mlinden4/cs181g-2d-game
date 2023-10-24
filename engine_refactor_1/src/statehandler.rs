use crate::gpuprops::GPUSprite;
use crate::gpuprops::GPUCamera;
use crate::units;
use chickenwire::{coordinate::cube::Cube, prelude::MultiCoord};
use chickenwire::hexgrid::HexGrid;
use chickenwire::coordinate;
use crate::tile;
use crate::tile::Terrain;
use bytemuck::Zeroable;
use crate::Tile;
use crate::gpuprops;
use crate::spriterenderer;
use crate::wgpuimpl;
use crate::input;

use crate::gamemap;
use gamemap::*;


pub enum GameMode {    //The 1st bool indicates if the state needs to be initialized (true for yes)
    MainMenu(bool),
    MapCreator(bool),
    WarGame(bool, usize),  //Usize indicates which player's turn it is (1 or 2)
    GameOver(bool, usize) //Usize indiciate which player won (1 or 2)
}

pub struct GameState {
    pub game_mode: GameMode,
    pub hexgrid: HexGrid<tile::Tile>,
    pub player1_units: Vec<units::Unit>,
    pub player2_units: Vec<units::Unit>,
    pub global_tile: tile::Tile,
    pub moving_unit_location: Option<coordinate::MultiCoord>, // Which unit list, which index, which unit
}

pub fn initalizeMapCreator(gpu:&wgpuimpl::WGPU, camera:&mut gpuprops::GPUCamera, texture:wgpu::Texture, 
    sprites:&mut spriterenderer::SpriteRenderer, game_state:&mut GameState) {

    sprites.add_sprite_group(&gpu, texture, vec![GPUSprite::zeroed(); 1024]);   // 0 is terrain hex
    // Resverve extra space for each sprite sheet thing. LIke 1024 for the hex map and 1024 for the units, etc.
    // TODO: Make function to calculate size of hexgrid instead of 1024 above. Can also reallocate dymanically

    const TILE_NUM : usize = 1024;
    gamemap::hexgrid_to_sprites(&camera, &game_state.hexgrid, sprites.get_sprites_mut(0));
    sprites.refresh_sprites(&gpu, 0, 0..TILE_NUM);

    game_state.global_tile = Tile::new(tile::Terrain::Forest);
    game_state.moving_unit_location= None;
    game_state.game_mode = GameMode::MapCreator(false);
}

pub fn updateMapCreator(gpu:&wgpuimpl::WGPU, input:&mut input::Input, camera:&mut gpuprops::GPUCamera, 
    sprites:&mut spriterenderer::SpriteRenderer, game_state:&mut GameState) {
    
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key1) {
        game_state.global_tile = Tile::new(tile::Terrain::Plain);
        println!("{}", "PLAINS");
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key4) {
        game_state.global_tile = Tile::new(tile::Terrain::Mountain);
        println!("{}", "MOUNTAIN");
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key2) {
        game_state.global_tile = Tile::new(tile::Terrain::Coast);
        println!("{}", "COAST");
    }
    if input.is_key_pressed(winit::event::VirtualKeyCode::Key3) {
        game_state.global_tile = Tile::new(tile::Terrain::Forest);
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
        game_state.game_mode = GameMode::WarGame(true, 1);
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
        gamemap::save_hexgrid(&game_state.hexgrid);
    }

    if input.is_key_pressed(winit::event::VirtualKeyCode::L) {
        gamemap::load_hexgrid(&mut game_state.hexgrid);
        gamemap::hexgrid_to_sprites(&camera, &game_state.hexgrid, sprites.get_sprites_mut(0));
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
        

        game_state.hexgrid.update(coordinate::MultiCoord::force_cube(q, r, s), game_state.global_tile);

        gamemap::hexgrid_to_sprites(&camera, &game_state.hexgrid, sprites.get_sprites_mut(0));
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

pub fn initalizeWarGame(gpu:&wgpuimpl::WGPU, camera:&mut gpuprops::GPUCamera, sprite_sheet0: wgpu::Texture, 
    sprite_sheet1: wgpu::Texture, sprite_sheet2: wgpu::Texture, sprites:&mut spriterenderer::SpriteRenderer,  game_state:&mut GameState) {

    game_state.player1_units = Vec::default();
    game_state.player2_units = Vec::default();

    let tank1 = units::Unit::tank(coordinate::MultiCoord::force_cube(0, 0, 0));
    let tank2 = units::Unit::tank(coordinate::MultiCoord::force_cube(5, -1, -4));

    game_state.player1_units.push(tank1);
    game_state.player1_units.push(tank2);

    let tank3 = units::Unit::tank(coordinate::MultiCoord::force_cube(-7, 0, 7));
    let tank4 = units::Unit::tank(coordinate::MultiCoord::force_cube(-8, 0, 8));

    game_state.player2_units.push(tank3);
    game_state.player2_units.push(tank4);

    sprites.add_sprite_group(&gpu, sprite_sheet0, vec![GPUSprite::zeroed(); 1024]);   // 0 is terrain hex
    sprites.add_sprite_group(&gpu, sprite_sheet1, vec![GPUSprite::zeroed(); 1024]);   // 1 is player 1 units
    sprites.add_sprite_group(&gpu, sprite_sheet2, vec![GPUSprite::zeroed(); 1024]);   // 2 is player 2 units
    // Resverve extra space for each sprite sheet thing. LIke 1024 for the hex map and 1024 for the units, etc.
    // TODO: Make function to calculate size of hexgrid instead of 1024 above. Can also reallocate dymanically

    const TILE_NUM : usize = 1024; // usize is the type representing the offset in memory (32 on 32 bit systems, 64 on 64 etc. )
    // gpu.queue.write_buffer(&buffer_camera, 0, bytemuck::bytes_of(&camera));
    gamemap::hexgrid_to_sprites(&camera, &game_state.hexgrid, sprites.get_sprites_mut(0));
    gamemap::units_to_sprites(&camera, &game_state.player1_units, sprites.get_sprites_mut(1));
    gamemap::units_to_sprites(&camera, &game_state.player2_units, sprites.get_sprites_mut(2));
    sprites.refresh_sprites(&gpu, 0, 0..TILE_NUM);
    sprites.refresh_sprites(&gpu, 1, 0..TILE_NUM);
    sprites.refresh_sprites(&gpu, 2, 0..TILE_NUM);

    game_state.moving_unit_location= None;
    game_state.game_mode = GameMode::WarGame(false, 1); //Player 1's turn is first

}

pub fn updateWarGame(gpu:&wgpuimpl::WGPU, input:&mut input::Input, camera:&mut gpuprops::GPUCamera, 
    sprites:&mut spriterenderer::SpriteRenderer, game_state:&mut GameState) {
    
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
        game_state.game_mode = GameMode::MapCreator(true);
    }

    // if input.is_key_pressed(winit::event::VirtualKeyCode::Z) {
    //     game_state.player1_units[0].location = coordinate::MultiCoord::force_cube(6, -9, 3);
    //     gamemap::units_to_sprites(&camera, &game_state.player1_units, sprites.get_sprites_mut(1));
    //     println!("{}", "moved")
    // }
    // if input.is_key_pressed(winit::event::VirtualKeyCode::X) {
    //     game_state.player1_units[0].location = coordinate::MultiCoord::force_cube(0, 0, 0);
    //     gamemap::units_to_sprites(&camera, &game_state.player1_units, sprites.get_sprites_mut(1));
    //     println!("{}", "moved")
    // }

    // if input.is_key_pressed(winit::event::VirtualKeyCode::M) {
    //     gamemap::save_hexgrid(&hexgrid);
    // }

    if input.is_key_pressed(winit::event::VirtualKeyCode::L) {
        gamemap::load_hexgrid(&mut game_state.hexgrid);
        gamemap::hexgrid_to_sprites(&camera, &game_state.hexgrid, sprites.get_sprites_mut(0));
    }


    if input.is_mouse_pressed(winit::event::MouseButton::Left) {
        // TODO screen -> multicord needed
        let mouse_pos = input.mouse_pos();
        // Normalize mouse clicks to be 00 at bottom left corner
        // this stays ase gpu bc mouse coords normalize
        // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32, ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32); //OG
        // let (x_norm, y_norm) = (mouse_pos.x as f32 / gpu.config.width as f32,
        //                         ((gpu.config.height as f32) - (mouse_pos.y as f32))/ gpu.config.height as f32);
        
        let (x_norm, y_norm) = ((mouse_pos.x as f32 + camera.screen_pos[0]),
                                ((mouse_pos.y as f32 - camera.screen_size[1]) * (-1.0 as f32)) + camera.screen_pos[1]);

        // let (q, r, s) = xy_to_hex(&camera, hex_size, x_norm * camera.screen_size[0] + camera.screen_pos[0], y_norm * camera.screen_size[1] + camera.screen_pos[1]); //OG
        let (q, r, s) = gamemap::xy_to_hex(&camera, x_norm, y_norm);

        println!("{} {} {}", q, r, s);


        let clicked_coord = coordinate::MultiCoord::force_cube(q, r, s);

        // If there is a unit in moving unit, handle that
        if let Some(from_location) = game_state.moving_unit_location {
            if let GameMode::WarGame(_, 1) = game_state.game_mode {
                // If player 1, see if spot is available. If so move old unit there, and remove moving unit
                // If not empty, if in player1_units list, set the moving unit to that unit
                // If not empty, if in player2_units list, remove moving unit and do nothing (or something later on)
                let mut space_occupied = false;
                
                // Handle player 1 units
                for unit in &game_state.player1_units {
                    if unit.location == clicked_coord {
                        game_state.moving_unit_location = Some(unit.location);
                        space_occupied = true;
                        break;
                    }
                }

                // Handle player 2 units
                if !space_occupied {
                    for unit in &game_state.player2_units {
                        if unit.location == clicked_coord {
                            game_state.moving_unit_location = None;
                            space_occupied = true;
                            break;
                        }
                    }
                }

                // After looking through all units, no unit is there
                if !space_occupied {
                    for mut unit in &mut game_state.player1_units {
                        if unit.location == from_location {
                            unit.location = clicked_coord;
                            game_state.moving_unit_location = None;
                            break;
                        }
                    }
                    gamemap::units_to_sprites(&camera, &game_state.player1_units, sprites.get_sprites_mut(1));
                    game_state.game_mode = GameMode::WarGame(false, 2);   // Switch play to player 2
                }

                

            } else if let GameMode::WarGame(_, 2) = game_state.game_mode {
                // If player 2, search player2 units for matching multichoord
                // If a match, set it to the moving unit
                let mut space_occupied = false;
                
                // Handle player 2 units
                for unit in &game_state.player2_units {
                    if unit.location == clicked_coord {
                        game_state.moving_unit_location = Some(unit.location);
                        space_occupied = true;
                        break;
                    }
                }

                // Handle player 1 units
                if !space_occupied {
                    for unit in &game_state.player1_units {
                        if unit.location == clicked_coord {
                            // Do nothing
                            game_state.moving_unit_location = None;
                            space_occupied = true;
                            break;
                        }
                    }
                }

                // After looking through all units, no unit is there
                if !space_occupied {
                    for mut unit in &mut game_state.player2_units {
                        if unit.location == from_location {
                            unit.location = clicked_coord;
                            game_state.moving_unit_location = None;
                            break;
                        }
                    }
                    gamemap::units_to_sprites(&camera, &game_state.player2_units, sprites.get_sprites_mut(2));
                    game_state.game_mode = GameMode::WarGame(false, 1);   //Switch play to player 1
                }

                
            }

        // Otherwise, handle potentially selecting a new unit
        } else {
            if let GameMode::WarGame(_, 1) = game_state.game_mode {
                // If player 1, search player1 units for matching multichoord
                // If a match, set it to the moving unit
                for unit in &game_state.player1_units {
                    if unit.location == clicked_coord {
                        game_state.moving_unit_location = Some(unit.location);
                        break;
                    }
                }

            } else if let GameMode::WarGame(_, 2) = game_state.game_mode {
                // If player 2, search player2 units for matching multichoord
                // If a match, set it to the moving unit
                for unit in &game_state.player2_units {
                    if unit.location == clicked_coord {
                        game_state.moving_unit_location = Some(unit.location);
                        break;
                    }
                }
            }
        }
        

        // game_state.hexgrid.update(coordinate::MultiCoord::force_cube(q, r, s), game_state.global_tile);

        // gamemap::hexgrid_to_sprites(&camera, &game_state.hexgrid, sprites.get_sprites_mut(0));
    }
    
    sprites.set_camera(&gpu, &camera);
    
    let length = sprites.get_sprites(0).len(); // maybe only some of them instead of all?
    sprites.refresh_sprites(&gpu, 0, 0..length);
    let length = sprites.get_sprites(1).len();
    sprites.refresh_sprites(&gpu, 1, 0..length);
    let length = sprites.get_sprites(2).len();
    sprites.refresh_sprites(&gpu, 2, 0..length);

}
