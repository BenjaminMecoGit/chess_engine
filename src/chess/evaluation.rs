use crate::chess::{
    BoardState,
    //Move,
    //Piece,
    PieceKind,
    Side,
};

const INFLUENCE_EXPONENT: f64 = 1.05;
const INFLUENCE_COEFF: f64 = 0.01;
const CENTER_PROXIMITY: f64 = 0.25;

const KING_PROXIMITY: f64 = 0.05;
const KING_EDGE: f64 = 0.05;
const KING_CONTROL: f64 = 1.;

const PAWN_ADVANCE: f64 = 0.04;
const PAWN_CONTROL: f64 = 1.0;

const KNIGHT_CONTROL: f64 = 1.0;
const KNIGHT_ATTACK: f64 =  1.0;
const KNIGHT_SIDE: f64 = 0.04;

const BISHOP_CONTROL: f64 = 1.0;
const BISHOP_ATTACK: f64 =  1.0;

const ROOK_CONTROL: f64 = 1.0;
const ROOK_ATTACK: f64 =  1.0;

const QUEEN_CONTROL_DIAGONAL: f64 = 0.2;
const QUEEN_ATTACK_DIAGONAL: f64 =  0.2;

const QUEEN_CONTROL_STRAIGHT: f64 = 0.2;
const QUEEN_ATTACK_STRAIGHT: f64 =  0.2;


// methods for evaluating a position at face value
impl BoardState {

    pub fn evaluation(&self) -> f64 {

        let mut res: f64 = 0.;
        let mut control: [f64;64] = [0.;64];

        // calculates the material balance while also finding the king position and total amount of pieces
        let mut white_king: i8 = -1;
        let mut black_king: i8 = -1;
        let mut total_pieces: f64 = 0.;

        for piece in &self.pieces {
            total_pieces += 1.;

            if piece.kind == PieceKind::King {
                if piece.side == Side::White {
                    white_king = piece.square;
                }
                else {
                    black_king = piece.square;
                }
            }

            res += match piece.side {
                Side::White => {
                    piece.get_value()
                }
                Side::Black => {
                    -piece.get_value()
                }
            };
        }

        

        // various strategic evaluations of the position
        for piece in &self.pieces {
            
            let sq = piece.square;

            let sign: f64 = match piece.side {
                Side::White => 1.,
                Side::Black => -1.,
            };

            let king_distance: f64 = match piece.side {
                Side::Black => ((white_king%8 - sq%8).abs() + (white_king/8 - sq/8).abs()) as f64,
                Side::White => ((black_king%8 - sq%8).abs() + (black_king/8 - sq/8).abs()) as f64,
            };

            // value for being close to the king, this is more important the fewer pieces there are on the board
            res += -sign*king_distance*KING_PROXIMITY/(1. + total_pieces).sqrt();
                    
            match piece.kind {
                PieceKind::King => {
                    // Control exerted by the king
                    if sq % 8 > 0 && sq/8 > 0 {
                        control[(sq - 9) as usize] += sign*KING_CONTROL;
                    }
                    if sq/8 > 0 {
                        control[(sq - 8) as usize] += sign*KING_CONTROL;
                    }
                    if sq % 8 < 7 && sq/8 > 0 {
                        control[(sq - 7) as usize] += sign*KING_CONTROL;
                    }
                    if sq % 8 < 7 {
                        control[(sq + 1) as usize] += sign*KING_CONTROL;
                    }
                    if sq % 8 < 7 && sq/8 < 7 {
                        control[(sq + 9) as usize] += sign*KING_CONTROL;
                    }
                    if sq/8 < 7 {
                        control[(sq + 8) as usize] += sign*KING_CONTROL;
                    }
                    if sq % 8 > 0 && sq/8 < 7 {
                        control[(sq + 7) as usize] += sign*KING_CONTROL;
                    }
                    if sq % 8 > 0 {
                        control[(sq - 1) as usize] += sign*KING_CONTROL;
                    }
                    
                    // the King is valuable when there are no opposing pieces nearby

                    // the King is in trouble when close to the edge
                    let v: f64 = (sq%8) as f64;
                    let w: f64 = (sq/8) as f64;
                    res += sign*((7.*v - v*v + 7.*w - w*w).sqrt())*KING_EDGE/total_pieces;
                },
                PieceKind::Queen => {
                    
                    // The control exerted by the queens
                    let directions: [i8; 4] = [-9,-7,7,9];

                    for d in directions {
                        let mut pos: i8 = sq;
                        loop { 
                            if d == -9 && (pos < 8 || pos%8 == 0) {
                                break;
                            }
                            else if d == -7 && (pos < 8 || pos%8 == 7) {
                                break;
                            }
                            else if d == 9 && (pos > 55 || pos%8 == 7) {
                                break;
                            }
                            else if d == 7 && (pos > 55 || pos%8 == 0) {
                                break;
                            }
                            else {
                                pos += d;
                            }

                            if self.is_piece_at(pos) {
                                if !self.is_side_at(pos, piece.side) {
                                    control[pos as usize] += sign*QUEEN_ATTACK_DIAGONAL;
                                }
                                break;
                            }
                            control[pos as usize] += sign*QUEEN_CONTROL_DIAGONAL;
                        }
                    }
                    

                    let directions: [i8; 4] = [-1,1,8,-8];

                    for d in directions {
                        let mut pos: i8 = sq;
                        loop { 
                            pos += d;
                            if (pos/8 != sq/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                break;
                            }
                            else if self.is_piece_at(pos) {
                                if !self.is_side_at(pos, piece.side) {
                                    control[pos as usize] += sign*QUEEN_ATTACK_STRAIGHT;
                                }
                                break;
                            }
                            control[pos as usize] += sign*QUEEN_CONTROL_STRAIGHT;
                        }
                    }
                },
                PieceKind::Rook => {

                    // Control exerted by a rook
                    let directions: [i8; 4] = [-1,1,8,-8];
                    for d in directions {
                        let mut pos: i8 = sq;
                        loop { 
                            pos += d;
                            if (pos/8 != sq/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                break;
                            }
                            else if self.is_piece_at(pos) {
                                if !self.is_side_at(pos, piece.side) {
                                    control[pos as usize] += sign*ROOK_ATTACK;
                                }
                                break;
                            }
                            control[pos as usize] += sign*ROOK_CONTROL;
                        }
                    }
                },
                PieceKind::Bishop => {

                    // Control exerted by a bishop
                    let directions: [i8; 4] = [-9,-7,7,9];
                    for d in directions {
                        let mut pos: i8 = sq;
                        loop { 
                            if d == -9 && (pos < 8 || pos%8 == 0) {
                                break;
                            }
                            else if d == -7 && (pos < 8 || pos%8 == 7) {
                                break;
                            }
                            else if d == 9 && (pos > 55 || pos%8 == 7) {
                                break;
                            }
                            else if d == 7 && (pos > 55 || pos%8 == 0) {
                                break;
                            }
                            else {
                                pos += d;
                            }

                            if self.is_piece_at(pos) {
                                if !self.is_side_at(pos, piece.side) {
                                    control[pos as usize] += sign*BISHOP_ATTACK;
                                }
                                break;
                            }
                            control[pos as usize] += sign*BISHOP_CONTROL;
                        }
                    }
        
                },
                PieceKind::Knight => {

                    // "a knight on the rim is dim"
                    let v: f64 = (sq%8) as f64;
                    res += sign*((7.*v - v*v).sqrt())*KNIGHT_SIDE;
                    
                    // Control exerted by a knight, 
                    // this is some silly code and can for sure be written more cleanly
                    if sq/8 > 1 && sq % 8 > 0 {
                        let pos = (sq - 2*8 - 1) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 > 1 && sq % 8 < 7 {
                        let pos = (sq - 2*8 + 1) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 > 0 && sq % 8 > 1 {
                        let pos = (sq - 1*8 - 2) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 > 0 && sq % 8 < 6 {
                        let pos = (sq - 1*8 + 2) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 < 6 && sq % 8 > 0 {
                        let pos = (sq + 2*8 - 1) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 < 6 && sq % 8 < 7 {
                        let pos = (sq + 2*8 + 1) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 < 7 && sq % 8 > 1 {
                        let pos = (sq + 1*8 - 2) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
                        }
                    }
                    if sq/8 < 7 && sq % 8 < 6 {
                        let pos = (sq + 1*8 + 2) as usize;
                        if !self.is_piece_at(pos as i8) {
                            control[pos] += sign*KNIGHT_CONTROL;
                        }
                        else if !self.is_side_at(pos as i8, piece.side) {
                            control[pos] += sign*KNIGHT_ATTACK;
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
                    

                    // a pawn is worth more the further advanced it is
                    let advancement = match piece.side {
                        Side::White => (6 - sq / 8).max(0),
                        Side::Black => (sq / 8 - 1).max(0),
                    };

                    res += sign * (advancement as f64) * PAWN_ADVANCE;

                    // pawn structure condiderations
                },
            }

        }
        
        // calculating the net influence, with respect to some positional considerations
        for i in 0..64 {

            let mut c: f64 = control[i];
            let endgame_weight: f64 = ((32. - total_pieces) / 24.).clamp(0.0, 1.0);

            // control should be more valueable when it is closer to the opposing king,
            // when there are fewer pieces on the board
            let white_king_distance: f64 = ((white_king/8 - (i/8) as i8).abs() + (white_king%8 - (i%8) as i8).abs()) as f64;
            let black_king_distance: f64 = ((black_king/8 - (i/8) as i8).abs() + (black_king%8 - (i%8) as i8).abs()) as f64;

            let opposing_king_distance: f64 = if c > 0. 
                {
                    black_king_distance
                }
                else {
                    white_king_distance
                };

            c *= 1. + endgame_weight * KING_PROXIMITY / (1. + opposing_king_distance);

            // control matters more in the center when there are more pieces on the board
            let centralization_weight: f64 = (1. - endgame_weight) * (((i%8) as f64 - 3.5).abs() + ((i/8) as f64 - 3.5).abs()).sqrt();
            c *= 1. + CENTER_PROXIMITY*centralization_weight;

            res += control_to_eval(c);
        }

        return res;
    }
}

// this encourages the control to be different 
fn control_to_eval(control: f64) -> f64 {
    return control.signum()*INFLUENCE_COEFF*control.abs().powf(INFLUENCE_EXPONENT);
}