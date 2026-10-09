use crate::chess::{
    BoardState,
    //Move,
    //Piece,
    PieceKind,
    Side,
};

const KING_PROXIMITY: f64 = 0.05;
const KING_CONTROL: f64 = 0.8;
const KING_ACTIVITY_TABLE: [f64;64] = [1.;64];
const KING_SAFETY_TABLE: [f64;64] = king_safety_array();
const KING_WEIGHT: f64 = 0.9; 

const PAWN_ADVANCE: f64 = 0.04;
const PAWN_CONTROL: f64 = 1.5;

const KNIGHT_CONTROL: f64 = 0.9;
const KNIGHT_ATTACK: f64 =  0.9;
const KNIGHT_DEFENCE: f64 = 0.9;

const BISHOP_CONTROL: f64 = 1.0;
const BISHOP_ATTACK: f64 =  1.0;
const BISHOP_DEFENCE: f64 = 1.0;

const ROOK_CONTROL: f64 = 0.9;
const ROOK_ATTACK: f64 =  0.9;
const ROOK_DEFENCE: f64 = 0.9;

const QUEEN_CONTROL: f64 = 0.2;
const QUEEN_ATTACK: f64 =  0.2;
const QUEEN_DEFENCE: f64 = 0.2;

const INFLUENCE_COEFF: f64 = 0.1;
const CONTROL_WEIGHT_ARRAY: [f64; 64] = control_weight_array();
const CENTER_WEIGHT: f64 = 1.;
const ENDGAME_MATERIAL: f64 = 4.;

const fn control_weight_array() -> [f64;64] {
    let mut res: [f64; 64]= [0.; 64];
    let mut i = 0;
    
    loop {
        let mut j = 0;
        loop {
            res[(i*8 + j) as usize] = CENTER_WEIGHT*((7. - i as f64)*(i as f64) + (7. - j as f64)*(j as f64))/24.;
            j += 1;
            if j > 7 {
                break;
            }
        }
        i += 1;
        if i > 7 {
            break;
        }
    }

    return res;
}

const fn king_safety_array() -> [f64;64] {
    let mut res: [f64; 64]= [0.; 64];
    let mut i = 0;
    
    loop {
        let mut j = 0;
        loop {
            res[(i*8 + j) as usize] = 1. - KING_WEIGHT*((7. - i as f64)*(i as f64) + 0.5 * (7. - j as f64)*(j as f64))/18.;
            j += 1;
            if j > 7 {
                break;
            }
        }
        i += 1;
        if i > 7 {
            break;
        }
    }

    return res;
}


// methods for evaluating a position at face value [FIXED]
impl BoardState {

    // Evaluates a position at face value [FIXED]
    pub fn evaluation(&self) -> f64 {


        //a weight for the phase of the game
        let endgame_weight: f64 = self.phase_weight();

        // the material balance is more important later in the game
        let mut res: f64 = self.material * (1. + ENDGAME_MATERIAL * endgame_weight);

        // the total control exerted on the board
        // also calculates other things related to the result for the sake of optimization
        let control: [f64;64] = self.control(&mut res, endgame_weight);
   
        // calculating the net influence, with respect to some positional considerations
        for i in 0..64 {
            
            //the net control on this square
            let mut c: f64 = control[i];

            if c == 0.0 {
                continue;
            }
            
            // control should be more valueable when it is closer to the opposing king,
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

            c *= 1. + endgame_weight * KING_PROXIMITY / (1. + opposing_king_distance);

            c *= 1.0 + (1.0 - endgame_weight) * CONTROL_WEIGHT_ARRAY[i];

            res += INFLUENCE_COEFF * c;
        }
        
        return res;
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

                        let directions: [(i8, i8); 8] = [(-1,-1), (-1, 0), (-1,1), (0,1), (1,1), (1,0), (1,-1), (0,-1)];
                        for d in directions {
                            if let Some(destination) = self.stays_in_bounds(sq, d) {
                                control[destination as usize] += sign * KING_CONTROL * (KING_ACTIVITY_TABLE[sq as usize] * endgame_weight);
                            }
                        }

                        // the King is valuable when there are no opposing pieces nearby

                        // middlegame and endgame positioning for the king:
                        *res += sign * KING_ACTIVITY_TABLE[sq as usize] * endgame_weight * endgame_weight;
                        *res += sign * KING_SAFETY_TABLE[sq as usize] * (1.0 - endgame_weight * endgame_weight);

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
                                            control[destination as usize] += sign*QUEEN_ATTACK;
                                        }
                                        else {
                                            control[destination as usize] += sign*QUEEN_DEFENCE;
                                        }
                                        break;
                                    }
                                    control[destination as usize] += sign*QUEEN_CONTROL;
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
                                            control[destination as usize] += sign*ROOK_ATTACK;
                                        }
                                        else {
                                            control[destination as usize] += sign*ROOK_DEFENCE;
                                        }
                                        break;
                                    }
                                    control[destination as usize] += sign*ROOK_CONTROL;
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
                                            control[destination as usize] += sign*BISHOP_ATTACK;
                                        }
                                        else {
                                            control[destination as usize] += sign*BISHOP_DEFENCE;
                                        }
                                        break;
                                    }
                                    control[destination as usize] += sign*BISHOP_CONTROL;
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
                                        control[destination as usize] += sign*KNIGHT_CONTROL;
                                    }
                                    else if !self.is_side_at(destination as i8, piece.side) {
                                        control[destination as usize] += sign*KNIGHT_ATTACK;
                                    }
                                    else {
                                        control[destination as usize] += sign*KNIGHT_DEFENCE;
                                    }
                                }
                            }
                    },
                    PieceKind::Pawn => {
                        
                        // Control exerted by a pawn
                        if piece.side == Side::White {
                            if sq/8 > 0 { 
                                // capture to the left
                                if sq%8 > 0 {
                                    control[(sq - 9) as usize] += sign*PAWN_CONTROL;
                                }

                                // capture to the right
                                if sq%8 < 7 {
                                    control[(sq - 7) as usize] += sign*PAWN_CONTROL;
                                }
                            }
                        }
                        else {
                            if sq/8 < 7 { 
                                // capture to the left
                                if sq%8 > 0 {
                                    control[(sq + 7) as usize] += sign*PAWN_CONTROL;
                                }

                                // capture to the right
                                if sq%8 < 7 {
                                    control[(sq + 9) as usize] += sign*PAWN_CONTROL;
                                }
                            }
                        } 

                        // pawn structure considerations

                        // a more advanced pawn is morevaluable
                        let advancement = match piece.side {
                            Side::White => (6 - sq / 8).max(0),
                            Side::Black => (sq / 8 - 1).max(0),
                        };

                        

                        *res += sign * (advancement as f64) * PAWN_ADVANCE;   
                    },
                }
            }
        }

        return control;
    }

    fn phase_weight(&self) -> f64 {
        let mut phase = 0.0;

        for piece in self.piece_arr.iter().flatten() {
            phase += match piece.kind {
                PieceKind::Queen => 6.0,
                PieceKind::Rook => 3.0,
                PieceKind::Bishop |PieceKind::Knight => 1.0,
                PieceKind::Pawn |PieceKind::King => 0.0,
            };
        }

        // Starting phase: 2 queens * 6 + 4 rooks * 3 + 4 bishops * 1 + 4 knights * 1 = 32.
        return 1.0 - ((phase / 32.0) as f64).clamp(0.0, 1.0);
    }

}