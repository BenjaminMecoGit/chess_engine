use std::f64::INFINITY;

use crate::chess::{
    BoardState,
    Move,
    Side,
};

const CONFIDENCE_WEIGHT: f64 = 10_000.;
const MAX_CONFIDENCE: f64 = 0.4;
const ROLLOUT_DEPTH: usize = 2;
const C: f64 = 1.3;

pub struct AnalysisTree {
    pub root: AnalysisNode,
    pub board_state_backup: BoardState,
}

// functions performing modifications to the tree
impl AnalysisTree {

    // resets the analysis tree
    pub fn reset(&mut self) {
        self.root = AnalysisNode { 
            children: Vec::<AnalysisNode>::new(),
            visits: 0, 
            total_value: 0., 
            minimax_value: None,
            m: Move::Standard(-1,-1), 
            s: Side::White 
        };

        self.board_state_backup.to_starting_position();
    }

    // traverse down the tree
    pub fn traverse(&mut self) {
        // Traverse the tree using a copy of the current board
        self.root.traverse(&mut self.board_state_backup.clone());
    } 

    // make a custom move and update the tree accordingly
    // this function assumes that the given move is in fact legal
    pub fn make_given_move(&mut self, m: Move) {

        self.board_state_backup.apply_move_unchecked(&m);
        
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
    pub fn get_best_move(&self) -> Option<Move> {
        if let Some(index) = self.root.get_best_move() {
            return Some(self.root.children[index].m.clone());
        }
        else {
            return None;
        }
    }

    
    // the amount of visits in the root
    pub fn get_visits(&self) -> usize {
        return self.root.visits;
    }
    
}

pub struct AnalysisNode {
    pub children: Vec<AnalysisNode>,
    pub visits: usize,
    pub total_value: f64,
    pub minimax_value: Option<f64>,
    pub m: Move, // the move that led here
    pub s: Side, // the side whose turn it is now
}

impl AnalysisNode {

    // returns the average value from the current traversals
    pub fn get_value(&self) -> f64 {
        if self.visits == 0 {
            return 0.;
        }
        else {
            return mc_to_eval(self.total_value/(self.visits as f64));
        }
    }

    // returns the combined evaluation from the minimax search and the MC mean
    pub fn combined_value(&self) -> f64 {
        let mean = self.get_value();
        let minimax = self.minimax_value.unwrap_or(mean);

        let confidence = MAX_CONFIDENCE * self.visits as f64 / (self.visits as f64 + CONFIDENCE_WEIGHT);

        return confidence * minimax + (1. - confidence) * mean;
    }

    // returns the index of the promising child so far if there is one
    pub fn get_best_move(&self) -> Option<usize> {

        let mut highscore: usize = 0; 
        let mut winner: Option<usize> = None;

        for index in 0..self.children.len() {

            if self.children[index].visits >= highscore {
                highscore = self.children[index].visits;
                winner = Some(index);
            }
        }

        return winner;
    }

    // returns the index of the child with the best UCB_1 evaluation, if there is one (ties are handled poorly)
    pub fn ucb_child(&self) -> Option<usize> {

        if self.children.is_empty() {
            return None;
        }

        // first make sure to visit all children at least once
        if let Some(index) = self.children.iter().position(|child| child.visits == 0) {
            return Some(index);
        }

        // otherwise, find the child with the highest ucb evaluation
        let sign: f64 = match self.s {
            Side::White => 1.,
            Side::Black => -1.,
        };

        let parent_log:f64 = (self.visits as f64).max(1.).ln();

        self.children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let child_visits = child.visits as f64;
                let exploitation = sign * child.combined_value();
                let exploration = C * (parent_log/child_visits).sqrt();
                
                let ucb = exploitation + exploration;

                (index, ucb)
            })
            .max_by(|(_, left), (_, right)| {
                left.total_cmp(right)
            })
            .map(|(index, _)| {
                index
            })  

    }

    // initializes the children of a node
    pub fn initialize_children(&mut self, board_state: &mut BoardState) {
        let legal_moves = board_state.get_legal_moves();

        self.children = Vec::<AnalysisNode>::new();

        for m in legal_moves {
            self.children.push(
                AnalysisNode { children: Vec::<AnalysisNode>::new(), 
                               visits: 0, 
                               total_value: 0., 
                               minimax_value: None,
                               m: m,
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
        
        if self.children.len() == 0 {
            // this is a leaf
            if self.visits == 0 {
                // and with zero visits, then
                self.visits += 1;
                let rollout_result: f64 = self.rollout(board_state);
                self.total_value += rollout_result;
                self.minimax_value = Some(rollout_result);
                return rollout_result;
            }
            else {
                // if we have visited this already, 
                // then we should give children if it is the first revisit:
                if self.visits == 1 {
                    self.initialize_children(board_state);
                }

                // check for a terminal state
                if self.children.len() == 0 {
                    self.visits += 1;

                    if board_state.is_in_check(Side::White) {
                        self.total_value += -1.;
                        return -1.;
                    }
                    else if board_state.is_in_check(Side::Black) {
                        self.total_value += 1.;
                        return 1.
                    }
                    else {
                        return 0.;
                    }
                }
                else {
                    // and if this is not a terminal state, 
                    // then at this point there are children in the node, 
                    // so we can continue the recursion:
                    return self.traverse(board_state);
                }

            }
        }
        else {
            // if there are children, then first we have to choose the ucb child
            let index: usize = self.ucb_child().expect("There was no index in traverse!");

            // apply the child's move to the board
            board_state.apply_move_unchecked(&self.children[index].m);

            // then go deeper
            let result = self.children[index].traverse(board_state);

            // having the result, we are ready to back propagate:
            self.visits += 1;
            self.total_value += result;
            self.update_minimax_value();

            return result;

        }


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

}


// a simple alpha - beta pruning way of doing minimax
pub fn minimax(board_state: &mut BoardState, depth: usize, mut alpha: f64, mut beta: f64) -> f64 {

    // first check for a terminal state
    let mut legal_moves = board_state.get_legal_moves();

    if legal_moves.len() == 0 {
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

    // next we check if we have reached the recursion bottom
    if depth == 0 {
        return board_state.evaluation();
    }

    // if we are at a maximizing node (white to move) 
    if board_state.side_to_move == Side::White {

        // because we are a maximizing player now
        let mut value = -INFINITY;

        // then make each legal move one by one
        legal_moves.sort_by(|m1, m2| board_state.priority(m2).total_cmp(&board_state.priority(m1)));

        for m in legal_moves {
            //let mut temp_board = board_state.clone();
            //temp_board.apply_move_unchecked(&m);
            let un_move = board_state.apply_move_unchecked(&m);

            // check if we beat value
            value = value.max(minimax(board_state, depth - 1, alpha, beta));

            // unmake the move before anything else 

            let restored = board_state.un_move_unchecked(un_move);

            debug_assert!(restored,"Failed to restore board during minimax");
        
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
        legal_moves.sort_by(|m1, m2| board_state.priority(m2).total_cmp(&board_state.priority(m1)));

        for m in legal_moves {
            //let mut temp_board = board_state.clone();
            //temp_board.apply_move_unchecked(&m);
            let un_move = board_state.apply_move_unchecked(&m);

            // check if we beat value
            value = value.min(minimax(board_state, depth - 1, alpha, beta));

            // unmake the move before anything else 
            board_state.un_move_unchecked(un_move);

            if value <= alpha {
                // in this case, we have evidence that we already know the optimal value
                break;
            }

            beta = beta.min(value);
        }
        return value;
    }

}



const NORMALIZATION: f64 = 12.;
// returns a value normalizes between -1 and 1, given an evaluation
fn eval_to_mc(evaluation: f64) -> f64 {
    return -1. + 2./(1. + (-evaluation/NORMALIZATION).exp());
} 

fn mc_to_eval(mc: f64) -> f64 {
    return -(2./(1. + mc) - 1.).ln()*NORMALIZATION;
} 