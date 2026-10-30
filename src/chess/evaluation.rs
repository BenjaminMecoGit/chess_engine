use crate::chess::{
    BoardState,
    Move,
    //Piece,
    PieceKind,
    Side,
}; 

const KING_PROXIMITY: f64 = 0.01;
const KING_CONTROL: f64 = 0.5;
const KING_ACTIVITY_TABLE: [f64;64] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.2, 1.2, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.3, 1.3, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.3, 1.3, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.2, 1.2, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

const KING_SAFETY_TABLE: [f64;64] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2,
    0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2,
    0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2,
    0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2,
    0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2,
    0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 0.2,
    1.0, 1.3, 1.3, 1.0, 1.0, 1.0, 1.3, 1.0,
    ];

const KING_WEIGHT: f64 = 0.5; 
const CASTLING_RIGHT_VALUE: f64 = 0.2;
const CASTLED_KING_BONUS: f64 = 0.6;

const PAWN_CONTROL: f64 = 1.;
const PAWN_ATTACK: f64 = 1.1;
const PAWN_DEFEND: f64 = 1.1;
const PAWN_ACTIVITY_TABLE: [f64;64] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];
const PAWN_ADVANCE: f64 = 0.9;
const PAWN_ADVANCE_TABLE: [f64;64] = [
    9.00, 9.00, 9.00, 9.00, 9.00, 9.00, 9.00, 9.00,
    0.14, 0.24, 0.24, 0.34, 0.34, 0.24, 0.24, 0.14,
    0.13, 0.23, 0.23, 0.33, 0.33, 0.23, 0.23, 0.12,
    0.12, 0.22, 0.22, 0.32, 0.32, 0.22, 0.22, 0.11,
    0.11, 0.21, 0.21, 0.31, 0.31, 0.21, 0.21, 0.11,
    0.10, 0.20, 0.20, 0.30, 0.30, 0.20, 0.20, 0.10,
    0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
    0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
    ];


const KNIGHT_CONTROL: f64 = 1.0;
const KNIGHT_ATTACK: f64 =  1.1;
const KNIGHT_DEFENCE: f64 = 1.1;
const KNIGHT_ACTIVITY_TABLE: [f64;64] = [
    1.0, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.0,
    1.0, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.2, 1.2, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.2, 1.2, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.2, 1.2, 1.2, 1.2, 1.2, 1.0,
    1.0, 1.1, 1.2, 1.2, 1.2, 1.2, 1.1, 1.0,
    1.0, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];



const BISHOP_CONTROL: f64 = 1.0;
const BISHOP_ATTACK: f64 =  1.1;
const BISHOP_DEFENCE: f64 = 1.1;
const BISHOP_ACTIVITY_TABLE: [f64;64] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

const ROOK_CONTROL: f64 = 0.9;
const ROOK_ATTACK: f64 =  1.0;
const ROOK_DEFENCE: f64 = 1.0;
const ROOK_ACTIVITY_TABLE: [f64;64] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

const QUEEN_CONTROL: f64 = 0.15;
const QUEEN_ATTACK: f64 =  0.2;
const QUEEN_DEFENCE: f64 = 0.2;
const QUEEN_ACTIVITY_TABLE: [f64;64] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

const INFLUENCE_COEFF: f64 = 0.07;
const ENDGAME_MATERIAL: f64 = 0.25;
const MATERIAL_WEIGHT: f64 = 2.0;

// methods for evaluating a position at face value
impl BoardState {

    // Evaluates a position at face value [FIXED]
    pub fn evaluation(&self) -> f64 {

        //a weight for the phase of the game
        let endgame_weight: f64 = self.phase_weight();

        // the material balance is more important later in the game
        let mut res: f64 = MATERIAL_WEIGHT * self.material * (1. + ENDGAME_MATERIAL * endgame_weight);

        // valuing having castling rights
        if self.white_short_castle || self.white_long_castle {
            res += CASTLING_RIGHT_VALUE;
        }

        if self.black_short_castle || self.black_long_castle {
            res -= CASTLING_RIGHT_VALUE;
        }

        // award having castled
        if self.white_king == 62 || self.white_king == 58 {
            res += CASTLED_KING_BONUS * (1.0 - endgame_weight);
        }

        if self.black_king == 6 || self.black_king == 2 {
            res -= CASTLED_KING_BONUS * (1.0 - endgame_weight);
        }


        // the total control exerted on the board
        // also calculates other things related to the result for the sake of optimization
        let control: [f64;64] = self.control(&mut res, endgame_weight);
   
        // calculating the net evaluation using the material, 
        // castling rights, 
        // and the positional considerations from the control on the board

        for i in 0..64 {
            
            //the net control on this square
            let mut c: f64 = control[i];

            if c == 0.0 {
                continue;
            }
            
            // control should be more valuable when it is closer to the opposing king,
            // when there are fewer pieces on the board
            let white_king_distance: f64 = ((self.white_king/8 - (i/8) as i8).abs() + (self.white_king%8 - (i%8) as i8).abs()) as f64;
            let black_king_distance: f64 = ((self.black_king/8 - (i/8) as i8).abs() + (self.black_king%8 - (i%8) as i8).abs()) as f64;

            let opposing_king_distance: f64 = if c > 0. 
                {
                    black_king_distance
                }
                else {
                    white_king_distance
                };

            c *= 1. + KING_PROXIMITY / (1. + opposing_king_distance);

            res += INFLUENCE_COEFF * c;
        }
        
        return res/MATERIAL_WEIGHT;
    }

    fn control(&self, res: &mut f64, endgame_weight: f64) -> [f64; 64] {

        let mut control = [0.;64];

        for i in 0..64 {
            if let Some(piece) = self.piece_arr[i] {

                let sq = i as i8;

                let sign: f64 = match piece.side {
                    Side::White => 1.,
                    Side::Black => -1.,
                };

                let perspective_position: usize = match piece.side {
                    Side::White => i as usize,
                    Side::Black => (i ^ 56) as usize,
                };

                /* 
                // distance to the king, this will be made use of of eventually
                let king_distance: f64 = match piece.side {
                    Side::Black => ((self.white_king%8 - sq%8).abs() + (self.white_king/8 - sq/8).abs()) as f64,
                    Side::White => ((self.black_king%8 - sq%8).abs() + (self.black_king/8 - sq/8).abs()) as f64,
                };
                */
                
                // this makes sure to calculate the control exerted by a piece
                match piece.kind {
                    PieceKind::King => {

                        // the positioning of thing, this is mostly relevant in the endgame
                        let directions: [(i8, i8); 8] = [(-1,-1), (-1, 0), (-1,1), (0,1), (1,1), (1,0), (1,-1), (0,-1)];
                        let king_activity_weight = endgame_weight.powi(3);
                        
                        for d in directions {
                            if let Some(destination) = self.stays_in_bounds(sq, d) {
                                control[destination as usize] += sign * KING_CONTROL * (KING_ACTIVITY_TABLE[perspective_position] * king_activity_weight);
                            }
                        }

                        // finally the positioning of the king, this is relevant in the opening and middlegame,
                        // less in the endgame
                        
                        // King safety on the back rank
                        *res += sign * KING_WEIGHT * KING_SAFETY_TABLE[perspective_position] * (1. - king_activity_weight);

                    },
                    PieceKind::Queen => {

                        // the queen can slide in any of 8 directions
                        let directions: [(i8,i8);8] = [(0,-1),(0,1),(-1,0),(1,0), (-1,-1),(-1,1),(1,-1),(1,1)];
                        for d in directions {
                            let mut pos: i8 = sq;
                            loop {
                                if let Some(destination) = self.stays_in_bounds(pos, d) {
                                    if self.is_piece_at(destination) {
                                        if !self.is_side_at(destination, piece.side) {
                                            control[destination as usize] += sign * QUEEN_ATTACK * QUEEN_ACTIVITY_TABLE[perspective_position];
                                        }
                                        else {
                                            control[destination as usize] += sign * QUEEN_DEFENCE * QUEEN_ACTIVITY_TABLE[perspective_position];
                                        }
                                        break;
                                    }
                                    control[destination as usize] += sign * QUEEN_CONTROL * QUEEN_ACTIVITY_TABLE[perspective_position];
                                    pos = destination;
                                }
                                else {
                                    break;
                                }
                            }
                        }
                    },
                    PieceKind::Rook => {

                        // Control exerted by a rook
                        let directions: [(i8,i8);4] = [(0,-1),(0,1),(-1,0),(1,0)];
                        for d in directions {
                            let mut pos: i8 = sq;
                            loop {
                                if let Some(destination) = self.stays_in_bounds(pos, d) {
                                    if self.is_piece_at(destination) {
                                        if !self.is_side_at(destination, piece.side) {
                                            control[destination as usize] += sign * ROOK_ATTACK * ROOK_ACTIVITY_TABLE[perspective_position];
                                        }
                                        else {
                                            control[destination as usize] += sign * ROOK_DEFENCE * ROOK_ACTIVITY_TABLE[perspective_position];
                                        }
                                        break;
                                    }
                                    control[destination as usize] += sign * ROOK_CONTROL * ROOK_ACTIVITY_TABLE[perspective_position];
                                    pos = destination;
                                }
                                else {
                                    break;
                                }
                            }
                        }
                    },
                    PieceKind::Bishop => {

                        // Control exerted by a bishop
                        let directions: [(i8,i8);4] = [(-1,-1),(-1,1),(1,-1),(1,1)];
                        for d in directions {
                            let mut pos: i8 = sq;
                            loop {
                                if let Some(destination) = self.stays_in_bounds(pos, d) {
                                    if self.is_piece_at(destination) {
                                        if !self.is_side_at(destination, piece.side) {
                                            control[destination as usize] += sign * BISHOP_ATTACK * BISHOP_ACTIVITY_TABLE[perspective_position];
                                        }
                                        else {
                                            control[destination as usize] += sign * BISHOP_DEFENCE * BISHOP_ACTIVITY_TABLE[perspective_position];
                                        }
                                        break;
                                    }
                                    control[destination as usize] += sign * BISHOP_CONTROL * BISHOP_ACTIVITY_TABLE[perspective_position];
                                    pos = destination;
                                }
                                else {
                                    break;
                                }
                            }
                        }
            
                    },
                    PieceKind::Knight => {

                        // Control exerted by a knight
                        let directions: [(i8,i8);8] = [(-2,-1), (-2,1), (-1,-2), (-1,2), (2,-1), (2,1), (1,-2), (1,2)];
                            for d in directions {
                                if let Some(destination) = self.stays_in_bounds(sq, d) {
                                    if !self.is_piece_at(destination as i8) {
                                        control[destination as usize] += sign * KNIGHT_CONTROL * KNIGHT_ACTIVITY_TABLE[perspective_position];
                                    }
                                    else if !self.is_side_at(destination as i8, piece.side) {
                                        control[destination as usize] += sign * KNIGHT_ATTACK * KNIGHT_ACTIVITY_TABLE[perspective_position];
                                    }
                                    else {
                                        control[destination as usize] += sign * KNIGHT_DEFENCE * KNIGHT_ACTIVITY_TABLE[perspective_position];
                                    }
                                }
                            }
                    },
                    PieceKind::Pawn => {
                        
                        let directions = match piece.side {
                            Side::White => [(-1,-1), (-1,1)],
                            Side::Black => [(1,-1), (1,1)],
                        };
                        
                        // Control exerted by a pawn
                        for d in directions {
                            if let Some(destination) = self.stays_in_bounds(sq, d) {
                                if !self.is_piece_at(destination as i8) {
                                    control[destination as usize] += sign * PAWN_CONTROL * PAWN_ACTIVITY_TABLE[perspective_position];
                                }
                                else if !self.is_side_at(destination as i8, piece.side) {
                                    control[destination as usize] += sign * PAWN_ATTACK * PAWN_ACTIVITY_TABLE[perspective_position];
                                }
                                else {
                                    control[destination as usize] += sign * PAWN_DEFEND * PAWN_ACTIVITY_TABLE[perspective_position];
                                }
                            } 
                        }

                        // pawn structure considerations below

                        // a more advanced pawn is more valuable
                        *res += sign * PAWN_ADVANCE * PAWN_ADVANCE_TABLE[perspective_position];   
                    },
                }
            }
        }

        return control;
    }

    // returns a priority value for a move, saying how urgently 
    // one should look at it in the analysis
    pub fn priority(&mut self, m: &Move) -> f64 {

        
        let mut res: f64 = 0.;
        let un_move = self.apply_move_unchecked(m);
        if self.is_in_check(self.side_to_move) {
            res += 1000.;
        }
        let result = self.un_move_unchecked(un_move);
        assert!(result, "Un move failed during priority calculation");

        // if a capture, this can only be of an opposing colour anyway
        if let Some(kind) = self.kind_at(m.get_to()) {
            res += 10. * match kind {
                PieceKind::King => 10.,
                PieceKind::Queen => 9.,
                PieceKind::Rook => 5.,
                PieceKind::Bishop => 3.,
                PieceKind::Knight => 3.,
                PieceKind::Pawn => 1.,
            };

            if let Some(kind) = self.kind_at(m.get_from()) {
                res -= match kind {
                    PieceKind::King => 10.,
                    PieceKind::Queen => 9.,
                    PieceKind::Rook => 5.,
                    PieceKind::Bishop => 3.,
                    PieceKind::Knight => 3.,
                    PieceKind::Pawn => 1.,
                };
            } 
        }

        return res.max(0.);
        
        
    }

    // returns an estimate for where in the game we are currently
    fn phase_weight(&self) -> f64 {
        let mut phase = 0.0;

        for piece in self.piece_arr.iter().flatten() {
            phase += match piece.kind {
                PieceKind::Queen => 10.0,
                PieceKind::Rook => 5.0,
                PieceKind::Bishop |PieceKind::Knight => 1.0,
                PieceKind::Pawn |PieceKind::King => 0.0,
            };
        }

        // Starting phase: 2 queens * 10 + 4 rooks * 5 + 4 bishops * 1 + 4 knights * 1 = 48.
        return 1.0 - ((phase / 48.0) as f64).clamp(0.0, 1.0);
    }

}