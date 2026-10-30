use std::f64::INFINITY;

use polyglot_book_rs::{
    PolyglotBook,
    PolyglotMove,
    PolyglotEntry,
};
 
use polyglot_book_rs::types::Piece as PolyglotPiece;

use crate::chess::{
    BoardState,
    Move,
    ScoredMove,
    Side,
    PieceKind,
};

const CONFIDENCE_WEIGHT: f64 = 50_000.;
const MAX_CONFIDENCE: f64 = 0.8;
const ROLLOUT_DEPTH: usize = 0;
const UPDATE_DELAY: usize = 500;
const BOOK_WEIGHT_EXPONENT: f64 = 1.25;
const DEPTH_THRESHOLD: usize = 100;
const PRIORITY_WEIGHT: f64 = 2.3;
const C: f64 = 1.4;

pub struct AnalysisTree {
    pub root: AnalysisNode,
    pub board_state_backup: BoardState,
}

// functions performing modifications to the tree
impl AnalysisTree {

    // resets the analysis tree to a given position
    pub fn reset_to(&mut self, board_state: &BoardState) {
        self.root = AnalysisNode { 
            children: Vec::<AnalysisNode>::new(),
            expanded: false,
            solved: None,
            visits: 0, 
            total_value: 0., 
            minimax_value: None,
            m: ScoredMove{ chess_move: Move::Standard(-1,-1), score: 0.}, 
            s: board_state.side_to_move,
        };

        self.board_state_backup = board_state.clone();
    }

    pub fn see_analysis(&self) {

        let mut total_visits: usize = 0;
        for child in &self.root.children {
            total_visits += child.visits;
        }

        if total_visits == 0 {
            return;
        }

        let accuracy: f64 = 1_000.;
        let mut index = 0;

        for child in &self.root.children {
            
            if (child.visits as f64) / (total_visits as f64) > -0.01 {
                println!(
                    "{:?}: solved = {:?}, side = {:?}, visits = {}, ucb = {:?}, combined = {}",
                    child.m.chess_move.to_text(),
                    child.solved,
                    child.s,
                    //(accuracy * (100. * child.visits as f64 / total_visits as f64)).round() / accuracy,
                    child.visits,
                    self.root.ucb(index),
                    (accuracy * child.combined_value()).round() / accuracy,
                );
            }
            index += 1;
        }
        println!(" ---- ")
    }

    // resets the analysis tree to the starting position 
    pub fn reset(&mut self) {
        self.root = AnalysisNode { 
            children: Vec::<AnalysisNode>::new(),
            expanded: false,
            solved: None,
            visits: 0, 
            total_value: 0., 
            minimax_value: None,
            m: ScoredMove {chess_move: Move::Standard(-1,-1), score: 0.}, 
            s: Side::White 
        };

        self.board_state_backup.to_starting_position();
    }

    // traverse down the tree
    pub fn traverse(&mut self) {
        // Traverse the tree using a copy of the current board
        if self.root.solved.is_none() {
            self.root.traverse(&mut self.board_state_backup);
        }
    } 

    // make a custom move and update the tree accordingly
    // this function assumes that the given move is in fact legal
    pub fn make_given_move(&mut self, m: ScoredMove) {

        self.board_state_backup.apply_move_unchecked(&m.chess_move);
        
        // find the child containing the suggested move, if it exists
        let mut index_of_move: Option<usize> = None;
        for index in 0..self.root.children.len() {
            if self.root.children[index].m == m {
                index_of_move = Some(index);
                break;
            }
        }

        // update the tree accordinly
        if let Some(index) = index_of_move {
            let new_root = self.root.children.swap_remove(index);

            self.root = new_root;
        }
        else {
            self.root = AnalysisNode { children: Vec::<AnalysisNode>::new(), 
                                        expanded: false,
                                        solved: None,
                                        visits: 0, 
                                        total_value: 0., 
                                        minimax_value: None,
                                        m: m,
                                        s: self.board_state_backup.side_to_move,
                                    };
        }

    }

}

// functions about information about the tree
impl AnalysisTree {

    // the best move in the tree
    pub fn get_best_move(&self, book: &Option<PolyglotBook>) -> Option<Move> {

        // if this is a book entry, then we choose randomly from the book
        if let Some(opening_book) = book {
            let book_moves = opening_book.get_all_moves_from_fen(&self.board_state_backup.to_fen());
            if book_moves.len() > 0 {
                if let Some(m) = self.choose_book_move(book_moves) {
                    return Some(m);
                }
            }

            // otherwise, we return the best move according to the analysis
            if let Some(index) = self.root.get_best_move() {
                return Some(self.root.children[index].m.chess_move);
            }
            else {
                return None;
            }
        }
        else {
            // If there is no book, we just return the best move according to the analysis
            if let Some(index) = self.root.get_best_move() {
                return Some(self.root.children[index].m.chess_move);
            }
            else {
                return None;
            }
        }
        
    }

    pub fn choose_book_move(&self, book_moves: Vec<PolyglotEntry>) -> Option<Move> {

        // first calculate the total weight
        let total: f64 = 
            book_moves
            .iter()
            .map(|entry| {(entry.weight as f64).powf(BOOK_WEIGHT_EXPONENT)} )
            .sum();

        // choose a random number in the appropriate range
        let mut choice = macroquad::rand::gen_range(0.0, total);

        // then choose the move according to choice
        for entry in book_moves {
            choice -= (entry.weight as f64).powf(BOOK_WEIGHT_EXPONENT);
            if choice < 0. {
                return self.board_state_backup.polyglot_move_to_move(entry.chess_move);
            }
        }

        return None;

    }
    
}

impl BoardState {
    pub fn polyglot_move_to_move(&self, poly_move: PolyglotMove) -> Option<Move> {

        // useful information
        let mut from: i8 = poly_move.from as i8;
        let mut to: i8 = poly_move.to as i8;

        from = from%8 + 8*(7 - from/8);
        to = to%8 + 8*(7 - to/8);
        let moving_piece = self.piece_arr[from as usize];

        // handling a kings move (in particular, the weird castling notation)
        if let Some(piece) = moving_piece && piece.kind == PieceKind::King {
            if from == 60 && to == 63 && piece.side == Side::White {
                return Some(Move::Castle(60,62));
            }
            if from == 60 && to == 56 && piece.side == Side::White {
                return Some(Move::Castle(60,58));
            }
            if from == 4 && to == 7 && piece.side == Side::Black {
                return Some(Move::Castle(4,6));
            }
            if from == 4 && to == 0 && piece.side == Side::Black {
                return Some(Move::Castle(4,2));
            }
            return Some(Move::Standard(from,to));
        }

        // then we see if this was a promoting move 
        if let Some(poly_piece) = poly_move.promotion {
            if let Some(kind) = convert_poly_kind(poly_piece) {
                return Some(Move::Promotion(convert_poly_square(poly_move.from), convert_poly_square(poly_move.to), kind));
            }
            else {
                return None;
            }
        }   

        // then if this was a move that is capturing en-passant 
        if let Some(piece) = moving_piece && piece.kind == PieceKind::Pawn && self.piece_arr[to as usize].is_none() {
            if (from - to).abs()%8 != 0 {
                return Some(Move::EnPassant(from, to));
            }
            else {
                return Some(Move::Standard(from, to));
            }
            
        }

        // if none of the above, then this is a standard move
        return Some(Move::Standard(from,to));
    }
}

pub fn convert_poly_square(square: u8) -> i8 {
    return (square%8 + 8*(7 - square/8)) as i8;
}

pub fn convert_poly_kind(poly_piece: PolyglotPiece) -> Option<PieceKind> {
    return match poly_piece {
        PolyglotPiece::WPawn |PolyglotPiece::BPawn => Some(PieceKind::Pawn),
        PolyglotPiece::WKnight |PolyglotPiece::BKnight => Some(PieceKind::Knight),
        PolyglotPiece::WBishop |PolyglotPiece::BBishop => Some(PieceKind::Bishop),
        PolyglotPiece::WRook |PolyglotPiece::BRook => Some(PieceKind::Rook),
        PolyglotPiece::WQueen |PolyglotPiece::BQueen => Some(PieceKind::Queen),
        PolyglotPiece::WKing |PolyglotPiece::BKing => Some(PieceKind::King),
        PolyglotPiece::Empty => None
    };
}


pub struct AnalysisNode {
    pub children: Vec<AnalysisNode>,
    pub expanded: bool,
    pub solved: Option<Solved>,
    pub visits: usize,
    pub total_value: f64,
    pub minimax_value: Option<f64>,
    pub m: ScoredMove, // the move that led here
    pub s: Side, // the side whose turn it is now
}

impl AnalysisNode {

    // returns a proxy for how deep the analysis has gone
    pub fn get_depth(&self) -> isize {

        if let Some(_) = self.solved {
            return -1 + self.children
                .iter()
                .max_by(|c_1,c_2| {c_1.get_depth().cmp(&c_2.get_depth())})
                .map(|child| 1 + child.get_depth())
                .unwrap_or(0);
        }

        if self.visits > DEPTH_THRESHOLD {
            if let Some(index) = self.get_best_move() {
                return 1 + self.children[index].get_depth();
            }
            else {
                return 1;
            }
        }
        return (1 + ROLLOUT_DEPTH) as isize;
        
    }

    // returns the combined evaluation from the minimax search and the MC mean
    pub fn combined_value(&self) -> f64 {

        if let Some(solved) = self.solved {
            return solved.solved_value();
        }

        if self.visits == 0 {
            return 0.0;
        }

        let mean = self.total_value / self.visits as f64;
        let minimax = self.minimax_value.unwrap_or(mean);

        let confidence = MAX_CONFIDENCE * self.visits as f64 / (self.visits as f64 + CONFIDENCE_WEIGHT);

        return confidence * minimax + (1. - confidence) * mean;
    }

    // returns the index of the promising child so far if there is one
    pub fn get_best_move(&self) -> Option<usize> {

        if self.children.is_empty() {
            return None;
        }

        let side = self.s;
        let opponent = side.opposite();

        // A proven winning move is always best.
        if let Some(index) = self.children.iter().position(|child| { child.solved == Some(Solved::Win(side)) }) {
            return Some(index);
        }

        // Ignore moves that are proven to lose,
        // provided another candidate exists.
        let mut candidates = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                child.solved
                    != Some(Solved::Win(opponent))
            })
            .peekable();

        if candidates.peek().is_some() {
            return match side {
                Side::White => {
                    candidates
                        .max_by(|(_, left), (_, right)| { child_selection_value(left).total_cmp(&child_selection_value(right)) })
                        .map(|(index, _)| index)
                },
                Side::Black => {
                    candidates
                        .min_by(|(_, left), (_, right)| { child_selection_value(left).total_cmp(&child_selection_value(right)) })
                        .map(|(index, _)| index)
                },
            };
        }

        // Every move is proven to lose.
        // Without mate-distance information, choose any.
        Some(0)
    }

    pub fn ucb(&self,index: usize) -> Option<f64> {
        if self.children.len() > index {
            let child = &self.children[index];
            let sign: f64 = match self.s {
                Side::White => 1.,
                Side::Black => -1.,
            };

            let total_visits: usize = self.children.iter().map(|child| child.visits).sum();

            let child_visits = child.visits as f64;

            let priority: f64 = PRIORITY_WEIGHT*(1. + child.m.score)/(child.visits as f64);

            let exploitation = sign * child.combined_value();

            let exploration = (C + priority) * ((self.visits as f64).max(1.0).ln() / child_visits).sqrt();
            //println!("exploitation: {}, priority: {}, exploration: {}", exploitation, priority, exploration);
            return Some((1. - child.visits as f64 / total_visits as f64) * (0.1 + exploitation + exploration));
        }
        else {
            return None;
        }
    }

    // returns the index of the child with the best UCB_1 evaluation, if there is one (ties are handled poorly)
    pub fn ucb_child(&self) -> Option<usize> {

        if self.children.is_empty() {
            return None;
        }

        // First visit every unsolved child at least once.
        if let Some(index) = self.children
            .iter()
            .position(|child| { child.solved.is_none() && child.visits == 0 })
        {
            return Some(index);
        }

        let sign = match self.s {
            Side::White => 1.0,
            Side::Black => -1.0,
        };

        let parent_log = (self.visits as f64).max(1.0).ln();
        let total_visits: usize = self.children.iter().map(|child| child.visits).sum();

        self.children
            .iter()
            .enumerate()
            .filter(|(_, child)| child.solved.is_none() )
            .map(|(index, child)| {
                let child_visits = child.visits as f64;

                let priority: f64 = PRIORITY_WEIGHT*(1. + child.m.score)/(child.visits as f64);

                let exploitation = sign * child.combined_value();

                let exploration = (C + priority) * (parent_log / child_visits).sqrt();

                (index, (1. - child.visits as f64 / total_visits as f64) * (0.1 + exploitation + exploration))
            })
            .max_by(|(_, left), (_, right)| { left.total_cmp(right) })
            .map(|(index, _)| index)

    }

    // initializes the children of a node
    pub fn initialize_children(&mut self, board_state: &mut BoardState) {

        assert!(!self.expanded, "Tried to expand an expanded node");

        let legal_move_list = board_state.get_legal_moves();

        self.children = Vec::with_capacity(legal_move_list.len());

        for m in legal_move_list {

            self.children.push(
                AnalysisNode { children: Vec::<AnalysisNode>::new(),
                            expanded: false, 
                            solved: None,
                            visits: 0, 
                            total_value: 0., 
                            minimax_value: None,
                            m: ScoredMove{chess_move: m, score: board_state.priority(&m)},
                            s: self.s.opposite(),
            });
            
        }
    }  

    // the rollout, for now nothing really random, just an evaluation at face value
    pub fn rollout(&self, board_state: &mut BoardState) -> f64 {
        return eval_to_mc(minimax(board_state, ROLLOUT_DEPTH, -INFINITY,INFINITY));
    }

    // makes one traversal down and updates the information in each node
    pub fn traverse(&mut self, board_state: &mut BoardState) -> f64 {
        
        self.update_solved();

        // solved nodes need not be traversed
        if let Some(solved) = self.solved {
            return solved.solved_value();
        }

        // on first visit we do static evaluation only
        if self.visits == 0 {
            let result = self.rollout(board_state);

            self.visits = 1;
            self.total_value = result;
            self.minimax_value = Some(result);

            return result;
        }

        // on second visit we expand
        if !self.expanded {
            self.initialize_children(board_state);
            self.expanded = true;

            if self.children.is_empty() {
                let solved = if board_state
                    .is_in_check(board_state.side_to_move)
                {
                    Solved::Win(
                        board_state.side_to_move.opposite(),
                    )
                } else {
                    Solved::Draw
                };

                let result = solved.solved_value();

                self.solved = Some(solved);
                self.visits += 1;
                self.total_value += result;
                self.minimax_value = Some(result);

                return result;
            }
        }

        let index = self.ucb_child().expect("Unsolved node has no unsolved child");

        let un_move = board_state.apply_move_unchecked(&self.children[index].m.chess_move);

        let result = self.children[index].traverse(board_state);

        let restored = board_state.un_move_unchecked(un_move);

        assert!(restored, "Failed to restore board during traversal");

        self.visits += 1;
        self.total_value += result;

        self.update_solved();

        if self.visits % UPDATE_DELAY == 0 {
            self.update_minimax_value();
        }

        return result;
    }
    
    fn update_minimax_value(&mut self) {
        let child_values = self
            .children
            .iter()
            .filter_map(|child| {
                child.minimax_value
            });

        self.minimax_value = match self.s {
            Side::White => {
                child_values.max_by(f64::total_cmp)
            }

            Side::Black => {
                child_values.min_by(f64::total_cmp)
            }
        };
    }

    fn update_solved(&mut self) {

        // if the node has not been expanded, then no conclusion can be made
        if !self.expanded {
            return;
        }

        // at this stage the node has been expanded, whenever we expand,
        // we make sure to decide on the nodes state of being solved, 
        // so we only need to relay this information of it is
        if self.solved.is_some() {
            return;
        }

        // establish the sides
        let side = self.s;
        let opposite = side.opposite();

        // If there is one continuation that the
        // current player can force as a win, the
        // current node is won.
        if self.children.iter().any(|child| { child.solved == Some(Solved::Win(side)) }) {
            
            let solved = Solved::Win(side);

            self.solved = Some(solved);
            self.minimax_value = Some(solved.solved_value());

            return;
        }

        // A draw or loss cannot be proven until every
        // possible continuation has been solved.
        if self.children.iter().any(|child| { child.solved.is_none() }) {
            return;
        }

        // Every child is now solved. If at least one
        // child draws, the current player can preserve
        // that draw.
        if self.children.iter().any(|child| {
            child.solved == Some(Solved::Draw)
        }) {
            self.solved = Some(Solved::Draw);
            self.minimax_value = Some(0.0);

            return;
        }

        // Every possible move is a win for the opponent.
        let solved = Solved::Win(opposite);

        self.solved = Some(solved);
        self.minimax_value = Some(solved.solved_value());
    }

}

fn child_selection_value(child: &AnalysisNode) -> f64 {
    match child.solved {
        Some(solved) => solved.solved_value(),
        None => child.combined_value(),
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Solved {
    Win(Side),
    Draw,
}

impl Solved {
    fn solved_value(&self) -> f64 {
        match self {
            Solved::Win(Side::White) => 1.0,
            Solved::Win(Side::Black) => -1.0,
            Solved::Draw => 0.0,
        }
    }
}

// a simple alpha - beta pruning way of doing minimax
fn minimax(board_state: &mut BoardState, depth: usize, mut alpha: f64, mut beta: f64) -> f64 {


    // check if we have reached the recursion bottom
    if depth == 0 {
        return board_state.evaluation();
    }

    // then check for a terminal state
    let legal_move_list = board_state.get_legal_moves();

    if legal_move_list.len() == 0 {
        if board_state.is_in_check(board_state.side_to_move) {
            return match board_state.side_to_move {
                Side::White => -10_000. - depth as f64,
                Side::Black => 10_000. + depth as f64,
            };
        }
        else {
            return 0.;
        }
    }

    // if we are at a maximizing node (white to move) 
    if board_state.side_to_move == Side::White {

        // because we are a maximizing player now
        let mut value = -INFINITY;

        // then make each legal move one by one
        //legal_move_list.as_mut_slice().sort_by_key(|entry| std::cmp::Reverse(entry.score) );

        for m in legal_move_list {
            
            let un_move = board_state.apply_move_unchecked(&m);

            // check if we beat value
            value = value.max(minimax(board_state, depth - 1, alpha, beta));

            // unmake the move before anything else 

            let restored = board_state.un_move_unchecked(un_move);

            assert!(restored,"Failed to restore board during minimax");
            //board_state.assert_cached_state();
        
            if value >= beta {
                // in this case, we have evidence that we already know the optimal value
                break;
            }

            alpha = alpha.max(value);
        
        }
        return value;
    }
    else {

        // because we are a minimizing player now
        let mut value = INFINITY;

        // then make each legal move one by one
        //legal_move_list.as_mut_slice().sort_unstable_by_key(|entry| std::cmp::Reverse(entry.score));

        for m in legal_move_list {

            let un_move = board_state.apply_move_unchecked(&m);

            // check if we beat value
            value = value.min(minimax(board_state, depth - 1, alpha, beta));

            // unmake the move before anything else 
            let restored = board_state.un_move_unchecked(un_move);
            assert!(restored, "Failed to restore board during minimax");
            //board_state.assert_cached_state();
            
            if value <= alpha {
                // in this case, we have evidence that we already know the optimal value
                break;
            }

            beta = beta.min(value);
            
        }
        return value;
    }

}


const NORMALIZATION: f64 = 4.;
// returns a value normalizes between -1 and 1, given an evaluation
pub fn eval_to_mc(evaluation: f64) -> f64 {
    return -1. + 2./(1. + (-evaluation/NORMALIZATION).exp());
} 

pub fn mc_to_eval(mc: f64) -> f64 {
    let mc = mc.clamp(-1.0 + f64::EPSILON, 1.0 - f64::EPSILON);
    -((2.0 / (1.0 + mc)) - 1.0).ln() * NORMALIZATION
}