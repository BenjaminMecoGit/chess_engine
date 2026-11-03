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

use rand::Rng;

use crate::chess::{
    coordinates_to_move,
    BoardState,
    ScoredMove,
    Move,
    UnMove,
    Piece,
    PieceKind,
    Side,
};

use crate::analysis::{
    //AnalysisNode,
    //AnalysisTree,
    //Solved,
    //mc_to_eval,
};

const ACCUMULATOR_SIZE: usize = 128;
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
    pub inputs: [[Option<u16>; INPUT_SIZE];2],
    pub accumulator_values: [[i16; ACCUMULATOR_SIZE];2],
}

impl Network {

    // returns a new Network with randomly initialized weights in the specified ranges
    pub fn new(acc_range: usize, out_range: usize) -> Option<Self> {
        
        if acc_range > 127 ||out_range > 127 {
            return None;
        }

        let mut rng = rand::thread_rng();
        
        let mut res = Network {
            input_weights: vec![[0i8; ACCUMULATOR_SIZE]; INPUT_DIM].into_boxed_slice(),
            input_bias: [0; ACCUMULATOR_SIZE],
            output_weights: [0; 2 * ACCUMULATOR_SIZE],
            output_bias: 0,
            inputs: [[None; INPUT_SIZE];2],
            accumulator_values: [[0; ACCUMULATOR_SIZE];2],
        };

        for i in 0..INPUT_DIM {
            for j in 0..ACCUMULATOR_SIZE {
                res.input_weights[i][j] = rng.gen_range(-(acc_range as isize)..=(acc_range as isize)) as i8;
            }
        }

        for k in 0..(2 * ACCUMULATOR_SIZE) {
            res.output_weights[k] = rng.gen_range(-(out_range as isize)..=(out_range as isize)) as i8;
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
                self.inputs[0][count] = Some((64 * piece_to_index(&piece, &Side::White) + sq) as u16); 
                self.inputs[1][count] = Some((64 * piece_to_index(&piece, &Side::Black) + (sq ^ 56)) as u16);

                count += 1;
 
            }
        }

        // accumulate the values
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
    pub fn update(&mut self, m: Move, board: BoardState) {
        
    }

    // efficiently updates the network, given an unmove
    pub fn un_update(&mut self, um: UnMove) {
        // TO BE IMPLEMENTED
    }

    // updates the network according to three parameters:
    // a pieve moving from "from", to "to"
    pub fn update_helper(
        &mut self, 
        moving_piece: (usize, usize, Piece),  
        remove_piece: Option<(Piece,usize)>, 
        add_piece: Option<(Piece, usize)>,
    ) {
        // first make sure to move the moving piece
        let moving_side = &moving_piece.2.side;

        let potential_index = 
        self.inputs[side_to_index(&moving_side)]
            .iter_mut()
            .find(|x| {
                if let Some(index) = x {
                    *index == moving_piece.0 as u16
                }
                else {
                    false
                }
            });
        
        let potential_index = if potential_index.is_some() {
                *potential_index.unwrap()
            }
            else {
                None
            };
        
        

    }

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