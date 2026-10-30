const MAX_MOVES: usize = 230;
 
use arrayvec::ArrayVec;

pub type MoveList = ArrayVec<Move, MAX_MOVES>;

mod evaluation;

// the board state in a game of chess [FIXED]
#[derive(Clone, Debug)]
pub struct BoardState {
    pub piece_arr: [Option<Piece>;64],
    pub side_to_move: Side,
    pub white_short_castle: bool,
    pub white_long_castle: bool,
    pub black_short_castle: bool,
    pub black_long_castle: bool,
    pub en_passant: Option<(i8,i8)>,
    pub white_king: i8,
    pub black_king: i8,
    pub material: f64,
    pub total_pieces: u8,
    pub half_moves: usize,
    pub full_moves: usize,
}

// methods for making and getting legal moves on the board [FIXED]
impl BoardState { 
 
    // Applies a move without checking for legality. 
    // The caller must guarantee that a move is indeed legal before calling
    // Returns the information needed to retur to the previous board state
    pub fn apply_move_unchecked(&mut self, m: &Move) -> UnMove {

        // before making the move, we record the current state
        let captured_piece: Option<Piece> = match m {
            Move::Standard(_, to)|Move::Promotion(_, to, _) => {
                if let Some(p) = self.piece_arr[*to as usize] {
                    Some(p)
                }
                else {
                    None
                }
            },
            Move::Castle(_,_) => None,
            Move::EnPassant(from,to) => {
                self.piece_arr[((*from / 8) * 8 + *to % 8) as usize]
            }
        };

        let res = UnMove{
            movement: *m,
            previous_side_to_move: self.side_to_move,
            capture: captured_piece,
            white_short_castle: self.white_short_castle,
            white_long_castle: self.white_long_castle,
            black_short_castle: self.black_short_castle,
            black_long_castle: self.black_long_castle,
            en_passant: self.en_passant,
            white_king: self.white_king,
            black_king: self.black_king,
            material: self.material,
            total_pieces: self.total_pieces,
            half_moves: self.half_moves,
            full_moves: self.full_moves,
        };

        // keep track of the captured piece in the board data
        if let Some(piece) = captured_piece {
            self.total_pieces -= 1;
            self.half_moves = 0;

            self.material += match piece.side {
                Side::White => -piece.get_value(),
                Side::Black => piece.get_value()
            };
        }  

        // reset halfmoves in case of a pawn move
        if let Some(piece) = self.piece_arr[m.get_from() as usize] {
            if piece.kind == PieceKind::Pawn {
                self.half_moves = 0;
            }
        }
        
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

                // handle the king positions if one of them moved
                if let Some(piece) = self.piece_arr[*from as usize] {
                    if piece.kind == PieceKind::King {
                        match piece.side {
                            Side::White => {
                                self.white_king = *to;
                            },
                            Side::Black => {
                                self.black_king = *to;
                            },
                        }
                    }
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

                // move the king in the board state
                if let Some(piece) = self.piece_arr[*from as usize] {
                    if piece.kind == PieceKind::King {
                        match piece.side {
                            Side::White => {
                                self.white_king = *to;
                            },
                            Side::Black => {
                                self.black_king = *to;
                            },
                        }
                    }
                }
                // move the king in the square array
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

                let pawn = self.piece_arr[*from as usize].expect("There is no pawn to promote");

                let promoted_value = match kind {
                    PieceKind::Queen => 9.0,
                    PieceKind::Rook => 5.0,
                    PieceKind::Bishop => 3.0,
                    PieceKind::Knight => 3.0,
                    _ => panic!("Invalid promotion piece"),
                };

                let promotion_gain = promoted_value - pawn.get_value();

                self.material += match pawn.side {
                    Side::White => promotion_gain,
                    Side::Black => -promotion_gain,
                };

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

        // keep track of the full moves
        if self.side_to_move == Side::Black {
            self.full_moves += 1;
        }

        self.side_to_move = self.side_to_move.opposite();

        // finally give the unmove
        return res;

    }

    // says if a given move is legal or not 
    // [FIXED, but we should not have this in the analysis at any point]
    pub fn is_legal_move(&mut self, m: &Move) -> bool {
        return self.get_legal_moves().contains(&m);
    }

    // returns all the legal moves in the current BoardState [FIXED, but can be optimized quite a lot]
    pub fn get_legal_moves(&mut self) -> MoveList {
        
        let mut res = MoveList::new();
        for i in 0..64 {
            if let Some(piece) = self.piece_arr[i] {
                if piece.side == self.side_to_move {
                
                    let sq = i as i8;
                    let side = piece.side;
                    match piece.kind {
                        
                        PieceKind::King => {

                            // the moves on the immediate squares
                            let directions: [(i8,i8);8] = [(-1,-1), (-1,0), (-1,1), (0,1), (1,1), (1,0), (1,-1), (0,-1)];
                            for d in directions {
                                if let Some(destination) = self.stays_in_bounds(sq, d) {
                                    if !self.is_side_at(destination, side) {
                                        let m = Move::Standard(sq,destination);
                                        if !self.results_in_check(m) {
                                            res.push(m);
                                        }
                                    }
                                }
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
                                    // being in check is controlled above
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
                                    // being in check is controlled above
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
                                    res.push( Move::Castle(4,6));
                                    // being in check is controlled above
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
                                    res.push( Move::Castle(4,2));
                                    // being in check is controlled above
                                }
                            }
                        }, 
                        PieceKind::Queen => {

                            // the queen can slide in any of 8 directions
                            let directions: [(i8,i8);8] = [(0,-1),(0,1),(-1,0),(1,0), (-1,-1),(-1,1),(1,-1),(1,1)];
                            for d in directions {
                                let mut pos: i8 = sq;
                                loop {
                                    if let Some(destination) = self.stays_in_bounds(pos, d) {
                                        if self.is_piece_at(destination) {
                                            if !self.is_side_at(destination, side) {
                                                let m = Move::Standard(sq, destination);
                                                if !self.results_in_check(m) {
                                                    res.push(m);
                                                }
                                            }
                                            break;
                                        }
                                        let m = Move::Standard(sq,destination);
                                        if !self.results_in_check(m) {
                                            res.push(m);
                                        }

                                        pos = destination;
                                    }
                                    else {
                                        break;
                                    }
                                }
                            }
                        }, 
                        PieceKind::Rook |PieceKind::Bishop => {
                            
                            // the rook and bishop can slide in any of 4 directions
                            let directions: [(i8,i8);4] = if piece.kind == PieceKind::Rook {
                                [(0,-1),(0,1),(-1,0),(1,0)]
                            }
                            else {
                                [(-1,-1),(-1,1),(1,-1),(1,1)]
                            };
                            
                            for d in directions {
                                let mut pos: i8 = sq;
                                loop {
                                    if let Some(destination) = self.stays_in_bounds(pos, d) {
                                        if self.is_piece_at(destination) {
                                            if !self.is_side_at(destination, side) {
                                                let m = Move::Standard(sq, destination);
                                                if !self.results_in_check(m) {
                                                    res.push(m);
                                                }
                                            }
                                            break;
                                        }
                                        let m = Move::Standard(sq,destination);
                                        if !self.results_in_check(m) {
                                            res.push(m);
                                        }

                                        pos = destination;
                                    }
                                    else {
                                        break;
                                    }
                                }
                            }
                        }, 
                        PieceKind::Knight => {

                            // this piece can only move in L-shapes
                            let directions: [(i8,i8);8] = [(-2,-1), (-2,1), (-1,-2), (-1,2), (2,-1), (2,1), (1,-2), (1,2)];
                            for d in directions {
                                if let Some(destination) = self.stays_in_bounds(sq, d) {
                                    if !self.is_side_at(destination, side) {
                                        let m = Move::Standard(sq,destination);
                                        if !self.results_in_check(m) {
                                            res.push(m);
                                        }
                                    }
                                }
                            }
                        }, 
                        PieceKind::Pawn => {

                            // first we find the direction in which a pawn moves
                            let dir: i8 = match side {
                                Side::White => -1,
                                Side::Black => 1
                            };

                            // thus the possible (non-enpassant) displacements are
                            let displacements = [(dir,-1), (dir,0), (dir,1), (2*dir,0)];                                
                            // now we have to handle the various cases
                            for d in displacements {
                                if let Some(destination) = self.stays_in_bounds(sq, d) {
    
                                    // moving one step forward
                                    if d == (dir,0) {
                                        if !self.is_piece_at(destination) {
                                            match destination/8 {
                                                0|7 => {
                                                    for kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                                                        let m = Move::Promotion(sq,destination,kind);
                                                        if !self.results_in_check(m) {
                                                            res.push(m);
                                                        }
                                                    }
                                                },
                                                _ => {
                                                    let m = Move::Standard(sq,destination);
                                                    if !self.results_in_check(m) {
                                                        res.push(m);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // moving two steps forward
                                    if d == (2*dir,0) {
                                        if (sq/8 == 1 && side == Side::Black ||sq/8 == 6 && side == Side::White) && !self.is_piece_at(sq + 8*dir) && !self.is_piece_at(sq + 16*dir) {
                                            let m = Move::Standard(sq,destination);
                                            if !self.results_in_check(m) {
                                                res.push(m);
                                            }
                                        }
                                    }

                                    // capturing a piece one step forward
                                    if d == (dir,1) ||d == (dir,-1) {
                                        if self.is_side_at(destination,side.opposite()) {
                                            match destination/8 {
                                                0|7 => {
                                                    for kind in [PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                                                        let m = Move::Promotion(sq,destination,kind);
                                                        if !self.results_in_check(m) {
                                                            res.push(m);
                                                        }
                                                    }
                                                },
                                                _ => {
                                                    let m = Move::Standard(sq,destination);
                                                    if !self.results_in_check(m) {
                                                        res.push(m);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // en passant
                            if let Some((from,to)) = self.en_passant {
                                if to%8 > 0 && sq == to - 1 || to%8 < 7 && sq == to + 1 {
                                    let m = Move::EnPassant(sq,(to + from)/2);
                                    if !self.results_in_check(m) {
                                        res.push(m);
                                    }
                                }
                            }
                        }, 
                    }
                }
            }
        }
        //println!("True legal moves were: {}\n the new legal moves were: {}", res.len(), res_temp.len());
        
        return res;
    }

    pub fn results_in_check(&mut self, chess_move: Move) -> bool {

        let un_move = self.apply_move_unchecked(&chess_move);
        let is_illegal = self.is_in_check(self.side_to_move.opposite());
        let success = self.un_move_unchecked(un_move);
        assert!(success, "Failed to unmake {chess_move:?}");
        //self.assert_cached_state();

        return is_illegal;

    }

    // returns of the king of the specified side is currently in check
    // this is mostly for legality checking for now [FIXED]
    pub fn is_in_check(&self, side: Side) -> bool {

        let king_position: i8 = match side {
            Side::White => self.white_king,
            Side::Black => self.black_king,
        };

        self.is_attacked(king_position, side.opposite())
    }
        
    // returns true if the given square is attacked by the given side [OPTIMIZATIONS FOR LOOKUPS]
    pub fn is_attacked(&self, square: i8, side: Side) -> bool {

        // first we look for an attack in the direction of a rook
        let directions: [(i8,i8); 4] = [(-1,0),(1,0),(0,1),(0,-1)];
        for d in directions {
            let mut pos: i8 = square;
            loop {
                if let Some(destination) = self.stays_in_bounds(pos, d) {
                    if self.is_piece_at(destination) {
                        if self.is_side_at(destination, side) && (self.is_kind_at(destination, PieceKind::Queen) ||self.is_kind_at(destination, PieceKind::Rook)) {
                            return true;
                        }
                        break;
                    }
                    pos = destination;
                }
                else {
                    break;
                }
            }
        }

        // then in the directions of a bishop
        let directions: [(i8,i8); 4] = [(-1,-1),(-1,1),(1,-1),(1,1)];
        for d in directions {
            let mut pos: i8 = square;
            loop {
                if let Some(destination) = self.stays_in_bounds(pos, d) {
                    if self.is_piece_at(destination) {
                        if self.is_side_at(destination, side) && (self.is_kind_at(destination, PieceKind::Queen) ||self.is_kind_at(destination, PieceKind::Bishop)) {
                            return true;
                        }
                        break;
                    }
                    pos = destination;
                }
                else {
                    break;
                }
            }
        }

        // then in the possible squares for a knight
        let directions: [(i8,i8);8] = [(1,2), (1,-2), (2,1), (2,-1), (-1,2), (-1,-2), (-2,1), (-2,-1)];
        for d in directions {
            if let Some(destination) = self.stays_in_bounds(square, d) {
                // now we are in bounds
                if let Some(piece) = self.piece_arr[destination as usize] {
                    if piece.side == side && piece.kind == PieceKind::Knight {
                        return true;
                    }
                }
            }   
        }

        // then possible attacks for a king
        let directions: [(i8, i8); 8] = [(-1,-1), (-1, 0), (-1,1), (0,1), (1,1), (1,0), (1,-1), (0,-1)];
        for d in directions {
            if let Some(destination) = self.stays_in_bounds(square, d) {
                // now we are in bounds
                if let Some(piece) = self.piece_arr[destination as usize] {
                    if piece.side == side && piece.kind == PieceKind::King {
                        return true;
                    }
                }
            }
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
    }

    // this unmakes a move, given an un-move
    // returns true if successful [FIXED]
    pub fn un_move_unchecked(&mut self, um: UnMove) -> bool {

        // implement the un-move on piece placement, 

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
                            println!("Un move failed at standard move: {}", um.movement.to_text());
                            return false;
                        }
                    }
                    else {
                        println!("Un move failed at standard move (occupations error): {}", um.movement.to_text());
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
                        println!("Un move failed at standard move: {}", um.movement.to_text());
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
                                    println!("Un move failed at en-passant (moved piece is not pawn): {}", um.movement.to_text());
                                    return false;
                                }
                            }
                            else {
                                println!("Un move failed at en-passant (moving piece is not pawn or capture square is not empty): {}", um.movement.to_text());
                                return false;
                            }
                        }
                        else {
                            println!("Un move failed at en-passant (capture is not pawn): {}", um.movement.to_text());
                            return false;
                        }
                    }
                    else {
                        println!("Un move failed at en-passant (no capture): {}", um.movement.to_text());
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
                                self.piece_arr[from as usize] = Some(p);

                                // and we give back any captured piece
                                self.piece_arr[to as usize] = if let Some(captured_piece) = um.capture {
                                        Some(captured_piece)
                                    }
                                    else {
                                        None
                                    };
                            }
                            else {
                                println!("Un move failed at promotion (non-pawn move): {}", um.movement.to_text());
                                return false;
                            }
                        }
                        else {
                            println!("Un move failed at promotion (no piece in destination): {}", um.movement.to_text());
                            return false;
                        }
                    }
                    else {
                        println!("Un move failed at promotion (destination is not final rank): {}", um.movement.to_text());
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
            self.white_king = um.white_king;
            self.black_king = um.black_king;
            self.material = um.material;
            self.total_pieces = um.total_pieces;
            self.half_moves = um.half_moves;
            self.full_moves = um.full_moves;

            return true;
        }
        else {
            return false;
        }
    }

    /*
    // for debugging purposes
    #[cfg(debug_assertions)]
    pub fn assert_cached_state(&self) {
        
        let mut white_king = None;
        let mut black_king = None;
        let mut material = 0.0;
        let mut total_pieces = 0_u8;

        for (square, piece) in self.piece_arr.iter().enumerate() {
            let Some(piece) = piece else {
                continue;
            };

            total_pieces += 1;

            match piece.side {
                Side::White => material += piece.get_value(),
                Side::Black => material -= piece.get_value(),
            }

            if piece.kind == PieceKind::King {
                match piece.side {
                    Side::White => {
                        white_king = Some(square as i8);
                    }
                    Side::Black => {
                        black_king = Some(square as i8);
                    }
                }
            }
        }

        debug_assert_eq!(self.white_king, white_king.unwrap());
        debug_assert_eq!(self.black_king, black_king.unwrap());
        debug_assert_eq!(self.total_pieces, total_pieces);
        debug_assert_eq!(self.material, material);
    }
    */

    // Relocates a piece while editing a position.
    // This does not change side_to_move, create an UnMove, apply promotion rules, or modify castling rights.
    ///
    /// It does: update cached king positions, update material and total piece count if a piece is replaced, 
    // invalidate en passant and prevent either king from being overwritten.
    ///
    /// The method returns false if the edit is invalid.
    pub fn relocate_piece(&mut self, from: i8, to: i8) -> bool {
        // Prevent invalid array indexing.
        if !(0..64).contains(&from) ||!(0..64).contains(&to) {
            return false;
        }

        // Relocating a piece to its current square is a harmless no-op.
        if from == to {
            return self.piece_arr[from as usize].is_some();
        }

        let Some(moving_piece) = self.piece_arr[from as usize] 
        else {
            return false;
        };

        let destination_piece = self.piece_arr[to as usize];

        // Neither king may be removed or overwritten.
        if let Some(piece) = destination_piece {
            if piece.kind == PieceKind::King {
                return false;
            }
        }

        // If the destination is occupied, remove that piece from
        // the cached material balance and piece count.

        if let Some(removed_piece) = destination_piece {
            self.total_pieces -= 1;

            match removed_piece.side {
                Side::White => self.material -= removed_piece.get_value(),
                Side::Black => self.material += removed_piece.get_value(),
            }
        };

        // Relocate the piece.
        self.piece_arr[to as usize] = Some(moving_piece);

        self.piece_arr[from as usize] = None;

        // Keep cached king positions synchronized.
        if moving_piece.kind == PieceKind::King {
            match moving_piece.side {
                Side::White => self.white_king = to,
                Side::Black => self.black_king = to,
            }
        }

        // Arbitrarily editing the position invalidates en passant.
        self.en_passant = None;

        return true;    
    }

}

// for converting coordinates into moves [FIXED]
pub fn coordinates_to_move(board_state: &mut BoardState, from: i8, to: i8) -> Move {
    
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

// methods for accessing piece information at various squares
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

    // calculates if a movement is in bounds (given as (rank, file))
    // returns None if out of bounds
    // otherwise returns Some(resulting square)
    pub fn stays_in_bounds(&self, square: i8, displacement: (i8,i8)) -> Option<i8> {
        if square / 8 + displacement.0 > -1 && square / 8 + displacement.0 < 8 && square % 8 + displacement.1 > -1 && square % 8 + displacement.1 < 8 {
            return Some(square + 8*displacement.0 + displacement.1);
        }
        else {
            return None;
        }
    }

    // adds a piece to the board in the exploration mode
    pub fn add_piece(&mut self, square: i8, piece: Option<Piece>) -> bool {

        // record a change in the material balance and total pieces
        if let Some(captured_piece) = self.piece_arr[square as usize] {

            if captured_piece.kind == PieceKind::King {
                return false;
            }

            self.total_pieces -= 1;
            match captured_piece.side {
                Side::White => self.material -= captured_piece.get_value(),
                Side::Black => self.material += captured_piece.get_value(),
            }
        }

        if let Some(placed_piece) = piece {
            match placed_piece.side {
                Side::White => self.material += placed_piece.get_value(),
                Side::Black => self.material -= placed_piece.get_value(),
            }
            self.total_pieces += 1;
        }   

        // control en-passant
        if let Some((from,to)) = self.en_passant {
            if to == square ||(to + from)/2 == square {
                self.en_passant = None;

            }
        }     

        // finally place the piece in the square
        self.piece_arr[square as usize] = piece;

        return true;   
    }

    pub fn clear(&mut self) {
        // clears all the pieces but the kings
        for i in 0..64 {
            if let Some(piece) = self.piece_arr[i] {
                if piece.kind != PieceKind::King {
                    self.piece_arr[i] = None;
                }
            }
        }

        self.white_short_castle = false;
        self.white_long_castle = false;
        self.black_short_castle = false;
        self.black_long_castle = false;
        self.en_passant = None;
        self.material = 0.;
        self.total_pieces = 2;

    }

    // gets the fen code of the position
    pub fn to_fen(&self) -> String {
       
        let mut res = "".to_string();
        let mut empty_squares: usize = 0;

        for i in 0..8 {
            for j in 0..8 {
                if let Some(piece) = self.piece_arr[(8*i + j) as usize] {
                    // indicate the empty squares
                    if empty_squares > 0 {
                        res += &empty_squares.to_string();
                        empty_squares = 0;
                    }

                    // note the piece
                    res += match (piece.side, piece.kind) {
                        (Side::White, PieceKind:: King) => "K",
                        (Side::White, PieceKind:: Queen) => "Q",
                        (Side::White, PieceKind:: Rook) => "R",
                        (Side::White, PieceKind:: Bishop) => "B",
                        (Side::White, PieceKind:: Knight) => "N",
                        (Side::White, PieceKind:: Pawn) => "P",
                        (Side::Black, PieceKind:: King) => "k",
                        (Side::Black, PieceKind:: Queen) => "q",
                        (Side::Black, PieceKind:: Rook) => "r",
                        (Side::Black, PieceKind:: Bishop) => "b",
                        (Side::Black, PieceKind:: Knight) => "n",
                        (Side::Black, PieceKind:: Pawn) => "p",
                    };
                }
                else {
                    // another empty square
                    empty_squares += 1;
                }
            }
            // indicate a new rank
            if i < 7 {
                if empty_squares > 0 {
                    res += &empty_squares.to_string();
                    empty_squares = 0;
                }
                res += "/";
            }
        }

        // the side to move
        res += match self.side_to_move { 
            Side::White => " w ",
            Side::Black => " b ",
        };

        // castling rights
        if self.white_short_castle {
            res += "K"
        }
        if self.white_long_castle {
            res += "Q"
        }
        if !self.white_short_castle && !self.white_long_castle {
            res += "-";
        }

        if self.black_short_castle {
            res += "k"
        }
        if self.black_long_castle {
            res += "q"
        }
        if !self.black_short_castle && !self.black_long_castle {
            res += "-";
        }

        res += " ";

        // en passant
        if let Some((from, to)) = self.en_passant {
            // if there was an en-passant made, then we just need to make sure that there is another pawn in place
            

            if let Some(piece) = self.piece_arr[((from  + to)/2 - 1) as usize] && self.stays_in_bounds((from + to)/2, (0,-1)).is_some() {
                if let Some(pawn) = self.piece_arr[to as usize] {
                    if piece.kind == PieceKind::Pawn && piece.side.opposite() == pawn.side && pawn.kind == piece.kind {
                        res += &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'][(to%8) as usize].to_string();
                        res += &(8 - (from/8 + to/8)/2).to_string();
                    }
                }                
            }
            else if let Some(piece) = self.piece_arr[((from  + to)/2 + 1) as usize] && self.stays_in_bounds((from + to)/2, (0,1)).is_some()  {
                if let Some(pawn) = self.piece_arr[to as usize] {
                    if piece.kind == PieceKind::Pawn && piece.side.opposite() == pawn.side && pawn.kind == piece.kind {
                        res += &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'][(to%8) as usize].to_string();
                        res += &(8 - (from/8 + to/8)/2).to_string();
                    }
                }
            }
            else {
                res += "-";
            }

        }
        else {
            res += "-";
        }

        // the halfmove clock
        res += " ";
        res += &self.half_moves.to_string();

        // the fullmove clock
        res += " ";
        res += &self.full_moves.to_string();

        return res;
    }


}

// methods for setting up the starting position of the board
impl BoardState { 

    pub fn to_starting_position(&mut self) {
        self.side_to_move = Side::White;
        self.piece_arr = self.starting_pieces();

        self.white_short_castle = true;
        self.white_long_castle = true;
        self.black_short_castle = true;
        self.black_long_castle = true;
        self.en_passant = None;

        self.white_king = 60;
        self.black_king = 4;
        self.material = 0.0;
        self.total_pieces = 32;

        self.half_moves = 0;
        self.full_moves = 1;
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

// some finer structures for the analysis
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoredMove {
    pub chess_move: Move,
    pub score: f64,
}

// saves the data that is necessary to revert to the previous state [FIXED]
#[derive(Debug, PartialEq, Clone)]
pub struct UnMove {
    pub movement: Move,
    pub previous_side_to_move: Side,
    pub capture: Option<Piece>,
    pub white_short_castle: bool,
    pub white_long_castle: bool,
    pub black_short_castle: bool,
    pub black_long_castle: bool,
    pub en_passant: Option<(i8,i8)>,
    pub white_king: i8,
    pub black_king: i8,
    pub material: f64,
    pub total_pieces: u8,
    pub half_moves: usize,
    pub full_moves: usize,
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