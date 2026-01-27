use crate::precomputed_data::magic_score::MagicScore;

const ROOK_MAGICS: [u64; 64] = [
    9403516572779159768, 18031991769276419, 36037730593931392, 72075186769559720, 144150458380851344, 144125101129343488, 2341874005868298496, 6989589371565383936,
    140740709614689, 1266774876372992, 9288811687198736, 1499276497826873472, 562992903360545, 5348037443190824, 1163054913073316112, 4612248970566828545,
    13997188191625445696, 148619062603161600, 2387190377267339264, 15564450209139327008, 282574795046948, 18437160797536768, 4398853391873, 621516539853572803,
    18017149436242624, 81082403732374048, 38843624097612096, 144187764433756177, 2264996101226624, 4785181979444224, 6918091986184982788, 13871228251214533637,
    11574251317228503040, 297237781585854528, 445856500590588032, 9223407328617836544, 2594777450772702210, 1175580454988817408, 18023400828044805, 3895636002916,
    457964846612488, 342278545789190208, 9223407221495332992, 2612160351784009760, 5476658793659039749, 595038118214762520, 576768620124438658, 282030168473633,
    70918684549376, 286287768584960, 9232943595082777088, 1192468807558890624, 2394787869622400, 2306970009710231616, 10376434287542075520, 306245341869580800,
    18155136406848321, 720593807627788417, 2324209364723831041, 6917612629181599745, 2198319602747056130, 2306124535731585537, 5801216866496549380, 576532428868550930
];

const ROOK_SHIFTS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];

const BISHOP_MAGICS: [u64; 64] = [
    10150734465081856, 9224510048586566656, 2310632636504605440, 298403059965952000, 1158590723998614040, 1441717098832200192, 19168920146903168, 11547515326736500736,
    1417305166971907, 2305860672811992192, 2269465049833473, 163019854479872, 4616191894661365793, 594475709701881872, 4611686576909452420, 9225624128770803776,
    10673533618892310532, 436919567764212864, 878204302458487104, 19140333113706508, 1153765931957094400, 75999347523322368, 145384028926124544, 36593947119880256,
    4901112904027342848, 577058888810763268, 864834374473089664, 1126174851923970, 4757354267147280384, 2254548610584706, 581479013286400, 9376565085139437568,
    9223688731235454976, 9156750101980160, 4613023333875728904, 9227066947291054592, 675541610555605025, 290483004941570048, 220821518354579976, 18157904393798144,
    1226111737422942208, 19283269858762768, 9800396152099574786, 11259274218971648, 3460664555948998912, 16194961869392320704, 4507999846556674, 4789474831846529,
    2817095368967872, 9953518955573280832, 2310365318272516112, 22593323441192960, 281543729873410, 175939308782084, 1229500634337150080, 4527823259711528,
    72660130839466016, 13979177108980142592, 615304643257389056, 37437134516388352, 2305843020219679232, 288230398968873474, 2287018839442450, 9009467150108688
];

const BISHOP_SHIFTS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6
];

const MAGIC_NUMBERS: [[u64; 64]; 2] = [
    BISHOP_MAGICS,
    ROOK_MAGICS
];

const SHIFTS: [[usize; 64]; 2] = [
    BISHOP_SHIFTS,
    ROOK_SHIFTS
];

pub enum MagicValidationResult {
    Valid(MagicScore),
    Invalid
}

pub fn validate_magic_number(magic: u64, shift: usize, square: usize, slider_index: usize) -> MagicValidationResult {
    let blocker_patterns = super::BITBOARD_DATA.get_blocker_patterns(square, slider_index);
    let blocker_pattern_count = blocker_patterns.len();

    if blocker_pattern_count != get_relevant_occupancy_cardinality(
        super::BITBOARD_DATA.get_relevant_occupancy(square, slider_index)
    ) {
        panic!("Incorrect blocker pattern generation");
    }

    let mut used_indices = Vec::with_capacity(blocker_pattern_count);
    used_indices.resize(blocker_pattern_count, false);
    let mut used_patterns = Vec::with_capacity(blocker_pattern_count);
    used_patterns.resize(blocker_pattern_count, 0);

    let mut max_index = 0;

    for pattern_index in 0..blocker_pattern_count {
        let pattern = blocker_patterns[pattern_index];
        let pseudo_legal_move_pattern = super::BITBOARD_DATA
            .get_pseudo_legal_moves(square, slider_index)[pattern_index];
        let lookup_index = generate_lookup_table_index(
            magic, 
            shift, 
            pattern, 
            super::BITBOARD_DATA.get_relevant_occupancy(square, slider_index),
        );

        if lookup_index >= blocker_pattern_count {
            return MagicValidationResult::Invalid;
        }

        if used_indices[lookup_index] {
            if used_patterns[lookup_index] != pseudo_legal_move_pattern {
                return MagicValidationResult::Invalid;
            }
        } 
        used_indices[lookup_index] = true;
        used_patterns[lookup_index] = pseudo_legal_move_pattern;

        max_index = max_index.max(lookup_index);
    }

    MagicValidationResult::Valid(MagicScore::from_validated(square, max_index, slider_index))
}

/// Returns the number of possible blocker patterns for the provided square
const fn get_relevant_occupancy_cardinality(relevant_occ: u64) -> usize {
    1 << relevant_occ.count_ones()
}

/// Returns the number of all distinct attack sets for the provided square for the rook
fn get_rook_distinct_attack_set_count(square: usize) -> usize {
    let mut result: usize = 1;
    let start_index: usize = 0;
    let end_index: usize = 4;
    for direction_index in start_index..end_index {
        let squares = super::SQUARE_DATA.get_squares_to_edge(square, direction_index);
        result *= if squares != 0 {squares} else {1};
    }

    result
}
/// Returns the number of all distinct attack sets for the provided square for the bishop
pub fn get_bishop_distinct_attack_set_count(square: usize) -> usize {
    let mut result: usize = 1;
    let start_index: usize = 4;
    let end_index: usize = 8;
    for direction_index in start_index..end_index {
        let squares = super::SQUARE_DATA.get_squares_to_edge(square, direction_index);
        result *= if squares != 0 {squares} else {1};
    }

    result
}
/// Returns the number of all distinct attack sets for the provided square
pub fn get_distinct_attack_set_count(slider_index: usize, square: usize) -> usize {
    match slider_index {
        0 => get_bishop_distinct_attack_set_count(square),
        1 => get_rook_distinct_attack_set_count(square),
        _ => panic!("Invalid slider index")
    }
}


/// Returns the minimal possible size for the provided square in the lookup table in bytes
pub fn get_min_lookup_square_size(slider_index: usize, square: usize) -> usize {
    (get_min_bits(get_distinct_attack_set_count(slider_index, square)) + 7) / 8 // Round up to nearest byte
}

/// Returns the minimal number of bits to store all indices of the lookup table for the provided number
pub fn get_min_bits(n: usize) -> usize {
    (usize::BITS - (n - 1).leading_zeros()) as usize
}

/// Returns the minimal number of bits to store all indices of the lookup table
pub fn get_shift(square: usize, slider_index: usize) -> usize {
    debug_assert!(slider_index < 2, "Slider index out of bounds");
    debug_assert!(square < 64, "Square index out of bounds");
    unsafe {
        *SHIFTS.get_unchecked(slider_index).get_unchecked(square)
    }
}

/// Returns the magic number for the provided square
pub fn get_magic_number(square: usize, slider_index: usize) -> u64 {
    debug_assert!(slider_index < 2, "Slider index out of bounds");
    debug_assert!(square < 64, "Square index out of bounds");
    unsafe {
        *MAGIC_NUMBERS.get_unchecked(slider_index).get_unchecked(square)
    }
}

/// Returns the magic bitboard lookup table index for the provided square and blocker pattern
pub fn generate_lookup_table_index(magic: u64, bits_to_shift: usize, blockers: u64, relevant_occ: u64) -> usize {
    debug_assert!(bits_to_shift < 64, "bits_to_shift must be less than 64");
    
    ((blockers & relevant_occ).wrapping_mul(magic) >> (64 - bits_to_shift)) as usize
}

