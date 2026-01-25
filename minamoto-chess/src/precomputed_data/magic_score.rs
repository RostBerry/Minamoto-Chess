use super::magic_bitboard_data;

#[derive(Clone)]
pub struct MagicScore {
    score: f64,
    size: usize,
    max_index: usize
}

impl MagicScore {
    #[cold]
    /// Creates a *MagicScore* without validating the magic number
    pub fn from_validated(square: usize, max_index: usize, slider_index: usize) -> Self {
        let size = max_index * 8;
        let theoretical_min_size = magic_bitboard_data::get_min_lookup_square_size(slider_index, square);

        if size < theoretical_min_size {
            panic!("I geniunely don't know what's happening, max index {}, theoretical min size {}", max_index, theoretical_min_size);
        }

        let score = theoretical_min_size as f64 / size as f64;

        Self {
            score,
            max_index,
            size
        }
    }

    pub fn get_score(&self) -> f64 {
        self.score
    } 

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_max_index(&self) -> usize {
        self.max_index
    }
}

impl std::fmt::Display for MagicScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  Table Size: {}\n", self.size)?;
        write!(f, "  Overall Score: {:.2}%", self.score * 100.0)
    }
}