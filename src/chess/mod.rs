use std::f64::INFINITY;

mod evaluation;

// the board state in a game of chess [FIXED]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardState {
    pub piece_arr: [Option<Piece>;64],
    pub side_to_move: Side,
    pub white_short_castle: bool,
    pub white_long_castle: bool,
    pub black_short_castle: bool,
    pub black_long_castle: bool,
    pub en_passant: Option<(i8,i8)>,
}

// methods for making and getting legal moves on the board [FIXED]
impl BoardState { 

    // Applies a move without checking for legality. 
    // The caller must guarantee that a move is indeed legal before calling
    // Returns the information needed to retur to the previous board state
    // [FIXED]
    pub fn apply_move_unchecked(&mut self, m: &Move) -> UnMove {

        // before making the move, we record the current state

        let captured_piece: Option<Piece> = match m {
            Move::Standard(_, to)|Move::Promotion(_, to, _) => {
                if let Some(p) = self.piece_arr[*to as usize] {
                    Some(p.clone())
                }
                else {
                    None
                }
            },
            Move::Castle(_,_) => None,
            Move::EnPassant(from,to) => {
                self.piece_arr[((*from / 8) * 8 + *to % 8) as usize].clone()
            }
        };  

        let res = UnMove{
            movement: m.clone(),
            previous_side_to_move: self.side_to_move,
            capture: captured_piece,
            white_short_castle: self.white_short_castle,
            white_long_castle: self.white_long_castle,
            black_short_castle: self.black_short_castle,
            black_long_castle: self.black_long_castle,
            en_passant: self.en_passant,
        };
        
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
                
                // move the piece on "from" to "to", this removes the captured piece automatically
                self.piece_arr[*to as usize] = self.piece_arr[*from as usize];
                self.piece_arr[*from as usize] = None;

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

                // move the king
                self.piece_arr[*to as usize] = self.piece_arr[*from as usize];
                self.piece_arr[*from as usize] = None;
                
                // move the rook
                if *to > *from {
                    self.piece_arr[(*to - 1) as usize] = self.piece_arr[(*to + 1) as usize];
                    self.piece_arr[(*to + 1) as usize] = None;
                }
                else {
                    self.piece_arr[(*to + 1) as usize] = self.piece_arr[(*to - 2) as usize];
                    self.piece_arr[(*to - 2) as usize] = None;
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

                // move the pawn and capture any piece there in the process
                self.piece_arr[*to as usize] = self.piece_arr[*from as usize];
                self.piece_arr[*from as usize] = None;

                // promote the pawn
                self.piece_arr[*to as usize].as_mut().expect("There is no pawn to promote!").kind = *kind;
            },
            Move::EnPassant(from, to) => {

                // en passant never means a movement of the kings or rooks,
                // so this means that the castling rights are kept and need not be considered here

                // en passant is now not valid
                self.en_passant = None;

                // remove any captured pawn
                // note that en passant captures behind the moving pawn
                if self.side_to_move == Side::White {
                    self.piece_arr[(*to + 8) as usize] = None;
                }
                else {
                    self.piece_arr[(*to - 8) as usize] = None;
                }

                // move the pawn on "from" to "to"
                self.piece_arr[*to as usize] = self.piece_arr[*from as usize];
                self.piece_arr[*from as usize] = None; 
            }
        }

        self.side_to_move = self.side_to_move.opposite();

        // finally give the unmove
        return res;

    }

    // says if a given move is legal or not 
    // [FIXED, but we should not have this in the analysis at any point]
    pub fn is_legal_move(&mut self, m: &Move) -> bool {
        return self.get_legal_moves().contains(m);
    }

    // returns all the legal moves in the current BoardState [FIXED, but can be optimized quite a lot]
    pub fn get_legal_moves(&mut self) -> Vec<Move> {
        let mut res: Vec<Move> = Vec::<Move>::new();

        for i in 0..64 {
            if let Some(piece) = self.piece_arr[i] {
                if piece.side == self.side_to_move {
                
                    let sq = i as i8;
                    match piece.kind {
                        
                        PieceKind::King => {
                            // the moves on the immediate squares
                            if sq % 8 > 0 && sq/8 > 0 && !self.is_side_at(sq - 9, self.side_to_move) {
                                res.push(Move::Standard(sq,sq - 9));
                            }
                            if sq/8 > 0 && !self.is_side_at(sq - 8, self.side_to_move) {
                                res.push(Move::Standard(sq,sq - 8));
                            }
                            if sq % 8 < 7 && sq/8 > 0 && !self.is_side_at(sq - 7, self.side_to_move) {
                                res.push(Move::Standard(sq,sq - 7));
                            }
                            if sq % 8 < 7 && !self.is_side_at(sq + 1, self.side_to_move) {
                                res.push(Move::Standard(sq,sq + 1));
                            }
                            if sq % 8 < 7 && sq/8 < 7 && !self.is_side_at(sq + 9, self.side_to_move) {
                                res.push(Move::Standard(sq,sq + 9));
                            }
                            if sq/8 < 7 && !self.is_side_at(sq + 8, self.side_to_move) {
                                res.push(Move::Standard(sq,sq + 8));
                            }
                            if sq % 8 > 0 && sq/8 < 7 && !self.is_side_at(sq + 7, self.side_to_move) {
                                res.push(Move::Standard(sq,sq + 7));
                            }
                            if sq % 8 > 0 && !self.is_side_at(sq - 1, self.side_to_move) {
                                res.push(Move::Standard(sq,sq - 1));
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
                                let mut pos :i8 = sq;
                                loop { 
                                    pos += d;
                                    if (pos/8 != sq/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                        break;
                                    }
                                    else if self.is_piece_at(pos) {
                                        if !self.is_side_at(pos, self.side_to_move) {
                                            res.push(Move::Standard(sq, pos));
                                        }
                                        break;
                                    }
                                    res.push(Move::Standard(sq,pos));
                                }
                            }

                            // and also on diagonals until there is a collision
                            let directions: [i8; 4] = [-9,-7,7,9];

                            for d in directions {
                                let mut pos :i8 = sq;
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
                                            res.push(Move::Standard(sq, pos));
                                        }
                                        break;
                                    }
                                    res.push(Move::Standard(sq,pos));
                                }
                            }
                            
                        }, 
                        PieceKind::Rook => {
                            // this piece can move on files and ranks until there is a collision
                            let directions: [i8; 4] = [-1,1,8,-8];

                            for d in directions {
                                let mut pos :i8 = sq;
                                loop { 
                                    pos += d;
                                    if (pos/8 != sq/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                                        break;
                                    }
                                    else if self.is_piece_at(pos) {
                                        if !self.is_side_at(pos, self.side_to_move) {
                                            res.push(Move::Standard(sq, pos));
                                        }
                                        break;
                                    }
                                    res.push(Move::Standard(sq,pos));
                                }
                            }
                        }, 
                        PieceKind::Bishop => {
                            // this piece can move on diagonals until there is a collision
                            let directions: [i8; 4] = [-9,-7,7,9];

                            for d in directions {
                                let mut pos :i8 = sq;
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
                                            res.push(Move::Standard(sq, pos));
                                        }
                                        break;
                                    }
                                    res.push(Move::Standard(sq,pos));
                                }
                            }
                        }, 
                        PieceKind::Knight => {

                            // this piece can only move in L-shapes
                            if sq/8 > 1 && sq % 8 > 0 {
                                if !self.is_side_at(sq - 2*8 - 1, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq - 2*8 - 1));
                                }
                            }
                            if sq/8 > 1 && sq % 8 < 7 {
                                if !self.is_side_at(sq - 2*8 + 1, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq - 2*8 + 1));
                                }
                            }
                            if sq/8 > 0 && sq % 8 > 1 {
                                if !self.is_side_at(sq - 1*8 - 2, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq - 1*8 - 2));
                                }
                            }
                            if sq/8 > 0 && sq % 8 < 6 {
                                if !self.is_side_at(sq - 1*8 + 2, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq - 1*8 + 2));
                                }
                            }
                            if sq/8 < 6 && sq % 8 > 0 {
                                if !self.is_side_at(sq + 2*8 - 1, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq + 2*8 - 1));
                                }
                            }
                            if sq/8 < 6 && sq % 8 < 7 {
                                if !self.is_side_at(sq + 2*8 + 1, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq + 2*8 + 1));
                                }
                            }
                            if sq/8 < 7 && sq % 8 > 1 {
                                if !self.is_side_at(sq + 1*8 - 2, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq + 1*8 - 2));
                                }
                            }
                            if sq/8 < 7 && sq % 8 < 6 {
                                if !self.is_side_at(sq + 1*8 + 2, self.side_to_move) {
                                    res.push(Move::Standard(sq, sq + 1*8 + 2));
                                }
                            }
                            
                        }, 
                        PieceKind::Pawn => {
                            
                            if self.side_to_move == Side::White {

                                // not promoting
                                if sq/8 > 1 { 
                                    // one step forward
                                    if !self.is_piece_at(sq - 8) {
                                        res.push(Move::Standard(sq, sq - 8));
                                    }

                                    // capture to the left
                                    if sq%8 > 0 && self.is_side_at(sq - 9, self.side_to_move.opposite()){
                                        res.push(Move::Standard(sq, sq - 9));
                                    }

                                    // capture to the right
                                    if sq%8 < 7 && self.is_side_at(sq - 7, self.side_to_move.opposite()){
                                        res.push(Move::Standard(sq, sq - 7));
                                    }
                                }
                                
                                // promoting 
                                if sq/8 == 1 {
                                    if !self.is_piece_at(sq - 8) {
                                        res.push(Move::Promotion(sq, sq - 8, PieceKind::Queen));
                                        res.push(Move::Promotion(sq, sq - 8, PieceKind::Rook));
                                        res.push(Move::Promotion(sq, sq - 8, PieceKind::Bishop));
                                        res.push(Move::Promotion(sq, sq - 8, PieceKind::Knight));
                                    }

                                    // capture to the left
                                    if sq%8 > 0 && self.is_side_at(sq - 9, self.side_to_move.opposite()){
                                        res.push(Move::Promotion(sq, sq - 9, PieceKind::Queen));
                                        res.push(Move::Promotion(sq, sq - 9, PieceKind::Rook));
                                        res.push(Move::Promotion(sq, sq - 9, PieceKind::Bishop));
                                        res.push(Move::Promotion(sq, sq - 9, PieceKind::Knight));
                                    }

                                    // capture to the right
                                    if sq%8 < 7 && self.is_side_at(sq - 7, self.side_to_move.opposite()){
                                        res.push(Move::Promotion(sq, sq - 7, PieceKind::Queen));
                                        res.push(Move::Promotion(sq, sq - 7, PieceKind::Rook));
                                        res.push(Move::Promotion(sq, sq - 7, PieceKind::Bishop));
                                        res.push(Move::Promotion(sq, sq - 7, PieceKind::Knight));
                                    }

                                }
                            
                                if sq/8 == 6 { // two moves sq the starting square
                                    if !self.is_piece_at(sq - 8) && !self.is_piece_at(sq - 16) {
                                        res.push(Move::Standard(sq, sq - 16));
                                    }
                                } 

                                // en passant
                                if let Some((from,to)) = self.en_passant {
                                    if to%8 > 0 && sq == to - 1 || to%8 < 7 && sq == to + 1 {
                                        res.push(Move::EnPassant(sq,(to + from)/2));
                                    }
                                }

                            }
                            else {
                                // not promoting
                                if sq/8 < 6 { 
                                    if !self.is_piece_at(sq + 8) {
                                        res.push(Move::Standard(sq, sq + 8));
                                    }

                                    // capture to the left
                                    if sq%8 > 0 && self.is_side_at(sq + 7, self.side_to_move.opposite()){
                                        res.push(Move::Standard(sq, sq + 7));
                                    }

                                    // capture to the right
                                    if sq%8 < 7 && self.is_side_at(sq + 9, self.side_to_move.opposite()){
                                        res.push(Move::Standard(sq, sq + 9));
                                    }

                                }
                                
                                // promoting
                                if sq/8 == 6 { 
                                    if !self.is_piece_at(sq + 8) {
                                        res.push(Move::Promotion(sq, sq + 8, PieceKind::Queen));
                                        res.push(Move::Promotion(sq, sq + 8, PieceKind::Rook));
                                        res.push(Move::Promotion(sq, sq + 8, PieceKind::Bishop));
                                        res.push(Move::Promotion(sq, sq + 8, PieceKind::Knight));
                                    }

                                    // capture to the left
                                    if sq%8 > 0 && self.is_side_at(sq + 7, self.side_to_move.opposite()){
                                        res.push(Move::Promotion(sq, sq + 7, PieceKind::Queen));
                                        res.push(Move::Promotion(sq, sq + 7, PieceKind::Rook));
                                        res.push(Move::Promotion(sq, sq + 7, PieceKind::Bishop));
                                        res.push(Move::Promotion(sq, sq + 7, PieceKind::Knight));
                                    }

                                    // capture to the right
                                    if sq%8 < 7 && self.is_side_at(sq + 9, self.side_to_move.opposite()){
                                        res.push(Move::Promotion(sq, sq + 9, PieceKind::Queen));
                                        res.push(Move::Promotion(sq, sq + 9, PieceKind::Rook));
                                        res.push(Move::Promotion(sq, sq + 9, PieceKind::Bishop));
                                        res.push(Move::Promotion(sq, sq + 9, PieceKind::Knight));
                                    }
                                }
                                
                                // two moves from the starting square
                                if sq/8 == 1 { 
                                    if !self.is_piece_at(sq + 8) && !self.is_piece_at(sq + 16) {
                                        res.push(Move::Standard(sq, sq + 16));
                                    }
                                } 

                                // en passant
                                if let Some((from,to)) = self.en_passant {
                                    if to%8 > 0 && sq == to - 1 || to%8 < 7 && sq == to + 1 {
                                        res.push(Move::EnPassant(sq,(to + from)/2));
                                    }
                                }

                            }
                        }, 
                    }
                }
            }
        }

        // only retain those moves that do not end up leaving the moving side in check
        res.retain(|chess_move| {

            let un_move = self.apply_move_unchecked(chess_move);
            let is_illegal = self.is_in_check(self.side_to_move.opposite());
            let success = self.un_move_unchecked(un_move);
            debug_assert!(success, "Failed to unmake {chess_move:?}");

            !is_illegal
        });

        return res;

    }

    // returns of the king of the specified side is currently in check
    // this is mostly for legality checking for now [FIXED]
    pub fn is_in_check(&self, side: Side) -> bool {
        
        let king_position = self.piece_arr
                                    .iter()
                                    .position(|piece| {
                                        piece.is_some_and(|p| p.side == side && p.kind == PieceKind::King)
                                    })
                                    .expect("Board state has no king for the requested side") as i8;

        self.is_attacked(king_position, side.opposite())
    }
        
    // returns true if the given square is attacked by the given side [FIXED]
    pub fn is_attacked(&self, square: i8, side: Side) -> bool {

        // first we look for an attack in the direction of a rook
        let directions: [i8; 4] = [-1,1,8,-8];
        for d in directions {
            let mut pos: i8 = square;
            loop { 
                pos += d;
                if (pos/8 != square/8 && d.abs() == 1) || (pos < 0 || pos > 63) {
                    break;
                }
                else if self.is_piece_at(pos) {
                    if self.is_side_at(pos,side) && (self.is_kind_at(pos, PieceKind::Rook) || self.is_kind_at(pos, PieceKind::Queen)) {
                        return true;
                    }
                    break;
                }
            }
        }

        // then in the directions of a bishop
        let directions: [i8; 4] = [-9,-7,7,9];

        for d in directions {
            let mut pos: i8 = square;
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
                    if self.is_side_at(pos,side) && (self.is_kind_at(pos, PieceKind::Bishop) || self.is_kind_at(pos, PieceKind::Queen)){
                        return true;
                    }
                    break;
                }
            }
        }

        // then in the possible squares for a knight
        if square/8 > 1 && square % 8 > 0 {
            let pos = square - 2*8 - 1;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            } 
        }
        if square/8 > 1 && square % 8 < 7 {
            let pos = square - 2*8 + 1;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }
        if square/8 > 0 && square % 8 > 1 {
            let pos = square - 1*8 - 2;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }
        if square/8 > 0 && square % 8 < 6 {
            let pos = square - 1*8 + 2;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }
        if square/8 < 6 && square % 8 > 0 {
            let pos = square + 2*8 - 1;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }
        if square/8 < 6 && square % 8 < 7 {
            let pos = square + 2*8 + 1;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }
        if square/8 < 7 && square % 8 > 1 {
            let pos = square + 1*8 - 2;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }
        if square/8 < 7 && square % 8 < 6 {
            let pos = square + 1*8 + 2;
            if self.is_kind_at(pos, PieceKind::Knight) && self.is_side_at(pos, side) {
                return true;
            }
        }

        // possible attacks by the king
        if square % 8 > 0 && square/8 > 0 && self.is_side_at(square - 9, side) && self.is_kind_at(square - 9, PieceKind::King) {
            return true;
        }
        if square/8 > 0 && self.is_side_at(square - 8, side) && self.is_kind_at(square - 8, PieceKind::King) {
            return true;
        }
        if square % 8 < 7 && square/8 > 0 && self.is_side_at(square - 7, side) && self.is_kind_at(square - 7, PieceKind::King) {
            return true;
        }
        if square % 8 < 7 && self.is_side_at(square + 1, side) && self.is_kind_at(square + 1, PieceKind::King) {
            return true;
        }
        if square % 8 < 7 && square/8 < 7 && self.is_side_at(square + 9, side) && self.is_kind_at(square + 9, PieceKind::King) {
            return true;
        }
        if square/8 < 7 && self.is_side_at(square + 8, side) && self.is_kind_at(square + 8, PieceKind::King) {
            return true;
        }
        if square % 8 > 0 && square/8 < 7 && self.is_side_at(square + 7, side) && self.is_kind_at(square + 7, PieceKind::King){
            return true;
        }
        if square % 8 > 0 && self.is_side_at(square - 1, side) && self.is_kind_at(square - 1, PieceKind::King) {
            return true;
        }

        // possible attacks by pawns
        match side {
            Side::Black => {
                if square % 8 > 0 && square/8 > 0 && self.is_kind_at(square - 9, PieceKind::Pawn) && self.is_side_at(square - 9, Side::Black) {
                    return true;
                }
                if square % 8 < 7 && square/8 > 0 && self.is_kind_at(square - 7, PieceKind::Pawn) && self.is_side_at(square - 7, Side::Black) {
                    return true;
                }
            },
            Side::White => {
                if square % 8 > 0 && square/8 < 7 && self.is_kind_at(square + 7, PieceKind::Pawn) && self.is_side_at(square + 7, Side::White) {
                    return true;
                }
                if square % 8 < 7 && square/8 < 7 && self.is_kind_at(square + 9, PieceKind::Pawn) && self.is_side_at(square + 9, Side::White) {
                    return true;
                }
            },
        }

        // This is reached if there is no attack
        return false;
        
        /*
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
        */

        //return false;
    }

    // returns a priority value for a move, saying how urgently 
    // one should look at it in the analysis [FIXED]
    pub fn priority(&self, m: &Move) -> f64 {

        let mut res: f64 = 0.;

        // if a capture, this can only be of an opposing colour anyway
        if let Some(kind) = self.kind_at(m.get_to()) {
            res += 10. * match kind {
                PieceKind::King => INFINITY,
                PieceKind::Queen => 9.,
                PieceKind::Rook => 5.,
                PieceKind::Bishop => 3.,
                PieceKind::Knight => 3.,
                PieceKind::Pawn => 1.,
            };
        }

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

        return res;
    }

    // this unmakes a move, given an un-move
    // returns true if successful [FIXED]
    pub fn un_move_unchecked(&mut self, um: UnMove) -> bool {

        // implement the un-move on piece placement, 
        // with controls for consistency

        if um.previous_side_to_move.opposite() == self.side_to_move {

            match um.movement {
                Move::Standard(from,to) => {

                    if let (None, Some(p)) = (&self.piece_arr[from as usize], &self.piece_arr[to as usize]) {
                        if p.side == self.side_to_move.opposite() {
                            // if all is well, move the piece back
                            self.piece_arr[from as usize] = Some(p).cloned();

                            // restore any captured piece if there was one
                            if let Some(captured_piece) = um.capture {
                                self.piece_arr[to as usize] = Some(captured_piece);
                            }
                            else {
                                self.piece_arr[to as usize] = None;
                            }
                        }
                        else {
                            return false;
                        }
                    }
                    else {
                        return false;
                    }
                },
                Move::Castle(from,to) => {

                    // a castle cannot capture:
                    if let Some(_) = um.capture {
                        return false;
                    }

                    // check for a king and rook in place
                    //let king_index: Option<usize> = self.pieces.iter().position(|piece| piece.square == to);
                    //let rook_index: Option<usize> = self.pieces.iter().position(|piece| piece.square == (to + from)/2);

                    if let (Some(k), Some(r)) = (self.piece_arr[to as usize], self.piece_arr[((from + to)/2) as usize]) {
                        if 
                            k.kind == PieceKind::King && 
                            k.side == self.side_to_move.opposite() && 
                            !self.is_piece_at(from) &&
                            r.kind == PieceKind::Rook && 
                            r.side == self.side_to_move.opposite() && 
                            !self.is_piece_at(to + (to - from).signum()*(1 + (7 - to%8)/4))
                        {
                            // move the rook and the king
                            self.piece_arr[from as usize] = Some(k.clone());
                            self.piece_arr[(to + (to - from).signum()*(1 + (7 - to%8)/4)) as usize] = Some(r.clone());

                            // reset the squares where they were
                            self.piece_arr[to as usize] = None;
                            self.piece_arr[(to - (to - from).signum()) as usize] = None;
                        }
                        else {
                            return false;
                        }
                    }
                    else {
                        return false;
                    }
                },
                Move::EnPassant(from,to) => {

                    if let Some(captured_piece) = um.capture {
                        // an en passant must always capture a pawn
                        if captured_piece.kind == PieceKind::Pawn {
                            // control for correct pieces in the correct squares
                            let capture_square = (from / 8) * 8 + to % 8;

                            if let (Some(p), None, None) = (self.piece_arr[to as usize], self.piece_arr[capture_square as usize], self.piece_arr[from as usize]) {
                                if p.kind == PieceKind::Pawn {
                                    // if all is well, move one pawn back
                                    self.piece_arr[from as usize] = Some(p.clone());
                                    self.piece_arr[to as usize] = None;

                                    // and add the captured pawn again
                                    self.piece_arr[capture_square as usize] = Some(captured_piece);
                                }
                                else {
                                    return false;
                                }
                            }
                            else {
                                return false;
                            }
                        }
                        else {
                            return false;
                        }
                    }
                    else {
                        return false;
                    }
                },
                Move::Promotion(from,to, kind) => {
                    // a promotion must always happen on the final ranks
                    if to/8 == 7 || to/8 == 0 {
                        // control for the correct pieces in the correct squares
                        //let pawn_index: Option<usize> = self.pieces.iter().position(|piece| piece.square == to);
                        //let empty_index: Option<usize> = self.pieces.iter().position(|piece| piece.square == from);

                        if let (Some(mut p), None) = (self.piece_arr[to as usize], self.piece_arr[from as usize]) {
                            if p.kind == kind {
                                // if all is in place, then we un-promote
                                p.kind = PieceKind::Pawn;
                                // we move the pawn back
                                self.piece_arr[from as usize] = Some(p.clone());

                                // and we give back any captured piece
                                self.piece_arr[to as usize] = if let Some(captured_piece) = um.capture {
                                        Some(captured_piece)
                                    }
                                    else {
                                        None
                                    };
                            }
                            else {
                                return false;
                            }
                        }
                        else {
                            return false;
                        }
                    }
                    else {
                        return false;
                    }
                },
            }

            // recall the surrounding information
            self.side_to_move = um.previous_side_to_move;
            self.white_short_castle = um.white_short_castle;
            self.white_long_castle = um.white_long_castle;
            self.black_short_castle = um.black_short_castle;
            self.black_long_castle = um.black_long_castle;
            self.en_passant = um.en_passant;

            return true;
        }
        else {
            return false;
        }
    }
}

// for converting coordinates into moves [FIXED]
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
    if let Some((previous_from, previous_to)) = board_state.en_passant {
        let en_passant_destination = (previous_from + previous_to) / 2;

        if to == en_passant_destination
            && board_state.is_kind_at(from, PieceKind::Pawn)
            && !board_state.is_piece_at(to)
            && from / 8 == previous_to / 8
            && (from - previous_to).abs() == 1
        {
            return Move::EnPassant(from, to);
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

// methods for accessing piece information at various squares [FIXED]
impl BoardState { 

    pub fn is_piece_at(&self, square: i8) -> bool {
        if square < 0 || square > 63 {
            return false;
        }
        else {
            if let Some(_) = self.piece_arr[square as usize] {
                return true;
            }
            return false;
        }
    }

    pub fn kind_at(&self, square: i8) -> Option<PieceKind> {

        if let Some(p) = &self.piece_arr[square as usize] {
            return Some(p.kind);
        }

        return None;
    }

    pub fn is_kind_at(&self, square: i8, kind: PieceKind) -> bool {
        if let Some(p) = &self.piece_arr[square as usize] {
            if p.kind == kind {
                return true;
            }
        }
        return false;
    }

    pub fn is_side_at(&self, square: i8, side: Side) -> bool {
        if let Some(p) = &self.piece_arr[square as usize] {
            if p.side == side {
                return true;
            }
        }
        return false;
    }

}

// methods for setting up the starting position of the board [FIXED]
impl BoardState { 

    pub fn to_starting_position(&mut self) {
        self.side_to_move = Side::White;
        self.piece_arr = self.starting_pieces();
        self.white_short_castle = true; 
        self.white_long_castle = true; 
        self.black_short_castle = true; 
        self.black_long_castle = true; 
        self.en_passant = None;
    }

    fn starting_pieces(&self) -> [Option<Piece>;64] {
        let mut res = [None; 64];

        // black pieces
        res[0] = Some(Piece{side: Side::Black, kind: PieceKind::Rook});
        res[1] = Some(Piece{side: Side::Black, kind: PieceKind::Knight});
        res[2] = Some(Piece{side: Side::Black, kind: PieceKind::Bishop});
        res[3] = Some(Piece{side: Side::Black, kind: PieceKind::Queen});
        res[4] = Some(Piece{side: Side::Black, kind: PieceKind::King});
        res[5] = Some(Piece{side: Side::Black, kind: PieceKind::Bishop});
        res[6] = Some(Piece{side: Side::Black, kind: PieceKind::Knight});
        res[7] = Some(Piece{side: Side::Black, kind: PieceKind::Rook});
        
        // white pieces
        res[56] = Some(Piece{side: Side::White, kind: PieceKind::Rook});
        res[57] = Some(Piece{side: Side::White, kind: PieceKind::Knight});
        res[58] = Some(Piece{side: Side::White, kind: PieceKind::Bishop});
        res[59] = Some(Piece{side: Side::White, kind: PieceKind::Queen});
        res[60] = Some(Piece{side: Side::White, kind: PieceKind::King});
        res[61] = Some(Piece{side: Side::White, kind: PieceKind::Bishop});
        res[62] = Some(Piece{side: Side::White, kind: PieceKind::Knight});
        res[63] = Some(Piece{side: Side::White, kind: PieceKind::Rook});

        // pawns
        for i in 0..8 {
            res[8 + i] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
            res[48 + i] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
        }

        return res;

    }

}

// the Move enum [FIXED]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Move { 
    Standard(i8,i8),// this contains from, to, and then optional data
    Castle(i8,i8),
    EnPassant(i8,i8),
    Promotion(i8,i8, PieceKind), // for now promotions only give queens, we will fix this later
}

// functions relevant for Move [FIXED]
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

// saves the data that is necessary to revert to the previous state [FIXED]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnMove {
    pub movement: Move,
    pub previous_side_to_move: Side,
    pub capture: Option<Piece>,
    pub white_short_castle: bool,
    pub white_long_castle: bool,
    pub black_short_castle: bool,
    pub black_long_castle: bool,
    pub en_passant: Option<(i8,i8)>,
}

// the Piece structure, for now rather simple [FIXED]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Piece {
    pub side: Side,
    pub kind: PieceKind,
}


// for now just a piece value function [FIXED]
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

// the different kinds of pieces in an Enum [FIXED]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

// functions related to the Side enum [FIXED]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Black,
    White,
}

// just an opposite function [FIXED]
impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::White => return Side::Black,
            Side::Black => return Side::White,
        }
    }
}