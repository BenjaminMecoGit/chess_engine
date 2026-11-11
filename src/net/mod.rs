/// the input layer tracks the relative position of 
/// 64 king positions * 
/// 64 squares for each piece
/// 2 colors for each square
/// 5 non king types
/// = 40960 possible indices that should be 0 or 1.
/// 
/// One does not need to track whose side it is, the engine just learns to say 
/// "how good is this from the perspective of the side to move"
/// 
/// Note that in the inputs vector, we can have at most 30 possible pieces. [IN A LATER IMPLEMENTATOION]

// const ACCUMULATOR_SIZE: usize = 128;
// const INPUT_DIM: usize = 64 * 64 * 2 * 5;
// const INPUT_SIZE = 30;
// [THE ABOVE IS FUTURE WORK]


/// [FOR NOW]:
/// the input layer tracks the absolute position of 
/// 64 squares * 
/// 2 colors for each square *
/// 6 types of pieces
/// + 0 * 4 castling possibilities (ignored)
/// + 0 * 16 en passants (ignored)
/// = 768 possible indices that should be 0 or 1.
/// 
/// the index of a piece is calculated as 64 * (kind_index * 2 + side_index) + square
/// 
/// Note that in the input vector, we can have at most 32 possible pieces 
/// We ignore things like castling and en passant. The turn is baked into the doubling of the input

use macroquad::prelude::*;

use crate::chess::{
    BoardState,
    Move,
    UnMove,
    Piece,
    PieceKind,
    Side,
};

const ACCUMULATOR_SIZE: usize = 32;
const INPUT_DIM: usize = 64 * 2 * 6;
const INPUT_SIZE: usize = 64;

// constants for units
const QA: i32 = 127;     // accumulator / activation scale
const QB: i32 = 64;      // output-weight scale
const SCALE: i32 = 400;  // network output → centipawns
pub struct Network {
    // weights and biases, these are shared by perspectives
    pub input_weights: Box<[[i8; ACCUMULATOR_SIZE]]>,
    pub input_bias: [i16; ACCUMULATOR_SIZE],
    pub output_weights: [i8; ACCUMULATOR_SIZE * 2],
    pub output_bias: i32,
    // data to compute with, these are indexed by perspective, 0 is white and 1 is black
    pub inputs: [[Option<usize>; INPUT_SIZE];2],
    pub accumulator_values: [[i16; ACCUMULATOR_SIZE];2],
}

// functions related to the training of a network
impl Network {
    
}


// functions related to the handling of a finished network
impl Network {

    // returns a new Network with randomly initialized weights in the specified ranges
    pub fn new(acc_range: usize, out_range: usize) -> Option<Self> {
        
        if acc_range > 127 ||out_range > 127 {
            return None;
        }
        
        let mut res = Network {
            input_weights: vec![[0i8; ACCUMULATOR_SIZE]; INPUT_DIM].into_boxed_slice(),
            input_bias: [0; ACCUMULATOR_SIZE],
            output_weights: [0; 2 * ACCUMULATOR_SIZE],
            output_bias: 0,
            inputs: [[None; INPUT_SIZE];2],
            accumulator_values: [[0; ACCUMULATOR_SIZE];2],
        };

        // fixing the seed in macroquad
        macroquad::rand::srand(0x5EED_1234);

        for i in 0..INPUT_DIM {
            for j in 0..ACCUMULATOR_SIZE {
                res.input_weights[i][j] = rand::gen_range(-(acc_range as isize), acc_range as isize) as i8;
            }
        }

        for k in 0..(2 * ACCUMULATOR_SIZE) {
            res.output_weights[k] = rand::gen_range(-(out_range as isize), out_range as isize) as i8;
        }

        return Some(res);
    }

    // refreshes a network by using the data from a borrowed boardstate
    pub fn refresh(&mut self, board: &BoardState) {

        //clear any previous input
        self.inputs = [[None; INPUT_SIZE];2];

        // first calculate all the indices for the pieces on the board
        let mut count: usize = 0;

        for sq in 0..64 {
            if let Some(piece) = board.piece_arr[sq] {
                
                // a sanity check for the amount of pieces on the board
                if count >= INPUT_SIZE {
                    panic!("There are too many pieces on the board");
                }

                // record the index, 
                // notice that means that the piece is of the color that is about to move
                self.inputs[0][count] = Some(data_to_index(&piece, sq as i8, &Side::White)); 
                self.inputs[1][count] = Some(data_to_index(&piece, sq as i8, &Side::Black));

                count += 1;
 
            }
        }

        // accumulate the values
        self.accumulate();

    }

    pub fn accumulate(&mut self) {
        
        for i in 0..ACCUMULATOR_SIZE {
            
            let mut res_0: i16 = self.input_bias[i];
            let mut res_1: i16 = self.input_bias[i];

            for k in 0..INPUT_SIZE {
                if let Some(index) = self.inputs[0][k] {
                    res_0 += self.input_weights[index as usize][i] as i16;
                }

                if let Some(index) = self.inputs[1][k] {
                    res_1 += self.input_weights[index as usize][i] as i16;
                }
            }

            self.accumulator_values[0][i] = res_0;
            self.accumulator_values[1][i] = res_1;

        }
    } 

    // evaluates the network from the accumulated values
    pub fn evaluate(&self, side_to_move: &Side) -> i32 {
        
        let us = side_to_index(side_to_move);
        let them = us ^ 1;

        let mut sum: i32 = self.output_bias;  // stored in QA*QB units

        for k in 0..ACCUMULATOR_SIZE {
            let a = self.accumulator_values[us][k].clamp(0, QA as i16) as i32;
            let b = self.accumulator_values[them][k].clamp(0, QA as i16) as i32;
            sum += a * self.output_weights[k] as i32;
            sum += b * self.output_weights[ACCUMULATOR_SIZE + k] as i32;
        }

        return sum * SCALE / (QA * QB);
    }

    // efficiently updates the network, 
    // given a move and on what board that move is to be made
    // this function assumes that it will be used before the move is applied to the board
    pub fn update(&mut self, m: &Move, board: &BoardState) {
        
        match *m {
            Move::Standard(from, to) => {

                // First find the moving piece
                let moving_piece = board.piece_arr[from as usize];

                // place the moving piece in both perspectives
                self.update_helper(to, &moving_piece, &Side::White);
                self.update_helper(to, &moving_piece, &Side::Black);

                // remove the old piece
                self.update_helper(from, &None, &Side::White);
                self.update_helper(from, &None, &Side::Black);
            },
            Move::Castle(from, to) => {
                // First find the moving king
                let moving_king = board.piece_arr[from as usize];
                // place the moving piece in both perspectives
                self.update_helper(to, &moving_king, &Side::White);
                self.update_helper(to, &moving_king, &Side::Black);
                // remove the old king
                self.update_helper(from, &None, &Side::White);
                self.update_helper(from, &None, &Side::Black);

                // Find the moving rook
                let rook_from = match to {
                    2 => 0,
                    6 => 7,
                    58 => 56, 
                    _ => 63,
                };

                if let Some(moving_rook) = board.piece_arr[rook_from as usize] {

                    let rook_to = match to {
                        2 => 3,
                        6 => 5,
                        58 => 59, 
                        _ => 61,
                    };

                    // move the rook in both perspectives
                    self.update_helper(rook_to, &Some(moving_rook), &Side::White);
                    self.update_helper(rook_to, &Some(moving_rook), &Side::Black);
                    // remove the old rook
                    self.update_helper(rook_from, &None, &Side::White);
                    self.update_helper(rook_from, &None, &Side::Black);
                }
            },
            Move::EnPassant(from, to) => {
                // First find the moving pawn
                let moving_pawn = board.piece_arr[from as usize];

                // place the moving pawn in both perspectives
                self.update_helper(to, &moving_pawn, &Side::White);
                self.update_helper(to, &moving_pawn, &Side::Black);

                // remove the moving pawn in both perspectives
                self.update_helper(from, &None, &Side::White);
                self.update_helper(from, &None, &Side::Black);

                // find the square of the pawn to be removed
                let removal_square = match moving_pawn.unwrap().side {
                    Side::White => to + 8,
                    Side::Black => to - 8,
                };
                
                // remove the captured pawn from both perspectives
                self.update_helper(removal_square, &None, &Side::White);
                self.update_helper(removal_square, &None, &Side::Black);

            },
            Move::Promotion(from, to,kind) => {

                // create a piece that will be placed as a promotion
                let pawn = board.piece_arr[from as usize].unwrap();
                let promotion = Piece{side: pawn.side, kind: kind};
                
                // remove the pawn standing on "from"
                self.update_helper(from, &None, &Side::White);
                self.update_helper(from, &None, &Side::Black);

                // add the promotion
                self.update_helper(to, &Some(promotion), &Side::White);
                self.update_helper(to, &Some(promotion), &Side::Black);
            },
        }

    }

    // efficiently updates the network, given an unmove and the board state
    // this function assumes that it will be used before the unmove is applied to the board
    pub fn un_update(&mut self, um: &UnMove, board: &BoardState) {
        
        // first unmake the movement of the moving piece
        match um.movement {
            Move::Standard(from, to) => {
                // First find the moving piece
                let moving_piece = board.piece_arr[to as usize];

                // place the moving piece in both perspectives
                self.update_helper(from, &moving_piece, &Side::White);
                self.update_helper(from, &moving_piece, &Side::Black);

                // fix the square that the piece had moved to
                self.update_helper(to, &um.capture, &Side::White);
                self.update_helper(to, &um.capture, &Side::Black);
            },
            Move::Castle(from, to) => {
                // First finds the moving king
                let moving_king = board.piece_arr[to as usize];
                // place the moving piece in both perspectives
                self.update_helper(from, &moving_king, &Side::White);
                self.update_helper(from, &moving_king, &Side::Black);
                // remove the old king
                self.update_helper(to, &None, &Side::White);
                self.update_helper(to, &None, &Side::Black);

                // Find the moving rook
                let rook_to = match to {
                        2 => 3,
                        6 => 5,
                        58 => 59, 
                        _ => 61,
                    };

                if let Some(moving_rook) = board.piece_arr[rook_to as usize] {

                    let rook_from = match to {
                        2 => 0,
                        6 => 7,
                        58 => 56, 
                        _ => 63,
                    };

                    // move the rook in both perspectives
                    self.update_helper(rook_from, &Some(moving_rook), &Side::White);
                    self.update_helper(rook_from, &Some(moving_rook), &Side::Black);
                    // remove the old rook
                    self.update_helper(rook_to, &None, &Side::White);
                    self.update_helper(rook_to, &None, &Side::Black);
                }
            },
            Move::EnPassant(from, to) => {
                // First find the moving pawn
                let moving_pawn = board.piece_arr[to as usize];

                // place the moving pawn in both perspectives
                self.update_helper(from, &moving_pawn, &Side::White);
                self.update_helper(from, &moving_pawn, &Side::Black);

                // remove the moving pawn in both perspectives
                self.update_helper(to, &None, &Side::White);
                self.update_helper(to, &None, &Side::Black);

                // find the square of the pawn to be placed back
                let removal_square = match moving_pawn.unwrap().side {
                    Side::White => to + 8,
                    Side::Black => to - 8,
                };
                let pawn = Piece{side:moving_pawn.unwrap().side.opposite(), kind: PieceKind::Pawn};
                
                // add the captured pawn from both perspectives
                self.update_helper(removal_square, &Some(pawn), &Side::White);
                self.update_helper(removal_square, &Some(pawn), &Side::Black);
            },
            Move::Promotion(from, to, _) => {
                // create a pawn that was the promoted one
                let pawn = Piece{side: board.side_to_move.opposite(), kind: PieceKind::Pawn};
                
                // remove the piece standing on "to"
                self.update_helper(to, &um.capture, &Side::White);
                self.update_helper(to, &um.capture, &Side::Black);

                // add the old pawn back
                self.update_helper(from, &Some(pawn), &Side::White);
                self.update_helper(from, &Some(pawn), &Side::Black);
            },
        }


    }

    // updates the network by just changing one input 
    // makes sure that there is a square (from whites perspective) with a certain Piece on it
    fn update_helper(
        &mut self, 
        square: i8,  
        piece: &Option<Piece>,
        side: &Side,
    ) {
        // calculate the index that should be activated or deactivated
        let index = 
            if let Some(piece) = piece {
                Some(data_to_index(&piece, square, &side))
            }
            else {
                None
            };
        
        let mut buffer: Option<usize> = None;
        let mut square_found = false;
        let s = side_to_index(side);
        let sq = match *side {
            Side::White => square,
            Side::Black => square ^ 56,
        };

        for i in 0..INPUT_SIZE {

            // save the buffer when the first empty is found
            if buffer.is_none() && self.inputs[s][i].is_none() {
                buffer = Some(i);
            }

            // if the correct square is encountered, then we need to update
            if let Some(ind) = self.inputs[s][i] && sq == index_to_square(ind) {
                if square_found {
                    // this is for cleaning, only happens if we have seen this square before
                    self.inputs[s][i] = None;
                }
                else {
                    
                    // ensure that the inputs remember this index
                    self.inputs[s][i] = index;

                    // remove the influence of the removed piece
                    for j in 0..ACCUMULATOR_SIZE {
                        self.accumulator_values[s][j] -= self.input_weights[ind][j] as i16;
                    }

                    // add the influence of the placed piece
                    if let Some(k) = index {
                        for j in 0..ACCUMULATOR_SIZE {
                            self.accumulator_values[s][j] += self.input_weights[k][j] as i16;
                        }
                    }

                    // record that the square has been updated
                    square_found = true;
                }   
            }
        }

        // if the square is not found and there was an empty buffer, 
        // then just record at the buffer
        if !square_found && let Some(k) = buffer {
            self.inputs[s][k] = index;

            // add the influence of the placed piece
            if let Some(k) = index {
                for j in 0..ACCUMULATOR_SIZE {
                    self.accumulator_values[s][j] += self.input_weights[k][j] as i16;
                }
            }
        }

    }

}

// helping function that returns the index for a given piece on a given square, from a given perspective
fn data_to_index(piece: &Piece, square: i8, perspective: &Side) -> usize {
    let sq = match *perspective {
        Side::White => square,
        Side::Black => square ^ 56, 
    } as usize;
   (64 * piece_to_index(piece, perspective) + sq) as usize
}

// helping function that returns the index for a given piece (no information about the square)
fn piece_to_index(piece: &Piece, perspective: &Side) -> usize {
    2 * kind_to_index(&piece.kind) + (side_to_index(&piece.side) ^ side_to_index(perspective))
}

// helping function that returns the index for a given piece type
fn kind_to_index(kind:&PieceKind) -> usize {
    match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,    
    }
}

// helping function that returns the index for a given piece color
fn side_to_index(side: &Side) -> usize {
    match side {
        Side::White => 0,
        Side::Black => 1,    
    }
}

// finds the square given the occupied index
fn index_to_square(index: usize) -> i8 {
    (index % 64) as i8
}