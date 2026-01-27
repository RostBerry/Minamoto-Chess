use std::{fs, path::Path};

use rand::{rng, rngs::ThreadRng, RngCore};
use serde_json::Value;

use minamoto_chess_core::precomputed_data::{self, magic_bitboards::{self, MagicValidationResult, validate_magic_number}, square_magic::SquareMagic};

use crate::config;

fn generate_random_number(rng: &mut ThreadRng) -> u64 {
    let a = rng.next_u64() & 0xFFFF;
    let b = rng.next_u64() & 0xFFFF;
    let c = rng.next_u64() & 0xFFFF;
    let d = rng.next_u64() & 0xFFFF;
    a | (b << 16) | (c << 32) | (d << 48)
}

fn generate_magic_candidate(rng: &mut ThreadRng) -> u64 {
    generate_random_number(rng) & generate_random_number(rng) & generate_random_number(rng)
}

pub fn generate_magic_number(square: usize, slider_index: usize) -> SquareMagic {
    let mut rng = rng();

    let shift = magic_bitboards::get_shift(square, slider_index);
    
    loop {
        let candidate = generate_magic_candidate(&mut rng);
            match validate_magic_number(candidate, shift, square, slider_index) {
                MagicValidationResult::Valid(score) => {
                    return SquareMagic::with_score(square, candidate, shift, score);
                },
                MagicValidationResult::Invalid => continue
            }
    }
}

fn get_magic_json_path(path: &str) -> String {
    let path_in_cwd = format!("minamoto-chess-core/src/{}", path);
    let path_in_parent = format!("../minamoto-chess-core/src/{}", path);
    let path_in_grandparent = format!("../../minamoto-chess-core/src/{}", path);

    if Path::new(&path_in_cwd).exists() {
        path_in_cwd
    } else if Path::new(&path_in_parent).exists() {
        path_in_parent
    } else if Path::new(&path_in_grandparent).exists() {
        path_in_grandparent
    } else {
        format!("src/{}", path)
    }
}

fn generate_all_magics(slider_index: usize) -> [u64; 64] {
    let mut magics = [0; 64];
    for square in 0..64 {
        magics[square] = generate_magic_number(square, slider_index).get_magic();
    }

    // Read existing JSON
    let json_str = fs::read_to_string(get_magic_json_path(config::MAGIC_DUMP_PATH))
        .expect("Failed to read magics from JSON");
    let mut json: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse magics from JSON");

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
        get_magic_json_path(config::MAGIC_DUMP_PATH),
        serde_json::to_string_pretty(&json).unwrap()
    ).expect("Failed to write magics to JSON");

    magics
}