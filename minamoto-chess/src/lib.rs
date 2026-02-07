use minamoto_chess_core::{board::Board, r#move::Move, move_generation::{attack_calculator::AttackCalculator, move_gen::{self}}};
use wasm_bindgen::prelude::*;

use crate::{attack_info::AttackInfo, board_extensions::BoardExtensions, fen_api::FenApi, game::GameState, move_extensions::MoveExtensions, piece_dto::{Piece, PieceColor, PiecePlacement, PieceType}, uci_move::{PromotionType, UciMove, UciMoveCreationResult}};

pub mod fen_api;
pub mod perft;
pub mod board_representation;
pub mod attack_calc_extensions;
pub mod board_extensions;
pub mod move_extensions;
pub mod uci_move;
pub mod config;
pub mod magic;
pub mod attack_info;
pub mod game;
pub mod move_gen_extensions;
pub mod piece_dto;

#[wasm_bindgen]
pub struct Game {
    board: Board,
    material: i8,
    graveyard: [Vec<Piece>; 2],
    history: Vec<String>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(js_name = fromFen)]
    pub fn from_fen(fen: &str) -> Game {
        let board = Board::from_fen(fen);
        let material = board.count_material();
        
        Game { 
            board, 
            material, 
            graveyard: [Vec::new(), Vec::new()], 
            history: vec![fen.to_string()] 
        }
    }

    #[wasm_bindgen(js_name = toFen)]
    pub fn to_fen(&self) -> String {
        self.board.to_fen()
    }

    #[wasm_bindgen(js_name = getCurrentGameState)]
    pub fn get_current_game_state(&self) -> GameState {
        let attack_calc = AttackCalculator::new(&self.board);

        let mut moves: Vec<Move> = move_gen::create_empty_move_buffer();
        move_gen::generate_moves(&mut moves, &self.board, &attack_calc);

        GameState::from_current_state(&self.board, &moves, &attack_calc)
    }

    #[wasm_bindgen(js_name = getAllPieces)]
    pub fn get_all_pieces(&self) -> Vec<PiecePlacement> {
        let mut piece_placements: Vec<PiecePlacement> = Vec::new();

        for color in 0..2 {
            for piece_type in 1..7 {
                let piece_bb = self.board.get_piece_bitboard(color, piece_type);
                let mut squares: Vec<usize> = Vec::new();
                for square in 0..64 {
                    if (piece_bb >> square) & 1 == 1 {
                        squares.push(square);
                    }
                }
                for square in squares {
                    piece_placements.push(PiecePlacement {
                        piece: Piece {
                            piece_type: PieceType::from_num(piece_type).unwrap(),
                            color: PieceColor::from_num(color).unwrap(),
                        },
                        square,
                    });
                }
            }
        }

        piece_placements
    }

    #[wasm_bindgen(js_name = getAttackInfo)]
    pub fn get_attack_info(&self) -> AttackInfo {
        let attack_calc = AttackCalculator::new(&self.board);
        let attack_info = AttackInfo::from_attack_calculator(&attack_calc);

        attack_info
    }

    #[wasm_bindgen(js_name = getAllLegalMoves)]
    pub fn get_all_legal_moves(&self) -> Vec<UciMove> {
        let attack_calc = AttackCalculator::new(&self.board);

        let mut moves: Vec<Move> = move_gen::create_empty_move_buffer();
        move_gen::generate_moves(&mut moves, &self.board, &attack_calc);

        let uci_moves: Vec<UciMove> = moves.into_iter()
            .map(|m| UciMove::from_move(m))
            .collect();

        uci_moves
    }

    #[wasm_bindgen(js_name = getLegalMovesFromSquare)]
    pub fn get_legal_moves_from_square(&self, square: usize) -> Vec<UciMove> {
        let attack_calc = AttackCalculator::new(&self.board);

        let mut all_moves: Vec<Move> = move_gen::create_empty_move_buffer();
        move_gen::generate_moves(&mut all_moves, &self.board, &attack_calc);

        let filtered_moves = move_gen_extensions::get_legal_moves_from_square(&all_moves, square);

        let uci_moves: Vec<UciMove> = filtered_moves.into_iter()
            .map(|m| UciMove::from_move(m))
            .collect();

        uci_moves
    }

    #[wasm_bindgen(js_name = makeMove)]
    pub fn make_move(&mut self, start_square: usize, target_square: usize, promotion: Option<PieceType>) -> Result<GameState, String> {
        let uci_move = UciMove {
            start_square,
            target_square,
            promotion: promotion.map(|pt| match pt {
                PieceType::Queen => PromotionType::Queen,
                PieceType::Knight => PromotionType::Knight,
                PieceType::Rook => PromotionType::Rook,
                PieceType::Bishop => PromotionType::Bishop,
                PieceType::King | PieceType::Pawn => panic!("Invalid promotion piece type"),
            }),
        };

        self._make_move(uci_move)
    }

    #[wasm_bindgen(js_name = makeMoveFromUci)]
    pub fn make_move_from_uci(&mut self, uci_move_str: &str) -> Result<GameState, String> {
        match UciMove::from_str(uci_move_str) {
            UciMoveCreationResult::Success(uci_move) => self._make_move(uci_move),
            UciMoveCreationResult::Failure => Err("Invalid UCI move format".to_string()),
        }
    }

    #[wasm_bindgen(js_name = getMaterialBalance)]
    pub fn get_material_balance(&self) -> i8 {
        self.material
    }

    #[wasm_bindgen(js_name = getGraveyard)]
    pub fn get_graveyard(&self, color: PieceColor) -> Vec<Piece> {
        let color_index = match color {
            PieceColor::White => 0,
            PieceColor::Black => 1,
        };
        self.graveyard[color_index].clone()
    }

    #[wasm_bindgen(js_name = getHistory)]
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }

    fn _make_move(&mut self, uci_move: UciMove) -> Result<GameState, String> {
        // Validate move is legal
        let mut legal_moves = move_gen::create_empty_move_buffer();
        let attack_calc = AttackCalculator::new(&self.board);
        move_gen::generate_moves(&mut legal_moves, &self.board, &attack_calc);

        let mov = Move::from_uci(uci_move, &self.board);

        let is_legal = legal_moves.iter().any(|m| {
            m.start_square == mov.start_square
                && m.target_square == mov.target_square
                && std::mem::discriminant(&m.move_type) == std::mem::discriminant(&mov.move_type)
        });

        if !is_legal {
            return Err("Illegal move".to_string());
        }

        let opponent_color = self.board.get_opposite_color();
        
        let move_record = self.board.make_move(mov);

        // If a piece was captured, add it to the graveyard
        if move_record.captured_piece_type != 0 {
            let captured_piece = Piece {
                piece_type: PieceType::from_num(move_record.captured_piece_type).unwrap(),
                color: PieceColor::from_num(opponent_color).unwrap(),
            };
            self.graveyard[opponent_color].push(captured_piece);
        }

        // Add a FEN string of the new position to history
        let new_fen = self.board.to_fen();
        self.history.push(new_fen);

        let attack_calc = AttackCalculator::new(&self.board);
        legal_moves.clear();
        move_gen::generate_moves(&mut legal_moves, &self.board, &attack_calc);

        let game_state = GameState::from_current_state(&self.board, &legal_moves, &attack_calc);
        Ok(game_state)
    }
}