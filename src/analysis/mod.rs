
use crate::chess::{
    BoardState,
    Move,
    Side,
};

//const ROLLOUT_DEPTH: usize = 4;
const C: f64 = 1.4;

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
    pub m: Move, // the move that led here
    pub s: Side, // the side whose turn it is now
}

impl AnalysisNode {

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
        else {

            // first make sure to visit all children at least once
            if let Some(index) = self.children.iter().position(|child| child.visits == 0) {
                return Some(index);
            }

            // otherwise, find the child with the highest ucb evaluation
            let sign: f64 = match self.s {
                Side::White => 1.,
                Side::Black => -1.,
            };

            let parent_log:f64 = (self.visits as f64).ln();

            let mut record: f64 = f64::NEG_INFINITY;
            let mut winner: Option<usize> = None;

            for index in 0..self.children.len() {
                let ucb: f64 = sign*self.children[index].total_value/(self.children[index].visits as f64) + 
                            C * (parent_log/(self.children[index].visits as f64)).sqrt();

                if ucb > record {
                    winner = Some(index);
                    record = ucb;
                }
            }

            return winner;
        }
    }

    // initializes the children of a node
    pub fn initialize_children(&mut self, board_state: &BoardState) {
        let legal_moves = board_state.get_legal_moves();

        self.children = Vec::<AnalysisNode>::new();

        for m in legal_moves {
            self.children.push(
                AnalysisNode { children: Vec::<AnalysisNode>::new(), 
                               visits: 0, total_value: 0., 
                               m: m,
                               s: self.s.opposite(),
            });
        }
    }  

    // the rollout, for now nothing really random, just an evaluation at face value
    pub fn rollout(&self, board_state: &BoardState) -> f64 {
        return eval_to_mc(board_state.evaluation());
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
                    
                    let result: f64 = self.traverse(board_state);
                    self.total_value += result;
                    self.visits += 1;
                    return result;
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
            return result;

        }






        /*
        if self.children.len() > 0 {
            // if the current node is not a leaf, then we go deeper

            // find the index of the ucb child
            let ucb_child_index = self.ucb_child().expect("There was no child in a non-leaf node (traverse)");

            // apply their move to the board state used in tracking the position
            board_state.apply_move_unchecked(&self.children[ucb_child_index].m);

            // now go deeper in the recursion
            let mtc_evaluation: f64 = self.children[ucb_child_index].traverse(board_state);
            self.total_value += mtc_evaluation;
            self.visits += 1;

            return mtc_evaluation;
        }
        else { 
            if self.visits > 0 {
                // if the current node has been visited before, then it needs to be initialized:
                self.initialize_children(board_state);

                // try to find the index of the ucb child
                if let Some(ucb_child_index) = self.ucb_child() {
                    // apply their move to the board state used in tracking the position
                    board_state.apply_move_unchecked(&self.children[ucb_child_index].m);

                    // now go deeper in the recursion
                    let mtc_evaluation: f64 = self.children[ucb_child_index].traverse(board_state);
                    self.total_value += mtc_evaluation;
                    self.visits +=1;

                    return mtc_evaluation;
                }
                else { // if we arrive here, then we are at a terminal state, 
                        // we return the evaluation accordingly
                        if board_state.is_in_check(Side::White) {
                            return -1.;
                        }
                        else if board_state.is_in_check(Side::Black) {
                            return 1.;
                        }
                        else {
                            return 0.;
                        }

                }

                
            }
            else {
                // in the case of a leaf node visited for the first time, perform a rollout, 
                // there is no going deeper
                let mtc_evaluation: f64 = self.rollout(board_state);
                self.total_value += mtc_evaluation;
                self.visits +=1;
                
                return mtc_evaluation;
            }
        }
        */
    }

}

// returns a value normalizes between -1 and 1, given an evaluation
fn eval_to_mc(evaluation: f64) -> f64 {
    return -1. + 2./(1. + (-evaluation/8.).exp());
} 