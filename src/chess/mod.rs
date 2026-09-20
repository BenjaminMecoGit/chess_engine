const KING_PROXIMITY: f64 = 0.03;
const KING_EDGE: f64 = 0.00;

const PAWN_ADVANCE: f64 = 0.04;

const KNIGHT_MOVEMENT: f64 = 0.03;
const KNIGHT_SIDE: f64 = 0.04;


const BISHOP_MOVEMENT: f64 = 0.03;
const ATTACKING_BISHOP: f64 = 3.;

const ROOK_MOVEMENT: f64 = 0.03;
const ATTACKING_ROOK: f64 = 3.;

const QUEEN_MOVEMENT_DIAGONAL: f64 = 0.03;
const ATTACKING_QUEEN_DIAGONAL: f64 = 3.;

const QUEEN_MOVEMENT_STRAIGHT: f64 = 0.03;
const ATTACKING_QUEEN_STRAIGHT: f64 = 3.;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardState {
    pub pieces: Vec<Piece>,
    pub side_to_move: Side,
    pub white_short_castle: bool,
    pub white_long_castle: bool,
    pub black_short_castle: bool,
    pub black_long_castle: bool,
    pub en_passant: Option<(i8,i8)>,
}

// methods for evaluating a position at face value
impl BoardState {

    pub fn evaluation(&self) -> f64 {

        let mut res:f64 = 0.;

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
            
            let sign: f64 = match piece.side {
                Side::White => 1.,
                Side::Black => -1.,
            };

            let king_distance: f64 = match piece.side {
                Side::Black => ((white_king%8 - piece.square%8).abs() + (white_king/8 - piece.square/8).abs()) as f64,
                Side::White => ((black_king%8 - piece.square%8).abs() + (black_king/8 - piece.square/8).abs()) as f64,
            };

            // value for being close to the king, this is more important the fewer pieces there are on the board
            res += -sign*king_distance*KING_PROXIMITY/(1. + total_pieces).sqrt();
                    
            match piece.kind {
                PieceKind::King => {
                    // the King is valuable when there are no opposing pieces nearby

                    // the King is in trouble when close to the edge
                    let v: f64 = (piece.square%8) as f64;
                    let w: f64 = (piece.square/8) as f64;
                    res += sign*(7.*v - v*v + 7.*w - w*w)*KING_EDGE/total_pieces;
                },
                PieceKind::Queen => {
                    // Queens are less valuable when they are far from the opposing king
                    res += -sign*king_distance*KING_PROXIMITY/2.;

                    // Queens are valuable when they can attack plenty of squares
                    let mut movement: f64 = 0.;
                    let directions: [i8; 4] = [-9,-7,7,9];

                    for d in directions {
                        let mut pos: i8 = piece.square;
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
                                if !self.is_side_at(pos, self.side_to_move) {
                                    // this means an attacking bishop is important
                                    movement += ATTACKING_QUEEN_DIAGONAL;
                                }
                                break;
                            }
                            movement += 1.;
                        }
                    }
                    res += sign*movement*QUEEN_MOVEMENT_DIAGONAL;

                    let directions: [i8; 4] = [-1,1,8,-8];
                    let mut movement: f64 = 0.0;

                    for d in directions {
                        let mut pos: i8 = piece.square;
                        loop { 
                            pos += d;
                            if (pos/8 != piece.square/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                break;
                            }
                            else if self.is_piece_at(pos) {
                                if !self.is_side_at(pos, self.side_to_move) {
                                    movement += ATTACKING_QUEEN_STRAIGHT;
                                }
                                break;
                            }
                            movement += 1.;
                        }
                    }
                    res += sign*movement*QUEEN_MOVEMENT_STRAIGHT;
                },
                PieceKind::Rook => {
                    // rooks are valuable when mobile

                    let directions: [i8; 4] = [-1,1,8,-8];
                    let mut movement: f64 = 0.0;

                    for d in directions {
                        let mut pos: i8 = piece.square;
                        loop { 
                            pos += d;
                            if (pos/8 != piece.square/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                break;
                            }
                            else if self.is_piece_at(pos) {
                                if !self.is_side_at(pos, self.side_to_move) {
                                    movement += ATTACKING_ROOK;
                                }
                                break;
                            }
                            movement += 1.;
                        }
                    }
                    res += sign*movement*ROOK_MOVEMENT;
                },
                PieceKind::Bishop => {
                    // Bishops tend to be valuable on long diagonals with lots of movement
                    // this piece can move on diagonals until there is a collision
                    let mut movement: f64 = 0.;
                    let directions: [i8; 4] = [-9,-7,7,9];

                    for d in directions {
                        let mut pos: i8 = piece.square;
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
                                if !self.is_side_at(pos, self.side_to_move) {
                                    // this means an attacking bishop is important
                                    movement += ATTACKING_BISHOP;
                                }
                                break;
                            }
                            movement += 1.;
                        }
                    }
                    res += sign*movement*BISHOP_MOVEMENT;

                },
                PieceKind::Knight => {
                    // "a knight on the rim is dim"
                    let v: f64 = (piece.square%8) as f64;
                    res += sign*(7.*v - v*v)*KNIGHT_SIDE;

                    // a knight is more valuable when it has many squares to move to
                    let mut movement: f64 = 0.;
                    let from = piece.square;
                    if from/8 > 1 && from % 8 > 0 {
                        if !self.is_side_at(from - 2*8 - 1, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 > 1 && from % 8 < 7 {
                        if !self.is_side_at(from - 2*8 + 1, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 > 0 && from % 8 > 1 {
                        if !self.is_side_at(from - 1*8 - 2, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 > 0 && from % 8 < 6 {
                        if !self.is_side_at(from - 1*8 + 2, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 < 6 && from % 8 > 0 {
                        if !self.is_side_at(from + 2*8 - 1, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 < 6 && from % 8 < 7 {
                        if !self.is_side_at(from + 2*8 + 1, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 < 7 && from % 8 > 1 {
                        if !self.is_side_at(from + 1*8 - 2, self.side_to_move) {
                            movement += 1.;
                        }
                    }
                    if from/8 < 7 && from % 8 < 6 {
                        if !self.is_side_at(from + 1*8 + 2, self.side_to_move) {
                            movement += 1.;
                        }
                    }

                    res += sign*KNIGHT_MOVEMENT*movement;

                },
                PieceKind::Pawn => {
                    // a pawn is worth more the further advanced it is
                    if piece.side == Side::White {
                        res += ((7 - piece.square/8) as f64)*PAWN_ADVANCE;
                    }
                    else {
                        res -= ((piece.square/8) as f64)*PAWN_ADVANCE;
                    }

                    // pawn structure condiderations
                },
            }
            
        }

        return res;
    }
}

// methods for making and getting legal moves on the board
impl BoardState { 

    // this will attempt to make a suggested move
    // if illegal, nothing will happen and false is returned
    // if the move is legal, it is implemented and true is returned
    pub fn apply_move_unchecked(&mut self, m: &Move) {

        match m {
            Move::Standard(from, to) => {

                // handle white's castling rights 
                if self.white_short_castle {
                    if *to == 63 || *from == 63 {
                        self.white_short_castle = false;
                    }
                    
                    if *from == 60 {
                        self.white_short_castle = false;
                        self.white_long_castle = false;
                    }
                }

                if self.white_long_castle {
                    if *to == 56 || *from == 56 {
                        self.white_long_castle = false;
                    }

                    if *from == 60 {
                        self.white_short_castle = false;
                        self.white_long_castle = false;
                    }
                }

                // handle blacks's castling rights 
                if self.black_short_castle {
                    if *to == 7 || *from == 7 {
                        self.black_short_castle = false;
                    }
                    
                    if *from == 4 {
                        self.black_short_castle = false;
                        self.black_long_castle = false;
                    }
                }

                if self.black_long_castle {
                    if *to == 0 ||*from == 0 {
                        self.black_long_castle = false;
                    }

                    if *from == 4 {
                        self.black_short_castle = false;
                        self.black_long_castle = false;
                    }
                }

                // handle en passant
                if self.is_kind_at(*from, PieceKind::Pawn) && (*to - *from).abs() == 16 {
                    // set en passant
                    self.en_passant = Some((*from,*to));
                } 
                else {
                    self.en_passant = None;
                }

                // remove any captured pieces
                self.pieces.retain(|piece| piece.square != *to);
                
                // move the piece on "from" to "to"
                for j in 0..self.pieces.len(){
                    if self.pieces[j].square == *from {
                        self.pieces[j].square = *to;
                    }
                }

            },
            Move::Castle(from, to) => {
                
                // handle castling rights
                match self.side_to_move {
                    Side::White => {
                        self.white_short_castle = false;
                        self.white_long_castle = false;
                    },
                    Side::Black => {
                        self.black_short_castle = false;
                        self.black_long_castle = false;
                    },
                }

                // en passant is now not valid
                self.en_passant = None;

                // move the king and the rook
                for j in 0..self.pieces.len(){
                    if self.pieces[j].square == *from {
                        self.pieces[j].square = *to;
                    }
                    if *to > *from && self.pieces[j].square == *from + 3 {
                        self.pieces[j].square = from + 1;
                    }
                    if *to < *from && self.pieces[j].square == *from - 4 {
                        self.pieces[j].square = *from - 1;
                    }
                }

            },
            Move::Promotion(from, to,kind) => {

                // handle white's castling rights 
                if self.white_short_castle {
                    if *to == 63 {
                        self.white_short_castle = false;
                    }
                }

                if self.white_long_castle {
                    if *to == 56 {
                        self.white_long_castle = false;
                    }
                }

                // handle blacks's castling rights 
                if self.black_short_castle {
                    if *to == 7 {
                        self.black_short_castle = false;
                    }
                }

                if self.black_long_castle {
                    if *to == 0 {
                        self.black_long_castle = false;
                    }
                }

                // en passant is now not valid
                self.en_passant = None;

                // remove any captured pieces and the promoting pawn
                self.pieces.retain(|piece| piece.square != *from && piece.square != *to);

                // add a piece of the kind that is promoted to
                self.pieces.push(
                    Piece {
                        side: self.side_to_move.clone(), 
                        kind: kind.clone(),
                        square: to.clone(),
                    }
                );
            },
            Move::EnPassant(from, to) => {

                // en passant never means a movement of the kings or rooks,
                // so this means that the castling rights are kept and need not be considered here

                // en passant is now not valid
                self.en_passant = None;

                // remove any captured pieces, 
                // note that en passant captures behind the moving pawn
                if self.side_to_move == Side::White {
                    self.pieces.retain(|piece| piece.square != *to + 8);
                }
                else {
                    self.pieces.retain(|piece| piece.square != *to - 8);
                }

                // move the pawn on "from" to "to"
                for j in 0..self.pieces.len(){
                    if self.pieces[j].square == *from {
                        self.pieces[j].square = *to;
                    }
                }   
            }
        }

        self.side_to_move = self.side_to_move.opposite();

    }

    // says if a given move is legal or not
    pub fn is_legal_move(&self, m: &Move) -> bool {
        return self.get_legal_moves().contains(m);
    }

    // returns all the legal moves in the current BoardState
    pub fn get_legal_moves(&self) -> Vec<Move> {
        let mut res: Vec<Move> = Vec::<Move>::new();

        for piece in &self.pieces {
            if piece.side == self.side_to_move {
                
                let from: i8 = piece.square;
                match piece.kind {
                    
                    PieceKind::King => {
                        // the moves on the immediate squares
                        if from % 8 > 0 && from/8 > 0 && !self.is_side_at(from - 9, self.side_to_move) {
                            res.push(Move::Standard(from,from - 9));
                        }
                        if from/8 > 0 && !self.is_side_at(from - 8, self.side_to_move) {
                            res.push(Move::Standard(from,from - 8));
                        }
                        if from % 8 < 7 && from/8 > 0 && !self.is_side_at(from - 7, self.side_to_move) {
                            res.push(Move::Standard(from,from - 7));
                        }
                        if from % 8 < 7 && !self.is_side_at(from + 1, self.side_to_move) {
                            res.push(Move::Standard(from,from + 1));
                        }
                        if from % 8 < 7 && from/8 < 7 && !self.is_side_at(from + 9, self.side_to_move) {
                            res.push(Move::Standard(from,from + 9));
                        }
                        if from/8 < 7 && !self.is_side_at(from + 8, self.side_to_move) {
                            res.push(Move::Standard(from,from + 8));
                        }
                        if from % 8 > 0 && from/8 < 7 && !self.is_side_at(from + 7, self.side_to_move) {
                            res.push(Move::Standard(from,from + 7));
                        }
                        if from % 8 > 0 && !self.is_side_at(from - 1, self.side_to_move) {
                            res.push(Move::Standard(from,from - 1));
                        }

                        // castling 
                        if self.side_to_move == Side::White {

                            // short
                            if self.white_short_castle && 
                               self.is_kind_at(60, PieceKind::King) && self.is_side_at(60, Side::White) &&
                               self.is_kind_at(63, PieceKind::Rook) && self.is_side_at(63, Side::White) &&
                               !self.is_piece_at(61) && !self.is_piece_at(62) &&
                               !self.is_attacked(60, self.side_to_move.opposite()) && 
                               !self.is_attacked(61, self.side_to_move.opposite()) &&
                               !self.is_attacked(62, self.side_to_move.opposite())
                            {
                                res.push(Move::Castle(60,62));
                            }

                            // long
                            if self.white_long_castle && 
                               self.is_kind_at(60, PieceKind::King) && self.is_side_at(60, Side::White) &&
                               self.is_kind_at(56, PieceKind::Rook) && self.is_side_at(56, Side::White) &&
                               !self.is_piece_at(57) && !self.is_piece_at(58) && !self.is_piece_at(59) &&
                               !self.is_attacked(60, self.side_to_move.opposite()) && 
                               !self.is_attacked(59, self.side_to_move.opposite()) &&
                               !self.is_attacked(58, self.side_to_move.opposite())
                            {
                                res.push(Move::Castle(60,58));
                            }
                        }
                        else {
                            // short
                            if self.black_short_castle && 
                               self.is_kind_at(4, PieceKind::King) && self.is_side_at(4, Side::Black) &&
                               self.is_kind_at(7, PieceKind::Rook) && self.is_side_at(7, Side::Black) &&
                               !self.is_piece_at(5) && !self.is_piece_at(6) &&
                               !self.is_attacked(4, self.side_to_move.opposite()) && 
                               !self.is_attacked(5, self.side_to_move.opposite()) &&
                               !self.is_attacked(6, self.side_to_move.opposite())
                            {
                                res.push(Move::Castle(4,6));
                            }

                            // long
                            if self.black_long_castle && 
                               self.is_kind_at(4, PieceKind::King) && self.is_side_at(4, Side::Black) &&
                               self.is_kind_at(0, PieceKind::Rook) && self.is_side_at(0, Side::Black) &&
                               !self.is_piece_at(1) && !self.is_piece_at(2) && !self.is_piece_at(3) &&
                               !self.is_attacked(4, self.side_to_move.opposite()) && 
                               !self.is_attacked(3, self.side_to_move.opposite()) &&
                               !self.is_attacked(2, self.side_to_move.opposite())
                            {
                                res.push(Move::Castle(4,2));
                            }
                        }


                    }, 
                    PieceKind::Queen => {

                        // this piece can move on files and ranks until there is a collision
                        let directions: [i8; 4] = [-1,1,8,-8];

                        for d in directions {
                            let mut pos :i8 = from;
                            loop { 
                                pos += d;
                                if (pos/8 != from/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                    break;
                                }
                                else if self.is_piece_at(pos) {
                                    if !self.is_side_at(pos, self.side_to_move) {
                                        res.push(Move::Standard(from, pos));
                                    }
                                    break;
                                }
                                res.push(Move::Standard(from,pos));
                            }
                        }

                        // and also on diagonals until there is a collision
                        let directions: [i8; 4] = [-9,-7,7,9];

                        for d in directions {
                            let mut pos :i8 = from;
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
                                    if !self.is_side_at(pos, self.side_to_move) {
                                        res.push(Move::Standard(from, pos));
                                    }
                                    break;
                                }
                                res.push(Move::Standard(from,pos));
                            }
                        }
                        
                    }, 
                    PieceKind::Rook => {
                        // this piece can move on files and ranks until there is a collision
                        let directions: [i8; 4] = [-1,1,8,-8];

                        for d in directions {
                            let mut pos :i8 = from;
                            loop { 
                                pos += d;
                                if (pos/8 != from/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                    break;
                                }
                                else if self.is_piece_at(pos) {
                                    if !self.is_side_at(pos, self.side_to_move) {
                                        res.push(Move::Standard(from, pos));
                                    }
                                    break;
                                }
                                res.push(Move::Standard(from,pos));
                            }
                        }
                    }, 
                    PieceKind::Bishop => {
                        // this piece can move on diagonals until there is a collision
                        let directions: [i8; 4] = [-9,-7,7,9];

                        for d in directions {
                            let mut pos :i8 = from;
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
                                    if !self.is_side_at(pos, self.side_to_move) {
                                        res.push(Move::Standard(from, pos));
                                    }
                                    break;
                                }
                                res.push(Move::Standard(from,pos));
                            }
                        }
                    }, 
                    PieceKind::Knight => {

                        // this piece can only move in L-shapes
                        if from/8 > 1 && from % 8 > 0 {
                            if !self.is_side_at(from - 2*8 - 1, self.side_to_move) {
                                res.push(Move::Standard(from, from - 2*8 - 1));
                            }
                        }
                        if from/8 > 1 && from % 8 < 7 {
                            if !self.is_side_at(from - 2*8 + 1, self.side_to_move) {
                                res.push(Move::Standard(from, from - 2*8 + 1));
                            }
                        }
                        if from/8 > 0 && from % 8 > 1 {
                            if !self.is_side_at(from - 1*8 - 2, self.side_to_move) {
                                res.push(Move::Standard(from, from - 1*8 - 2));
                            }
                        }
                        if from/8 > 0 && from % 8 < 6 {
                            if !self.is_side_at(from - 1*8 + 2, self.side_to_move) {
                                res.push(Move::Standard(from, from - 1*8 + 2));
                            }
                        }
                        if from/8 < 6 && from % 8 > 0 {
                            if !self.is_side_at(from + 2*8 - 1, self.side_to_move) {
                                res.push(Move::Standard(from, from + 2*8 - 1));
                            }
                        }
                        if from/8 < 6 && from % 8 < 7 {
                            if !self.is_side_at(from + 2*8 + 1, self.side_to_move) {
                                res.push(Move::Standard(from, from + 2*8 + 1));
                            }
                        }
                        if from/8 < 7 && from % 8 > 1 {
                            if !self.is_side_at(from + 1*8 - 2, self.side_to_move) {
                                res.push(Move::Standard(from, from + 1*8 - 2));
                            }
                        }
                        if from/8 < 7 && from % 8 < 6 {
                            if !self.is_side_at(from + 1*8 + 2, self.side_to_move) {
                                res.push(Move::Standard(from, from + 1*8 + 2));
                            }
                        }
                        
                    }, 
                    PieceKind::Pawn => {
                        
                        if self.side_to_move == Side::White {

                            // not promoting
                            if from/8 > 1 { 
                                // one step forward
                                if !self.is_piece_at(from - 8) {
                                    res.push(Move::Standard(from, from - 8));
                                }

                                // capture to the left
                                if from%8 > 0 && self.is_side_at(from - 9, self.side_to_move.opposite()){
                                    res.push(Move::Standard(from, from - 9));
                                }

                                // capture to the right
                                if from%8 < 7 && self.is_side_at(from - 7, self.side_to_move.opposite()){
                                    res.push(Move::Standard(from, from - 7));
                                }
                            }
                            
                            // promoting 
                            if from/8 == 1 {
                                if !self.is_piece_at(from - 8) {
                                    res.push(Move::Promotion(from, from - 8, PieceKind::Queen));
                                    res.push(Move::Promotion(from, from - 8, PieceKind::Rook));
                                    res.push(Move::Promotion(from, from - 8, PieceKind::Bishop));
                                    res.push(Move::Promotion(from, from - 8, PieceKind::Knight));
                                }

                                // capture to the left
                                if from%8 > 0 && self.is_side_at(from - 9, self.side_to_move.opposite()){
                                    res.push(Move::Promotion(from, from - 9, PieceKind::Queen));
                                    res.push(Move::Promotion(from, from - 9, PieceKind::Rook));
                                    res.push(Move::Promotion(from, from - 9, PieceKind::Bishop));
                                    res.push(Move::Promotion(from, from - 9, PieceKind::Knight));
                                }

                                // capture to the right
                                if from%8 < 7 && self.is_side_at(from - 7, self.side_to_move.opposite()){
                                    res.push(Move::Promotion(from, from - 7, PieceKind::Queen));
                                    res.push(Move::Promotion(from, from - 7, PieceKind::Rook));
                                    res.push(Move::Promotion(from, from - 7, PieceKind::Bishop));
                                    res.push(Move::Promotion(from, from - 7, PieceKind::Knight));
                                }

                            }
                        
                            if from/8 == 6 { // two moves from the starting square
                                if !self.is_piece_at(from - 8) && !self.is_piece_at(from - 16) {
                                    res.push(Move::Standard(from, from - 16));
                                }
                            } 

                            // en passant
                            if let Some((from,to)) = self.en_passant {
                                if to%8 > 0 && piece.square == to - 1 || to%8 < 7 && piece.square == to + 1 {
                                    res.push(Move::EnPassant(piece.square,(to + from)/2));
                                }
                            }

                        }
                        else {
                            // not promoting
                            if from/8 < 6 { 
                                if !self.is_piece_at(from + 8) {
                                    res.push(Move::Standard(from, from + 8));
                                }

                                // capture to the left
                                if from%8 > 0 && self.is_side_at(from + 7, self.side_to_move.opposite()){
                                    res.push(Move::Standard(from, from + 7));
                                }

                                // capture to the right
                                if from%8 < 7 && self.is_side_at(from + 9, self.side_to_move.opposite()){
                                    res.push(Move::Standard(from, from + 9));
                                }

                            }
                            
                            // promoting
                            if from/8 == 6 { 
                                if !self.is_piece_at(from + 8) {
                                    res.push(Move::Promotion(from, from + 8, PieceKind::Queen));
                                    res.push(Move::Promotion(from, from + 8, PieceKind::Rook));
                                    res.push(Move::Promotion(from, from + 8, PieceKind::Bishop));
                                    res.push(Move::Promotion(from, from + 8, PieceKind::Knight));
                                }

                                // capture to the left
                                if from%8 > 0 && self.is_side_at(from + 7, self.side_to_move.opposite()){
                                    res.push(Move::Promotion(from, from + 7, PieceKind::Queen));
                                    res.push(Move::Promotion(from, from + 7, PieceKind::Rook));
                                    res.push(Move::Promotion(from, from + 7, PieceKind::Bishop));
                                    res.push(Move::Promotion(from, from + 7, PieceKind::Knight));
                                }

                                // capture to the right
                                if from%8 < 7 && self.is_side_at(from + 9, self.side_to_move.opposite()){
                                    res.push(Move::Promotion(from, from + 9, PieceKind::Queen));
                                    res.push(Move::Promotion(from, from + 9, PieceKind::Rook));
                                    res.push(Move::Promotion(from, from + 9, PieceKind::Bishop));
                                    res.push(Move::Promotion(from, from + 9, PieceKind::Knight));
                                }
                            }
                            
                            // two moves from the starting square
                            if from/8 == 1 { 
                                if !self.is_piece_at(from + 8) && !self.is_piece_at(from + 16) {
                                    res.push(Move::Standard(from, from + 16));
                                }
                            } 

                            // en passant
                            if let Some((from,to)) = self.en_passant {
                                if to%8 > 0 && piece.square == to - 1 || to%8 < 7 && piece.square == to + 1 {
                                    res.push(Move::EnPassant(piece.square,(to + from)/2));
                                }
                            }

                        }
                    }, 
                }
            }
        }

        // only retain those moves that do not end up leaving the moving side in check
        res.retain(|chess_move| {
            let mut resulting_state = self.clone();
            resulting_state.apply_move_unchecked(chess_move);

            !resulting_state.is_in_check(self.side_to_move)
        });

        return res;

    }

    // returns of the king of the specified side is currently in check
    // this is mostly for legality checking for now
    pub fn is_in_check(&self, side: Side) -> bool {
        
        let mut king_position: i8 = -1;
        
        for piece in &self.pieces {
            if piece.side == side && piece.kind == PieceKind::King {
                king_position = piece.square;
                break;
            }
        }

        if king_position == -1 {
            return false;
        }
        else {
            return self.is_attacked(king_position, side.opposite());
        }
    }
        
    // returns true if the given square is attacked by the given side
    pub fn is_attacked(&self, square: i8, side: Side) -> bool {
        
        for piece in &self.pieces {
            if piece.side == side {
                match piece.kind {
                    PieceKind::King => {
                        if 
                            square == piece.square - 9 && piece.square%8 > 0 ||
                            square == piece.square - 8 ||
                            square == piece.square - 7 && piece.square%8 < 7 ||
                            square == piece.square + 1 && piece.square%8 < 7 ||
                            square == piece.square + 9 && piece.square%8 < 7 ||
                            square == piece.square + 8 ||
                            square == piece.square + 7 && piece.square%8 > 0 ||
                            square == piece.square - 1 && piece.square%8 > 0
                        {
                            return true;
                        }
                    },
                    PieceKind::Queen => {

                        // looking along the files and ranks until a collision
                        let directions: [i8; 4] = [-1,1,8,-8];

                        for d in directions {
                            let mut pos :i8 = piece.square;
                            loop { 
                                pos += d;

                                if (pos/8 != piece.square/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                    break;
                                }

                                if pos == square {
                                    return true;
                                }

                                if self.is_piece_at(pos) {
                                    break;
                                }

                            }
                        }

                        // check all diagonals
                        let directions: [i8; 4] = [-9,-7,7,9];

                        for d in directions {
                            let mut pos :i8 = piece.square;
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

                                if pos == square {
                                    return true;
                                }

                                if self.is_piece_at(pos) {
                                    break;
                                }

                            }
                        }

                    },
                    PieceKind::Rook => {

                        // looking along the files and ranks until a collision
                        let directions: [i8; 4] = [-1,1,8,-8];

                        for d in directions {
                            let mut pos :i8 = piece.square;
                            loop { 
                                pos += d;

                                if (pos/8 != piece.square/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                    break;
                                }

                                if pos == square {
                                    return true;
                                }

                                if self.is_piece_at(pos) {
                                    break;
                                }
                                
                            }
                        }
                    },
                    PieceKind::Bishop => {

                        // check all diagonals
                        let directions: [i8; 4] = [-9,-7,7,9];

                        for d in directions {
                            let mut pos :i8 = piece.square;
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

                                if pos == square {
                                    return true;
                                }

                                if self.is_piece_at(pos) {
                                    break;
                                }

                            }
                        }

                    },
                    PieceKind::Knight => {

                        // just check L shapes that are in bounds
                        if 
                            square == piece.square - 2*8 - 1 && piece.square/8 > 1 && piece.square%8 > 0 ||
                            square == piece.square - 2*8 + 1 && piece.square/8 > 1 && piece.square%8 < 7 ||
                            square == piece.square - 1*8 - 2 && piece.square/8 > 0 && piece.square%8 > 1 ||
                            square == piece.square - 1*8 + 2 && piece.square/8 > 0 && piece.square%8 < 6 ||
                            square == piece.square + 2*8 - 1 && piece.square/8 < 6 && piece.square%8 > 0 ||
                            square == piece.square + 2*8 + 1 && piece.square/8 < 6 && piece.square%8 < 7 ||
                            square == piece.square + 1*8 - 2 && piece.square/8 < 7 && piece.square%8 > 1 ||
                            square == piece.square + 1*8 + 2 && piece.square/8 < 7 && piece.square%8 < 6
                        {
                            return true;
                        }

                    },
                    PieceKind::Pawn => {

                        // there are only a couple of squares attacked by pawns
                        // this does not see en passant attacks for now

                        if 
                            piece.side == Side::White && square == piece.square - 9 && piece.square % 8 > 0 ||
                            piece.side == Side::White && square == piece.square - 7 && piece.square % 8 < 7 ||
                            piece.side == Side::Black && square == piece.square + 7 && piece.square % 8 > 0 ||
                            piece.side == Side::Black && square == piece.square + 9 && piece.square % 8 < 7
                        {
                            return true;
                        }

                    },
                }
            }
        }

        return false;
    }

}

// for converting coordinates into moves
pub fn coordinates_to_move(board_state: &BoardState, from: i8, to: i8) -> Move {
    
    // castling for black
    if 
        board_state.is_kind_at(from,PieceKind::King) &&
        board_state.is_side_at(from, Side::Black) &&
        ((from == 4 && to == 2) || (from == 4 && to == 6)) 
    {
        return Move::Castle(from,to);
    }
    
    // castling for white
    if 
        board_state.is_kind_at(from,PieceKind::King) &&
        board_state.is_side_at(from, Side::White) &&
        ((from == 60 && to == 58 || (from == 60 && to == 62))) 
    {
        return Move::Castle(from,to);
    }
    
    // capturing en_passant
    if let Some((f,t)) = board_state.en_passant {
        if to == (f + t)/2 && board_state.is_kind_at(from, PieceKind::Pawn) {
            return Move::EnPassant(from, (t + f)/2);
        }
        else {
            return Move::Standard(from,to);
        }
    }
    
    // promotions and pawn other moves
    if 
        board_state.is_kind_at(from, PieceKind::Pawn) &&
        ((from/8 == 1 && to/8 == 0) || (from/8 == 6 && to/8 == 7)) {
            return Move::Promotion(from, to, PieceKind::Queen); 
            // for now, the players can only promote to queens
        }
    else { // lastly, this was a standard move
        return Move::Standard(from,to);
    }
}

// methods for accessing piece information at various squares
impl BoardState { 

    pub fn is_piece_at(&self, square: i8) -> bool {
        if square < 0 || square > 63 {
            return false;
        }
        else {
            for p in &self.pieces {
                if p.square == square {
                    return true;
                }
            }
            return false;
        }
    }

    pub fn is_kind_at(&self, square: i8, kind: PieceKind) -> bool {
        for p in &self.pieces {
            if p.square == square && p.kind == kind {
                return true;
            }
        }
        return false;
    }

    pub fn is_side_at(&self, square: i8, side: Side) -> bool {
        for p in &self.pieces {
            if p.square == square && p.side == side {
                return true;
            }
        }
        return false;
    }

}

// methods for setting up the starting position of the board
impl BoardState { 

    pub fn to_starting_position(&mut self) {
        self.side_to_move = Side::White;
        self.pieces = self.starting_pieces();
        self.white_short_castle = true; 
        self.white_long_castle = true; 
        self.black_short_castle = true; 
        self.black_long_castle = true; 
        self.en_passant = None;
    }

    fn starting_pieces(&self) -> Vec<Piece> {
        let mut res = Vec::<Piece>::new();

        // black pieces
        res.push(Piece{side: Side::Black, kind: PieceKind::Rook, square: 0});
        res.push(Piece{side: Side::Black, kind: PieceKind::Knight, square: 1});
        res.push(Piece{side: Side::Black, kind: PieceKind::Bishop, square: 2});
        res.push(Piece{side: Side::Black, kind: PieceKind::Queen, square: 3});
        res.push(Piece{side: Side::Black, kind: PieceKind::King, square: 4});
        res.push(Piece{side: Side::Black, kind: PieceKind::Bishop, square: 5});
        res.push(Piece{side: Side::Black, kind: PieceKind::Knight, square: 6});
        res.push(Piece{side: Side::Black, kind: PieceKind::Rook, square: 7});
        
        // white pieces
        res.push(Piece{side: Side::White, kind: PieceKind::Rook, square: 56});
        res.push(Piece{side: Side::White, kind: PieceKind::Knight, square: 57});
        res.push(Piece{side: Side::White, kind: PieceKind::Bishop, square: 58});
        res.push(Piece{side: Side::White, kind: PieceKind::Queen, square: 59});
        res.push(Piece{side: Side::White, kind: PieceKind::King, square: 60});
        res.push(Piece{side: Side::White, kind: PieceKind::Bishop, square: 61});
        res.push(Piece{side: Side::White, kind: PieceKind::Knight, square: 62});
        res.push(Piece{side: Side::White, kind: PieceKind::Rook, square: 63});

        // pawns
        for i in 0..8 {
            res.push(Piece{side: Side::Black, kind: PieceKind::Pawn, square: 8 + i});
            res.push(Piece{side: Side::White, kind: PieceKind::Pawn, square: 48 + i});
        }

        return res;

    }

}

// the Move enum 
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Move {
    Standard(i8,i8),
    Castle(i8,i8),
    EnPassant(i8,i8),
    Promotion(i8,i8, PieceKind), // for now promotions only give queens, we will fix this later
}

impl Move {

    pub fn to_text(&self) -> String {
        match self {
            Move::Standard(from, to) => {
                return self.square_to_text(*from) + " -> " + &self.square_to_text(*to);
            },
            Move::Castle(from, to) => {
                return self.square_to_text(*from) + " -> " + &self.square_to_text(*to);
            },
            Move::EnPassant(from, to) => {
                return self.square_to_text(*from) + " -> " + &self.square_to_text(*to);
            },
            Move::Promotion(from, to,_) => {
                return self.square_to_text(*from) + " -> " + &self.square_to_text(*to);
            },
        }
    }

    pub fn square_to_text(&self, square: i8) -> String {
        let letters = ["a", "b", "c", "d", "e", "f", "g", "h"];
        return String::from(letters[(square%8) as usize]) + &(8 - square/8).to_string();
    }

    pub fn get_from(&self) -> i8 {
        match self {
            Move::Standard(from, _) => return *from,
            Move::Castle(from, _) => return *from,
            Move::EnPassant(from, _) => return *from,
            Move::Promotion(from, _,_) => return *from
        }
    }

    pub fn get_to(&self) -> i8 {
        match self {
            Move::Standard(_,to) => return *to,
            Move::Castle(_,to) => return *to,
            Move::EnPassant(_,to) => return *to,
            Move::Promotion(_,to,_) => return *to
        }
    }

}

// the Piece structure, for now rather simple
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Piece {
    pub side: Side,
    pub kind: PieceKind,
    pub square: i8,
}

impl Piece {
    // returns the numerical value of the piece
    pub fn get_value(&self) -> f64 {
        match self.kind {
            PieceKind::King => 0.,
            PieceKind::Queen => 9.,
            PieceKind::Rook => 5.,
            PieceKind::Bishop => 3.,
            PieceKind::Knight => 3.,
            PieceKind::Pawn => 1.,
        }
    }

}

// the different kinds of pieces in an Enum
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

// functions related to the Side enum

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Black,
    White,
}

impl Side {

    pub fn opposite(&self) -> Side {
        match self {
            Side::White => return Side::Black,
            Side::Black => return Side::White,
        }
    }
}