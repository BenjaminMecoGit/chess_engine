use crate::chess::{
    BoardState,
    //Move,
    //Piece,
    PieceKind,
    Side,
};


const INFLUENCE_COEFF: f64 = 0.01;
const CENTER_PROXIMITY: f64 = 0.25;

const KING_PROXIMITY: f64 = 0.05;
const KING_EDGE: f64 = 0.05;
const KING_CONTROL: f64 = 1.;

const PAWN_ADVANCE: f64 = 0.04;
const PAWN_CONTROL: f64 = 1.0;

const KNIGHT_CONTROL: f64 = 1.0;
const KNIGHT_ATTACK: f64 =  1.0;
const KNIGHT_DEFENCE: f64 = 1.0;
const KNIGHT_SIDE: f64 = 0.04;

const BISHOP_CONTROL: f64 = 1.1;
const BISHOP_ATTACK: f64 =  1.1;
const BISHOP_DEFENCE: f64 = 1.1;

const ROOK_CONTROL: f64 = 1.0;
const ROOK_ATTACK: f64 =  1.0;
const ROOK_DEFENCE: f64 = 1.0;

const QUEEN_CONTROL_DIAGONAL: f64 = 0.2;
const QUEEN_ATTACK_DIAGONAL: f64 =  0.2;
const QUEEN_DEFENCE_DIAGONAL: f64 = 0.2;

const QUEEN_CONTROL_STRAIGHT: f64 = 0.2;
const QUEEN_ATTACK_STRAIGHT: f64 =  0.2;
const QUEEN_DEFENCE_STRAIGHT: f64 = 0.2;


// methods for evaluating a position at face value [FIXED]
impl BoardState {

    // Evaluates a position at face value [FIXED]
    pub fn evaluation(&self) -> f64 {

        let mut res: f64 = self.material;
        let mut control: [f64;64] = [0.;64];


        // various strategic evaluations of the position
        
        let king_proximity_scale = KING_PROXIMITY/(1. + self.total_pieces as f64).sqrt();
                
        for i in 0..64 {
            if let Some(piece) = self.piece_arr[i] {
                let sq = i as i8;

                let sign: f64 = match piece.side {
                    Side::White => 1.,
                    Side::Black => -1.,
                };

                let king_distance: f64 = match piece.side {
                    Side::Black => ((self.white_king%8 - sq%8).abs() + (self.white_king/8 - sq/8).abs()) as f64,
                    Side::White => ((self.black_king%8 - sq%8).abs() + (self.black_king/8 - sq/8).abs()) as f64,
                };

                // value for being close to the king, this is more important the fewer pieces there are on the board
                res += -sign*king_distance*king_proximity_scale;
                        
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
                        res += sign*((7.*v - v*v + 7.*w - w*w).sqrt())*KING_EDGE/(self.total_pieces as f64);
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
                                    else {
                                        control[pos as usize] += sign*QUEEN_DEFENCE_DIAGONAL;
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
                                    else {
                                        control[pos as usize] += sign*QUEEN_DEFENCE_STRAIGHT;
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
                                    else {
                                        control[pos as usize] += sign*ROOK_DEFENCE;
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
                                    else {
                                        control[pos as usize] += sign*BISHOP_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
                            else {
                                control[pos] += sign*KNIGHT_DEFENCE;
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
        }
        
        // calculating the net influence, with respect to some positional considerations

        let endgame_weight: f64 = ((32. - self.total_pieces as f64) / 24.).clamp(0.0, 1.0);

        for i in 0..64 {

            let mut c: f64 = control[i];
            

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

            // control matters more in the center when there are more pieces on the board
            let file_distance = ((i % 8) as f64 - 3.5).abs();
            let rank_distance = ((i / 8) as f64 - 3.5).abs();
            let center_distance = file_distance + rank_distance;

            // Distance is 1 on the four central squares and 7 in the corners.
            let center_proximity =
                ((7.0 - center_distance) / 6.0).clamp(0.0, 1.0);

            c *= 1.0 + CENTER_PROXIMITY * (1.0 - endgame_weight) * center_proximity;

            res += INFLUENCE_COEFF * c;
        }
        
        return res;
    }
}