use std::fs;

use serde_json::Value;

use std::path::Path;

use crate::{precomputed_data};

use super::magic_bitboard_gen;
use crate::config;

fn get_magic_json_path(path: &str) -> String {
    let path_in_cwd = format!("minamoto-chess/src/{}", path);
    let path_in_parent = format!("../minamoto-chess/src/{}", path);

    if Path::new(&path_in_cwd).exists() {
        path_in_cwd
    } else if Path::new(&path_in_parent).exists() {
        path_in_parent
    } else {
        format!("src/{}", path)
    }
}

lazy_static::lazy_static! {
    pub static ref SHIFTS: [[usize; 64]; 2] = {
        let json_str = fs::read_to_string(get_magic_json_path(config::MAGIC_DATA_PATH))
            .expect("Failed to read magics.json");
        let json: Value = serde_json::from_str(&json_str)
            .expect("Failed to parse magics.json");
        
        let mut array = [[0; 64]; 2];
        
        let bishop_shifts = json["bishopShifts"].as_array()
            .expect("bishopShifts not found in magics.json");
        for (i, shift) in bishop_shifts.iter().enumerate() {
            array[precomputed_data::SLIDER_BISHOP_INDEX][i] = shift.as_u64().unwrap() as usize;
        }

        let rook_shifts = json["rookShifts"].as_array()
            .expect("rookShifts not found in magics.json");
        for (i, shift) in rook_shifts.iter().enumerate() {
            array[precomputed_data::SLIDER_ROOK_INDEX][i] = shift.as_u64().unwrap() as usize;
        }
        
        array
    };

    pub static ref MAGIC_NUMBERS: [[u64; 64]; 2] = {
        let json_str = fs::read_to_string(get_magic_json_path(config::MAGIC_DATA_PATH))
            .expect("Failed to read magics.json");
        let json: Value = serde_json::from_str(&json_str)
            .expect("Failed to parse magics.json");
        
        let mut array = [[0; 64]; 2];

        let bishop_magics = match json["bishopMagics"].as_array() {
            Some(magic_array) => magic_array.clone(),
            None => {
                array[precomputed_data::SLIDER_BISHOP_INDEX] = generate_all_magics(precomputed_data::SLIDER_BISHOP_INDEX);
                return array;
            }
        };

        let rook_magics = match json["rookMagics"].as_array() {
            Some(magic_array) => magic_array.clone(),
            None => {
                array[precomputed_data::SLIDER_ROOK_INDEX] = generate_all_magics(precomputed_data::SLIDER_ROOK_INDEX);
                return array;
            }
        };
        
        for (i, magic) in bishop_magics.iter().enumerate() {
            array[precomputed_data::SLIDER_BISHOP_INDEX][i] = magic.as_u64().unwrap();
        }

        for (i, magic) in rook_magics.iter().enumerate() {
            array[precomputed_data::SLIDER_ROOK_INDEX][i] = magic.as_u64().unwrap();
        }
        
        array
    };
}

fn generate_all_magics(slider_index: usize) -> [u64; 64] {
    let mut magics = [0; 64];
    for square in 0..64 {
        magics[square] = magic_bitboard_gen::generate_magic_number(square, slider_index).get_magic();
    }

    // Read existing JSON
    let json_str = fs::read_to_string(get_magic_json_path(config::MAGIC_DATA_PATH))
        .expect("Failed to read magics.json");
    let mut json: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse magics.json");

    // Update the appropriate field
    let field_name = match slider_index {
        precomputed_data::SLIDER_BISHOP_INDEX => "bishopMagics",
        precomputed_data::SLIDER_ROOK_INDEX => "rookMagics",
        _ => panic!("Invalid slider index")
    };
    
    json[field_name] = serde_json::Value::Array(
        magics.iter().map(|&m| serde_json::Value::Number(m.into())).collect()
    );

    // Write back to file
    fs::write(
        get_magic_json_path(config::MAGIC_DATA_PATH),
        serde_json::to_string_pretty(&json).unwrap()
    ).expect("Failed to write magics.json");

    magics
}