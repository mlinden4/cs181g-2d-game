use crate::gpuprops::GPUSprite;
use crate::gpuprops::GPUCamera;
use crate::units::Unit;
use chickenwire::{coordinate::cube::Cube, prelude::MultiCoord};
use chickenwire::hexgrid::HexGrid;
use chickenwire::coordinate;
use crate::tile;
use crate::tile::Terrain;
use std::path::PathBuf;

use std::fs;
use std::io;

const HEXGRID_RADIUS:i32 = 10;
const HEX_SIZE:f32 = 64.0;
    
const FROM_X:f32 = 1.0/10.0;
const FROM_Y:f32 = 1.0/2.0;
const FROM_WIDTH:f32 = 1.0/10.0; //448 x 64
const FROM_HEIGHT:f32 = 1.0/2.0;

// move this out eventually
pub fn create_hexgrid() -> HexGrid<tile::Tile> {

    let default_tile = tile::Tile::new(Terrain::Coast);

    let mut terrain_hexgrid: HexGrid<tile::Tile> = HexGrid::new_radial(HEXGRID_RADIUS as u32, default_tile);

    terrain_hexgrid

}

pub fn new_sprite(ss_x:f32, ss_y:f32, world_x:f32, world_y:f32, size:f32) -> GPUSprite {
    GPUSprite {
        to_region: [world_x, world_y, size, size],
        from_region: [ss_x*FROM_X, ss_y*FROM_Y, FROM_WIDTH, FROM_HEIGHT],
    }
}

pub fn new_squishable_sprite(ss_x:f32, ss_y:f32, world_x:f32, world_y:f32, width:f32, height:f32) -> GPUSprite {
    GPUSprite {
        to_region: [world_x, world_y, width, height],
        from_region: [ss_x*FROM_X, ss_y*FROM_Y, FROM_WIDTH, FROM_HEIGHT],
    }
}

pub fn hexgrid_to_sprites(camera:&GPUCamera, hexgrid:&HexGrid<tile::Tile>, sprites: &mut[GPUSprite]) {

    let mut sprite_num = 0;
    // let mut output_sprites:Vec<GPUSprite> = vec![];

    for q in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
        for r in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
            for s in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
                if q + r + s == 0 {

                    let hex = hexgrid.get(coordinate::MultiCoord::force_cube(q, r, s)).unwrap();

                    let mut sprite_idx_x = 0.0;
                    let mut sprite_idx_y = 0.0;
                    match hex.terrain {
                        Terrain::Coast => { sprite_idx_x = 3.0; sprite_idx_y = 0.0; }
                        Terrain::Plain => { sprite_idx_x = 4.0; sprite_idx_y = 0.0; }
                        Terrain::Mountain => { sprite_idx_x = 0.0; sprite_idx_y = 0.0; }
                        Terrain::Forest => { sprite_idx_x = 2.0; sprite_idx_y = 0.0; }
                        // _ => ();
                    }

                    let (world_x_pos, world_y_pos) = hex_to_xy(camera, q as f32,r as f32,s as f32);

                    sprites[sprite_num] = new_sprite(sprite_idx_x, sprite_idx_y, world_x_pos, world_y_pos, HEX_SIZE);
                    
                    // GPUSprite {
                    //     to_region: [world_x_pos, world_y_pos, hex_size, hex_size],
                    //     from_region: [sprite_idx*from_x, from_y, from_width, from_height],
                    // };

                    sprite_num = sprite_num + 1;

                }
            }
        }
    }
}

pub fn units_to_sprites(camera:&GPUCamera, units:&[Unit], sprites: &mut[GPUSprite]){
    let mut sprite_num = 0;
    
    units.iter().for_each(|unit| {

        let mut sprite_idx_x = 0.0;
        let mut sprite_idx_y = 0.0;

        match unit.name.as_str() {
            "Tank" => { sprite_idx_x = 5.0; sprite_idx_y = 0.0 },
            "Helicopter" => { sprite_idx_x = 6.0; sprite_idx_y = 0.0 },
            _ => { sprite_idx_x = 1.0; sprite_idx_y = 0.0; },
        }

        let (q,r,s) = (unit.location.to_cube().unwrap().x(), unit.location.to_cube().unwrap().y(), unit.location.to_cube().unwrap().z());

        let (world_x_pos, world_y_pos) = hex_to_xy(camera, q as f32,r as f32,s as f32);

        sprites[sprite_num] = new_sprite(sprite_idx_x, sprite_idx_y, world_x_pos, world_y_pos, HEX_SIZE);
        
        
        // GPUSprite {
        //     to_region: [world_x_pos, world_y_pos, HEX_SIZE, HEX_SIZE],
        //     from_region: [sprite_idx*from_x, from_y, from_width, from_height],
        // };

        sprite_num = sprite_num + 1;
    });
}

pub fn units_to_healthbars(camera:&GPUCamera, units:&[Unit], sprites: &mut[GPUSprite], player_num:usize){

    let mut sprite_num = 0;
    
    units.iter().for_each(|unit| {

        let mut sprite_idx_x = 0.0;
        let mut sprite_idx_y = 0.0;

        // match usize {
        //     1 => { sprite_idx_x = 5.0; sprite_idx_y = 0.0 },
        //     2 => { sprite_idx_x = 6.0; sprite_idx_y = 0.0 },
        //     _ => { sprite_idx_x = 1.0; sprite_idx_y = 0.0; },
        // }

        if player_num == 1 {
            sprite_idx_x = 0.0; sprite_idx_y = 1.0;
        }else {
            sprite_idx_x = 1.0; sprite_idx_y = 1.0;
        }

        let (q,r,s) = (unit.location.to_cube().unwrap().x(), unit.location.to_cube().unwrap().y(), unit.location.to_cube().unwrap().z());

        let (world_x_pos, world_y_pos) = hex_to_xy(camera, q as f32,r as f32,s as f32);

        let health_percent = (unit.hp as f32) / (unit.max_hp as f32);

        sprites[sprite_num] = new_squishable_sprite(sprite_idx_x, sprite_idx_y, world_x_pos, world_y_pos, health_percent*HEX_SIZE ,HEX_SIZE);

        sprite_num = sprite_num + 1;
    });

}

pub fn hex_to_xy(camera:&GPUCamera, q:f32, r:f32, s:f32) -> (f32, f32) {

    let size:f32 = HEX_SIZE / 2.0 as f32; //32 px

    //64 wide, 56 tall

    let x:f32 = (size * ((3.0/2.0) * q)) + camera.screen_size[0] / 2.0;
    let y:f32 = (size * (3.0_f32.sqrt()/2.0 * q + 3.0_f32.sqrt() * r)) + camera.screen_size[1] / 2.0;

    (x-size, y-size)
}

pub fn xy_to_hex(camera:&GPUCamera, x:f32, y:f32) -> (i32, i32, i32) {

    let size:f32 = HEX_SIZE / 2.0 as f32; //32 px

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

pub fn save_hexgrid(hexgrid:&HexGrid<tile::Tile>) {
    println!("Enter filename to store to:");
    let mut file_name = String::new(); // filepath for input

    // Read input from the user and handle any potential errors.
    match io::stdin().read_line(&mut file_name) {
        Ok(_) => {
            println!("Saving to: {}", file_name);
        }
        Err(error) => {
            eprintln!("Error reading input: {}", error);
        }
    }


//     let mut path = PathBuf::new();

//     path.push("./content");
//     path.push(file_name.trim());

//     path.set_extension("map");

//     // let file_path = "./content/".to_string() + &file_name + ".map";

//     // let map_string = fs::read(path);

    let file_path = "./content/".to_string() + &file_name.trim() + ".map";



    let mut map_string = String::new();

    

    for q in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
        for r in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
            for s in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
                if q + r + s == 0 {
                    let hex = hexgrid.get(coordinate::MultiCoord::force_cube(q, r, s)).unwrap();

                    match hex.terrain {
                        Terrain::Coast => { map_string.push('C') }
                        Terrain::Plain => { map_string.push('P') }
                        Terrain::Mountain => { map_string.push('M') }
                        Terrain::Forest => { map_string.push('F') }
                    }

                }
            }
        }
    }

    // Attempt to write the content to the file
    if let Err(e) = fs::write(path, map_string) {
        eprintln!("Error writing to file: {}", e);
    } else {
        println!("File created successfully.");
    }

}


pub fn load_default_hexgrid(hexgrid:&mut HexGrid<tile::Tile>) -> io::Result<()>{

    let binding = "./content/defaultMap.map".to_string();
    let file_path = binding.trim();

    let map_string = fs::read_to_string(file_path)?;

    let mut new_hexgrid = create_hexgrid();
    let mut idx_counter:usize = 0;

    for q in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
        for r in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
            for s in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
                if q + r + s == 0 {
                    
                    match map_string.as_bytes()[idx_counter] {
                        b'C' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Coast)) },
                        b'P' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Plain))},
                        b'M' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Mountain))},
                        b'F' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Forest))},
                        _ => ()
                    }

                    idx_counter += 1;

                }
            }
        }
    }

    *hexgrid = new_hexgrid;

    Ok(())
    
}

pub fn load_hexgrid(hexgrid:&mut HexGrid<tile::Tile>) -> io::Result<()>{
    println!("Enter filename to load from:");
    let mut file_name = String::new(); // filepath for input

    // Read input from the user and handle any potential errors.
    match io::stdin().read_line(&mut file_name) {
        Ok(_) => {
            println!("Loading from: {}", file_name);
        }
        Err(error) => {
            println!("Error reading input: {}", error);
        }
    }


//     let mut path = PathBuf::new();

//     path.push("./content");
//     path.push(file_name.trim());

//     path.set_extension("map");

    // let file_path = "./content/".to_string() + &file_name + ".map";

    let file_path = "./content/".to_string() + &file_name.trim() + ".map";


    let map_string = fs::read(path).unwrap();


    let mut new_hexgrid = create_hexgrid();
    let mut idx_counter:usize = 0;

    for q in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
        for r in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
            for s in -HEXGRID_RADIUS..=HEXGRID_RADIUS {
                if q + r + s == 0 {
                    
                    match map_string[idx_counter] {
                        b'C' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Coast)) },
                        b'P' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Plain))},
                        b'M' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Mountain))},
                        b'F' => { new_hexgrid.set(coordinate::MultiCoord::force_cube(q, r, s), tile::Tile::new(Terrain::Forest))},
                        _ => ()
                    }

                    idx_counter += 1;

                }
            }
        }
    }

    *hexgrid = new_hexgrid;

    Ok(())
    
}