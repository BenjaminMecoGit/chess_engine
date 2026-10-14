use macroquad::prelude::*;
use std::collections::HashMap;
use polyglot_book_rs::PolyglotBook;

// package and function for working in different directories
use std::path::PathBuf;

fn asset_path(relative_path: &str) -> PathBuf {
    let executable = std::env::current_exe().expect("Could not locate executable");

    let executable_directory = executable.parent().expect("Executable has no parent directory");

    executable_directory.join(relative_path)
}
 
mod chess;
mod analysis;

use crate::chess::{
    coordinates_to_move,
    BoardState,
    ScoredMove,
    Move,
    Piece,
    PieceKind,
    Side,
};

use crate::analysis::{
    AnalysisNode,
    AnalysisTree,
    Solved,
    mc_to_eval,
};

// for appearence
const BACKGROUND_COLOR: Color = Color::new(81.0/256.0, 92.0/256.0, 81.0/256.0, 0.2);
const BUTTON_COLOR: Color = Color::new(20.0/256.0, 29.0/256.0, 38.0/256.0, 1.);
const BUTTON_TEXT_COLOR: Color = WHITE;
const DARK_SQUARE_COLOR: Color = Color::new(0.0/256.0, 68.0/256.0, 116.0/256.0, 1.);
const LIGHT_SQUARE_COLOR: Color = Color::new(251.0/256.0, 245.0/256.0, 222.0/256.0, 1.);
const SQUARE_HIGHLIGHT_COLOR: Color = Color::new(134./256.,204./256.,144./256.,0.86);
//const SQUARE_ATTACKED_COLOR: Color = Color::new(256./256.,0./256.,0./256.,0.86);
const SQUARE_SIZE: f32 = 80.0;
const PIECE_SIZE: f32 = 80.0;
const SMALL_SCALE: f32 = 0.75;
const FONT_SIZE: u16 = 40;
const SMALL_BOARD_X: f32 = 0.955;
const SMALL_BOARD_Y: f32 = 0.5;


// for analysis
const COMPUTER_THINK: f64 = 5.;
const COMPUTER_TIME_FRIEND: f64 = 25./1000.;
const COMPUTER_TIME_ON_PLAYER: f64 = 15./1000.;
const COMPUTER_TIME_ON_COMPUTER: f64 = 45./1000.;
const MAX_VISITS: usize = 10_000_000;

#[macroquad::main("Rust chess")]
async fn main() {

    // setting window size
    request_new_screen_size(1080.,800.);

    // initializing the textures for the pieces
    let piece_textures = ChessTextures {
        black_pawn_texture: load_texture(asset_path("assets/pieces/black_pawn.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        white_pawn_texture: load_texture(asset_path("assets/pieces/white_pawn.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        black_knight_texture: load_texture(asset_path("assets/pieces/black_knight.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        white_knight_texture: load_texture(asset_path("assets/pieces/white_knight.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        black_bishop_texture: load_texture(asset_path("assets/pieces/black_bishop.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        white_bishop_texture: load_texture(asset_path("assets/pieces/white_bishop.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        black_rook_texture: load_texture(asset_path("assets/pieces/black_rook.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        white_rook_texture: load_texture(asset_path("assets/pieces/white_rook.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        black_queen_texture: load_texture(asset_path("assets/pieces/black_queen.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        white_queen_texture: load_texture(asset_path("assets/pieces/white_queen.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        black_king_texture: load_texture(asset_path("assets/pieces/black_king.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
        white_king_texture: load_texture(asset_path("assets/pieces/white_king.png").to_str().expect("Asset path is not valid UTF-8")).await.expect("Could not load texture"),
    };

    // loading the font for various text asset_path("assets/pieces/white_king.png").to_str().expect("Asset path is not valid UTF-8")
    let font = load_ttf_font(asset_path("assets/fonts/IBMPlexSerif-Medium.ttf").to_str().expect("Asset path is not valid UTF-8")).await.unwrap();
    set_default_font(font);

    // loading the opening book
    let book = PolyglotBook::load(asset_path("assets/books/gm2001.bin").to_str().expect("Asset path is not valid UTF-8")).expect("Could not load opening book");

    // the initial game mode
    let mut game_mode: GameMode = GameMode::MainMenu;

    // initializing the board squares
    let mut squares: [[Square; 8];8] = [[Square {file: 0, 
                                                 rank: 0, 
                                                 side_length: SQUARE_SIZE, 
                                                 color: Color::new(0.,0.,0.,0.),
                                                 highlight_color: SQUARE_HIGHLIGHT_COLOR,
                                                 highlight: false,
                                                } ;8];8];

    for file in 0..8 {
        for rank in 0..8 {
            squares[file][rank].file = file as i8;
            squares[file][rank].rank = rank as i8;

            if (file + rank) % 2 == 0 {
                squares[file][rank].color = LIGHT_SQUARE_COLOR;
            }
            else {
                squares[file][rank].color = DARK_SQUARE_COLOR;
            }
        }
    }
                                                
    // initializing squares for dummy pieces in the explore mode
    let mut dummy_squares: [[Square; 5];2] = [[Square {
                                                file: 0, 
                                                rank: 0, 
                                                side_length: SQUARE_SIZE * SMALL_SCALE, 
                                                color: Color::new(0.,0.,0.,0.),
                                                highlight_color: SQUARE_HIGHLIGHT_COLOR,
                                                highlight: false,
                                            } ;5];2];
    let mut dummy_pieces: [[Piece; 5];2] = [[Piece {side: Side::White, kind: PieceKind::Pawn} ;5];2];
    
    for i in 0..2 {
        for j in 0..5 {
            dummy_squares[i as usize][j as usize].file = 8 + i as i8;
            dummy_squares[i as usize][j as usize].rank = 8 + j as i8;

            dummy_squares[i as usize][j as usize].color = match (i + j)%2 {
                0 => DARK_SQUARE_COLOR,
                _ => LIGHT_SQUARE_COLOR
            };

            dummy_pieces[i as usize][j as usize].side = match i {
                0 => Side::White,
                _ => Side::Black,
            };
            dummy_pieces[i as usize][j as usize].kind = match j {
                0 => PieceKind::Queen,
                1 => PieceKind::Rook,
                2 => PieceKind::Bishop,
                3 => PieceKind::Knight,
                _ => PieceKind::Pawn,
            };

        }
    }

    // the initial board state
    let mut board_state: BoardState = BoardState{piece_arr: [None; 64], 
                                                 side_to_move: Side::White,
                                                 white_short_castle: true,
                                                 white_long_castle: true,
                                                 black_short_castle: true,
                                                 black_long_castle: true,
                                                 en_passant: None,
                                                 white_king: 60,
                                                 black_king: 4,
                                                 material: 0.,
                                                 total_pieces: 32,
                                                 half_moves: 0,
                                                 full_moves: 1,
                                                };
    
    /*
    board_state = kiwipete();
    */

    board_state.to_starting_position();
    for i in 0..0 {
        println!("Number of legal moves at depth {}: {}", i, perft(&mut board_state, i))
    }


    // the initial analysis tree
    let mut analysis_tree = AnalysisTree{
        root: AnalysisNode{
            children: Vec::<AnalysisNode>::new(),
            expanded: false,
            solved: None,
            visits: 0,
            total_value: 0.,
            minimax_value: None,
            m: ScoredMove { chess_move: Move::Standard(-1,-1), score: 0. }, // the move that led here, the one for the root does not matter
            s: Side::White, // the side whose turn it is now
        },
        board_state_backup: board_state.clone(),
    };

    // states for the computer game mode
    let mut player_side: Side = Side::White;
    let mut perspective: Side = Side::White;
    let mut computer_ref_time: f64 = get_time();
    
    let mut buttons: HashMap<&str, Button> = HashMap::new();
    buttons.insert("friend mode", spawn_friend_mode_button());
    buttons.insert("computer mode", spawn_computer_mode_button());
    buttons.insert("explore mode", spawn_explore_mode_button());
    buttons.insert("main menu", spawn_go_to_main_button());
    buttons.insert("evaluation", spawn_evaluation_button());
    buttons.insert("perspective", spawn_change_perspective_button());
    buttons.insert("clear board", spawn_clear_board_button());
    buttons.insert("reset board", spawn_reset_board_button());
    buttons.insert("show analysis", spawn_show_analysis_button());
    buttons.insert("select side", spawn_select_side_button());

    buttons.insert("white mode", spawn_white_mode_button());
    buttons.insert("black mode", spawn_black_mode_button());
    buttons.insert("random mode", spawn_random_mode_button());

    buttons.insert("result window", spawn_game_winner_button());

    buttons.insert("white short castle", spawn_wsc_button());
    buttons.insert("white long castle", spawn_wlc_button());
    buttons.insert("black short castle", spawn_bsc_button());
    buttons.insert("black long castle", spawn_blc_button());
    
    buttons.get_mut("friend mode").expect("No friend button").visible = true;
    buttons.get_mut("computer mode").expect("No computer button").visible = true;
    buttons.get_mut("explore mode").expect("No explore button").visible = true;


    // variables for visuals for selected squares
    let mut selected_square: Option<i8> = None; 

    loop {

        // setting up the background
        clear_background(BACKGROUND_COLOR);

        // implementing button behaviors here
        if is_mouse_button_pressed(MouseButton::Left) {
            // handling actions for the buttons
            if buttons["main menu"].mouse_is_on() {

                // game relevent parameters
                game_mode = GameMode::MainMenu;
                analysis_tree.reset();
                board_state.to_starting_position();
                perspective = Side::White;
                player_side = Side::White;
                selected_square = None;

                for file in 0..8 {
                    for rank in 0..8 {
                        squares[file][rank].highlight = false;
                    }
                }

                for i in 0..2 {
                    for j in 0..5 {
                        dummy_squares[i][j].highlight = false;
                    }
                }

                // clear all buttons
                for (_, but) in &mut buttons {
                    but.visible = false;
                    but.shaded = false;
                }

                buttons.get_mut("evaluation").expect("No evaluation button").text = "Show evaluation".to_string();
                buttons.get_mut("evaluation").expect("No evaluation button").text_centered = true;

                // set main menu buttons
                buttons.get_mut("friend mode").expect("No friend button").visible = true;
                buttons.get_mut("computer mode").expect("No computer button").visible = true;
                buttons.get_mut("explore mode").expect("No explore button").visible = true;
                
            }
            else if buttons["evaluation"].mouse_is_on() {
                if buttons["evaluation"].text != "Show evaluation" {
                    buttons.get_mut("evaluation").expect("No evaluation button").text = "Show evaluation".to_string();
                    buttons.get_mut("evaluation").expect("No evaluation button").text_centered = true;
                }
                else {
                    buttons.get_mut("evaluation").expect("No evaluation button").text = "".to_string();
                    buttons.get_mut("evaluation").expect("No evaluation button").text_centered = false;
                }
            }
            else if buttons["perspective"].mouse_is_on() {

                // change the perspectives
                if perspective == Side::White {
                    perspective = Side::Black;
                }
                else {
                    perspective = Side::White;
                }

                // change the highlighting of squares
                for i in 0..32 {
                    (squares[i%8][i/8].highlight, squares[7-i%8][7-i/8].highlight) = (squares[7-i%8][7-i/8].highlight, squares[i%8][i/8].highlight);
                }
            }
            else if buttons["clear board"].mouse_is_on() {

                // clear the board
                board_state.clear();
                analysis_tree.reset_to(&board_state);

                // un-highlight all squares
                for file in 0..8 {
                    for rank in 0..8 {
                        squares[file as usize][rank as usize].highlight = false;
                    }
                }

                for i in 0..2 {
                    for j in 0..5 {
                        dummy_squares[i as usize][j as usize].highlight = false;
                    }
                }
                selected_square = None;
            }
            else if buttons["reset board"].mouse_is_on() {

                // clear the board
                board_state.to_starting_position();
                analysis_tree.reset_to(&board_state);

                // modify the turn button
                match board_state.side_to_move {
                    Side::White => {
                        buttons.get_mut("select side").expect("No select side button").text = "White to move".to_string();
                        buttons.get_mut("select side").expect("No select side button").color = WHITE;
                        buttons.get_mut("select side").expect("No select side button").text_color = BLACK;
                    },
                    Side::Black => {
                        buttons.get_mut("select side").expect("No select side button").text = "Black to move".to_string();
                        buttons.get_mut("select side").expect("No select side button").color = BLACK;
                        buttons.get_mut("select side").expect("No select side button").text_color = WHITE;
                    }
                }

                // un-highlight all squares
                for file in 0..8 {
                    for rank in 0..8 {
                        squares[file as usize][rank as usize].highlight = false;
                    }
                }

                for i in 0..2 {
                    for j in 0..5 {
                        dummy_squares[i as usize][j as usize].highlight = false;
                    }
                }
                selected_square = None;
            }
            else if buttons["select side"].mouse_is_on() {

                // modify the board
                board_state.side_to_move = board_state.side_to_move.opposite();
                analysis_tree.reset_to(&board_state);

                // modify the button
                match board_state.side_to_move {
                    Side::White => {
                        buttons.get_mut("select side").expect("No select side button").text = "White to move".to_string();
                        buttons.get_mut("select side").expect("No select side button").color = WHITE;
                        buttons.get_mut("select side").expect("No select side button").text_color = BLACK;
                    },
                    Side::Black => {
                        buttons.get_mut("select side").expect("No select side button").text = "Black to move".to_string();
                        buttons.get_mut("select side").expect("No select side button").color = BLACK;
                        buttons.get_mut("select side").expect("No select side button").text_color = WHITE;
                    }
                }
 
                // un-highlight all squares
                for file in 0..8 {
                    for rank in 0..8 {
                        squares[file as usize][rank as usize].highlight = false;
                    }
                }

                for i in 0..2 {
                    for j in 0..5 {
                        dummy_squares[i as usize][j as usize].highlight = false;
                    }
                }
                selected_square = None;
            }
            else if buttons["white short castle"].mouse_is_on() {
                // modify the board
                board_state.white_short_castle = true;
                analysis_tree.reset_to(&board_state);
                selected_square = None;
            }
            else if buttons["white long castle"].mouse_is_on() {
                // modify the board
                board_state.white_long_castle = true;
                analysis_tree.reset_to(&board_state);
                selected_square = None;
            }
            else if buttons["black short castle"].mouse_is_on() {
                // modify the board
                board_state.black_short_castle = true;
                analysis_tree.reset_to(&board_state);
                selected_square = None;
            }
            else if buttons["black long castle"].mouse_is_on() {
                // modify the board
                board_state.black_long_castle = true;
                analysis_tree.reset_to(&board_state);
                selected_square = None;
            }
            else if buttons["show analysis"].mouse_is_on() {
                // show the analysis
                analysis_tree.see_analysis();
            }

            // remove any result announcement if it is there
            buttons.get_mut("result window").expect("No result window").visible = false;
            buttons.get_mut("result window").expect("No result window").text = "".to_string();

        }

        // The different game states have different ways of working
        match game_mode { // todo: make this wiser in implementeation, this draw should not repeat this often
            GameMode::MainMenu => {
                
                // handling actions for the main menu buttons
                if is_mouse_button_pressed(MouseButton::Left) {
                    if buttons["friend mode"].mouse_is_on() {
                        game_mode = GameMode::Friend;

                        for (_, but) in &mut buttons {
                            but.visible = false;
                            but.shaded = false;
                        }

                        buttons.get_mut("evaluation").expect("No evaluation button").visible = true;
                        buttons.get_mut("perspective").expect("No perspective button").visible = true;
                        buttons.get_mut("main menu").expect("No menu button").visible = true;

                    }
                    else if buttons["computer mode"].mouse_is_on() {
                        // these buttons should now be visible
                        buttons.get_mut("black mode").expect("No black button").visible = true;
                        buttons.get_mut("white mode").expect("No white button").visible = true;
                        buttons.get_mut("random mode").expect("No random button").visible = true;
                        // and these should fade a bit
                        buttons.get_mut("friend mode").expect("No friend button").shaded = true;
                        buttons.get_mut("computer mode").expect("No computer button").shaded = true;
                        buttons.get_mut("explore mode").expect("No explore button").shaded = true;
                    }
                    else if buttons["explore mode"].mouse_is_on() {
                        game_mode = GameMode::Exploration;

                        // clear all buttons
                        for (_, but) in &mut buttons {
                            but.visible = false;
                            but.shaded = false;
                        }

                        // set the relevant buttons
                        buttons.get_mut("main menu").expect("No menu button").visible = true;
                        buttons.get_mut("evaluation").expect("No menu button").visible = true;
                        buttons.get_mut("perspective").expect("No perspective button").visible = true;
                        buttons.get_mut("clear board").expect("No clear board button").visible = true;
                        buttons.get_mut("reset board").expect("No reset board button").visible = true;
                        buttons.get_mut("show analysis").expect("No show analysis button").visible = true;
                        buttons.get_mut("select side").expect("No select side button").visible = true;

                        buttons.get_mut("white short castle").expect("No wsc button").visible = true;
                        buttons.get_mut("white long castle").expect("No wlc button").visible = true;
                        buttons.get_mut("black short castle").expect("No bsc button").visible = true;
                        buttons.get_mut("black long castle").expect("No blc button").visible = true;

                    }
                    else if buttons["white mode"].mouse_is_on() || buttons["black mode"].mouse_is_on() || buttons["random mode"].mouse_is_on() {
                        game_mode = GameMode::Computer;

                        // choose the side to be played
                        if buttons["white mode"].mouse_is_on() {
                            player_side = Side::White;
                            perspective = Side::White;
                        }
                        else if buttons["black mode"].mouse_is_on() {
                            player_side = Side::Black;
                            perspective = Side::Black;
                        }
                        else if rand::gen_range(-1.,1.) > 0. {
                            player_side = Side::White;
                            perspective = Side::White;
                        }
                        else {
                            player_side = Side::Black;
                            perspective = Side::Black;
                        }

                        // clear all buttons
                        for (_, but) in &mut buttons {
                            but.visible = false;
                            but.shaded = false;
                        }

                        // set visible buttons
                        buttons.get_mut("evaluation").expect("No evaluation button").visible = true;
                        buttons.get_mut("evaluation").expect("No evaluation button").text = "Show evaluation".to_string();
                        buttons.get_mut("perspective").expect("No perspective button").visible = true;
                        buttons.get_mut("main menu").expect("No menu button").visible = true;

                    }
                    else {
                        // revert to standard main menu

                        // clear all buttons
                        for (_, but) in &mut buttons {
                            but.visible = false;
                            but.shaded = false;
                        }

                        // set the standard buttons
                        buttons.get_mut("friend mode").expect("No friend button").visible = true;
                        buttons.get_mut("computer mode").expect("No computer button").visible = true;
                        buttons.get_mut("explore mode").expect("No explore button").visible = true;
                    }
                }

                draw_screen(&game_mode, &board_state, &squares, &perspective, &buttons, &piece_textures, &dummy_squares, &dummy_pieces);
            },
            GameMode::Friend|GameMode::Computer => {

                if is_mouse_button_pressed(MouseButton::Left) {

                    // catching the clicking of a chess square
                    let mut clicked_square: i8 = 64;

                    for file in 0..8 {
                        for rank in 0..8 {
                            if squares[file][rank].mouse_is_on() {
                                clicked_square = (rank * 8 + file) as i8;
                            }
                        }
                    }

                    // this only gets invoked if a square on the board was clicked
                    if clicked_square < 64 {

                        if let Some(old_square) = selected_square {

                            // deselect a square if it is clicked again
                            if clicked_square == old_square { 
                                squares[(clicked_square%8) as usize][(clicked_square/8) as usize].highlight = false;
                                selected_square = None;
                            }
                            else if game_mode == GameMode::Friend || board_state.side_to_move == player_side { 
                                // attempt to make the suggested move on the board if it is the player's move     

                                let suggested_move: Move;
                                if perspective == Side::White {
                                    suggested_move = coordinates_to_move(&mut board_state, old_square, clicked_square);
                                }
                                else {
                                    suggested_move = coordinates_to_move(&mut board_state, 63 - old_square, 63 - clicked_square);
                                }

                                if board_state.is_legal_move(&suggested_move) {

                                    board_state.apply_move_unchecked(&suggested_move);                               

                                    // update the analysis tree
                                    analysis_tree.make_given_move(ScoredMove{chess_move: suggested_move, score: 0.});

                                    // un-highlight all squares
                                    for file in 0..8 {
                                        for rank in 0..8 {
                                            squares[file as usize][rank as usize].highlight = false;
                                        }
                                    }
                                    
                                    // new highlighted squares
                                    squares[(old_square%8) as usize][(old_square/8) as usize].highlight = true;
                                    squares[(clicked_square%8) as usize][(clicked_square/8) as usize].highlight = true;

                                    selected_square = None;

                                    // set the checkpoint time for the computer
                                    computer_ref_time = get_time();
                                }
                                else {
                                    // un-highlight all squares
                                    for file in 0..8 {
                                        for rank in 0..8 {
                                            squares[file as usize][rank as usize].highlight = false;
                                        }
                                    }

                                    selected_square = None;
                                }
                                
                            }
                        }
                        else {
                            // a square can only become selected if it contains a piece of the right color
                            if 
                                perspective == Side::White && board_state.is_side_at(clicked_square, board_state.side_to_move) ||
                                perspective == Side::Black && board_state.is_side_at(63 - clicked_square, board_state.side_to_move)
                            {
                                squares[(clicked_square%8) as usize][(clicked_square/8) as usize].highlight = true;
                                selected_square = Some(clicked_square);
                            }
                        }
                    }
                }

                // after having handled all the clicking, we let the computer do some thinking
                if game_mode == GameMode::Friend {
                    let starting_time = get_time();
                    while get_time() - starting_time < COMPUTER_TIME_FRIEND && analysis_tree.root.visits < MAX_VISITS {
                        analysis_tree.traverse();
                    }
                }
                else if board_state.side_to_move == player_side {
                    let starting_time = get_time();
                    while get_time() - starting_time < COMPUTER_TIME_ON_PLAYER && analysis_tree.root.visits < MAX_VISITS {
                        analysis_tree.traverse();
                    }
                }
                else {

                    // if it is the computers turn, then we first let it do some thinking
                    
                    let starting_time = get_time();
                    while get_time() - starting_time < COMPUTER_TIME_ON_COMPUTER && analysis_tree.root.visits < MAX_VISITS {
                        analysis_tree.traverse();
                    }

                    // then, when it is time, we let the computer make a move.
                    if get_time() - computer_ref_time > COMPUTER_THINK {

                        // if the computer has a legal move to play, then do so
                        if let Some(suggested_move) = analysis_tree.get_best_move(&book) {
                            // make the move on the chess board
                            let old_square = suggested_move.get_from();
                            let new_square = suggested_move.get_to();

                            board_state.apply_move_unchecked(&suggested_move);

                            // update the analysis tree
                            println!("Visits: {}. ", analysis_tree.root.visits);
                            //analysis_tree.see_analysis();

                            analysis_tree.make_given_move(ScoredMove{chess_move: suggested_move, score: 0.});

                            // un-highlight all squares
                            for file in 0..8 {
                                for rank in 0..8 {
                                    squares[file as usize][rank as usize].highlight = false;
                                }
                            }
                                    
                            // new highlighted squares
                            if perspective == Side::White {
                                squares[(old_square%8) as usize][(old_square/8) as usize].highlight = true;
                                squares[(new_square%8) as usize][(new_square/8) as usize].highlight = true;
                            }
                            else {
                                squares[7 - (old_square%8) as usize][7 - (old_square/8) as usize].highlight = true;
                                squares[7 - (new_square%8) as usize][7 - (new_square/8) as usize].highlight = true;
                            }
                            
                        }
                        
                    }
                }
            },
            GameMode::Exploration => {
                
                // this game mode is similar to the player vs player game mode, 
                // but with the following differences
                // selecting a piece and outside of the board, the piece is removed
                // selecting a dummy piece to the side and clicking on a square on the board puts this piece here
                // one can give castling rights to either side to either player
                // one can give any pawn the property of having been moved two steps (allowing capture en-passant)
                
                if is_mouse_button_pressed(MouseButton::Left) {

                    // catching the clicking of a square on the board
                    let mut clicked_square: i8 = -1;

                    for file in 0..8 {
                        for rank in 0..8 {
                            if squares[file][rank].mouse_is_on() {
                                clicked_square = (rank * 8 + file) as i8;
                            }
                        }
                    }

                    // catching the clicking of one of the dummy pieces
                    for i in 0..2 {
                        for j in 0..5 {
                            if dummy_squares[i][j].mouse_is_on() {
                                clicked_square = (64 + 6*j + i) as i8;
                            }
                        }
                    }


                    // this only gets invoked if a square on the board was clicked
                    if clicked_square > -1 {

                        if let Some(old_square) = selected_square {

                            // deselect a square if it is clicked again
                            if clicked_square == old_square { 

                                if clicked_square < 64 {
                                    squares[(clicked_square%8) as usize][(clicked_square/8) as usize].highlight = false;
                                }
                                else {
                                    dummy_squares[((clicked_square - 64)%2) as usize][((clicked_square - 64)/6) as usize].highlight = false;
                                }
                                selected_square = None;
                            }
                            else { 

                                if old_square < 64 && clicked_square < 64 {

                                    // this amounts to a regular move being attempted
                                    // attempt to make the suggested move on the board if it is the player's move     
                                    let suggested_move: Move;

                                    if perspective == Side::White {
                                        suggested_move = coordinates_to_move(&mut board_state, old_square, clicked_square);
                                    }
                                    else {
                                        suggested_move = coordinates_to_move(&mut board_state, 63 - old_square, 63 - clicked_square);
                                    }

                                    if board_state.is_legal_move(&suggested_move) {
                                        // if this move is legal, apply it in the usual way
                                        board_state.apply_move_unchecked(&suggested_move);

                                        // update the analysis tree
                                        analysis_tree.make_given_move(ScoredMove{chess_move: suggested_move, score: 0.});

                                        // un-highlight all squares
                                        for file in 0..8 {
                                            for rank in 0..8 {
                                                squares[file as usize][rank as usize].highlight = false;
                                            }
                                        }
                                        
                                        // new highlighted squares
                                        squares[(old_square%8) as usize][(old_square/8) as usize].highlight = true;
                                        squares[(clicked_square%8) as usize][(clicked_square/8) as usize].highlight = true;

                                        selected_square = None;

                                        // set the checkpoint time for the computer
                                        computer_ref_time = get_time();
                                    }
                                    else {
                                        // if the move is not legal, then apply it as just a relocation
                                        if board_state.relocate_piece(suggested_move.get_from(), suggested_move.get_to()) {
                                            analysis_tree.reset_to(&board_state);
                                        }

                                        // un-highlight all squares
                                        for file in 0..8 {
                                            for rank in 0..8 {
                                                squares[file as usize][rank as usize].highlight = false;
                                            }
                                        }

                                        for i in 0..2 {
                                            for j in 0..5 {
                                                dummy_squares[i as usize][j as usize].highlight = false;
                                            }
                                        }

                                        selected_square = None;
                                    }
                                }
                                else if old_square > 63 && clicked_square < 64 {
                                    // this means that a dummy square should be placed on the board
                                    match perspective {
                                        Side::White => board_state.add_piece(clicked_square, Some(dummy_pieces[((old_square - 64)%2) as usize][((old_square - 64)/6) as usize])),
                                        Side::Black => board_state.add_piece(63 - clicked_square, Some(dummy_pieces[((old_square - 64)%2) as usize][((old_square - 64)/6) as usize])),
                                    };
                                    
                                    // and the analysis should be restarted
                                    analysis_tree.reset_to(&board_state);
                                    
                                    // un-highlight all squares
                                    for file in 0..8 {
                                        for rank in 0..8 {
                                            squares[file as usize][rank as usize].highlight = false;
                                        }
                                    }

                                    selected_square = None;

                                    // un-highlight all squares
                                    for file in 0..8 {
                                        for rank in 0..8 {
                                            squares[file as usize][rank as usize].highlight = false;
                                        }
                                    }

                                    for i in 0..2 {
                                        for j in 0..5 {
                                            dummy_squares[i as usize][j as usize].highlight = false;
                                        }
                                    }

                                    // set the checkpoint time for the computer
                                    computer_ref_time = get_time();
                                    
                                } 
                                else if old_square < 63 && clicked_square > 64{
                                    // this means that a board square was clicked and then a dummy square,
                                    // in this case we delete this piece from the board
                                    board_state.add_piece(old_square, None);
                                    analysis_tree.reset_to(&board_state);

                                    // un-highlight all squares
                                    for file in 0..8 {
                                        for rank in 0..8 {
                                            squares[file as usize][rank as usize].highlight = false;
                                        }
                                    }

                                    for i in 0..2 {
                                        for j in 0..5 {
                                            dummy_squares[i as usize][j as usize].highlight = false;
                                        }
                                    }

                                    selected_square = None;


                                }
                                else {
                                    // this means that one, and then another dummy square has been selected.
                                    // in this case we should just highlight the new and un-highlight the old

                                    // un-highlight all squares
                                    for file in 0..8 {
                                        for rank in 0..8 {
                                            squares[file as usize][rank as usize].highlight = false;
                                        }
                                    }

                                    for i in 0..2 {
                                        for j in 0..5 {
                                            dummy_squares[i as usize][j as usize].highlight = false;
                                        }
                                    }

                                    // and highlight the new
                                    selected_square = Some(clicked_square);
                                    dummy_squares[((clicked_square - 64)%2) as usize][((clicked_square - 64)/6) as usize].highlight = true;
                                }
                                
                            }
                        }
                        else {
                            if clicked_square < 64 {
                                // a square can only become selected if it contains a piece of the right color
                                if 
                                    perspective == Side::White && board_state.is_side_at(clicked_square, board_state.side_to_move) ||
                                    perspective == Side::Black && board_state.is_side_at(63 - clicked_square, board_state.side_to_move)
                                {
                                    squares[(clicked_square%8) as usize][(clicked_square/8) as usize].highlight = true;
                                    selected_square = Some(clicked_square);
                                }
                            }
                            else {
                                dummy_squares[((clicked_square - 64)%2) as usize][((clicked_square - 64)/6) as usize].highlight = true;
                                selected_square = Some(clicked_square);
                            }
                        }
                    }
                }

                // after having handled all the clicking, we let the computer do some thinking
                
                let starting_time = get_time();
                while get_time() - starting_time < COMPUTER_TIME_FRIEND && analysis_tree.root.visits < MAX_VISITS {
                    analysis_tree.traverse();
                }
                
            },
        }
        

        // showing the evaluation if it is to be shown
        if buttons["evaluation"].text != "Show evaluation".to_string() {
            if let Some(m) = analysis_tree.get_best_move(&book) {

                let evaluation_text = if let Some(solved) = analysis_tree.root.solved {
                    match solved {
                        Solved::Draw => "0.00",
                        Solved::Win(winner) if winner == Side::White => "W",
                        _ => "B"
                    }
                }
                else {
                    &((mc_to_eval(analysis_tree.root.combined_value()) * 100.).round()/100.).to_string()
                };


                buttons.get_mut("evaluation").expect("No evaluation button").text = 
                    m.to_text() + 
                    " : " + 
                    evaluation_text + 
                    " ( " + 
                    &analysis_tree.root.get_depth().to_string() +  
                    " )";
            }
            else {
                buttons.get_mut("evaluation").expect("No evaluation button").text = " -- ".to_string();
            }
        } 

        // showing the result of the game if there are no legal moves left
        if board_state.get_legal_moves().len() == 0 {
            buttons.get_mut("result window").expect("No result window").visible = true;

            if board_state.is_in_check(Side::White) {
                buttons.get_mut("result window").expect("No result window").text = "Black wins".to_string();
            }
            else if board_state.is_in_check(Side::Black) {
                buttons.get_mut("result window").expect("No result window").text = "White wins".to_string();
            }
            else {
                buttons.get_mut("result window").expect("No result window").text = "Draw".to_string();
            }
        }

        // modify the select side button
        match board_state.side_to_move {
            Side::White => {
                buttons.get_mut("select side").expect("No select side button").text = "White to move".to_string();
                buttons.get_mut("select side").expect("No select side button").color = WHITE;
                buttons.get_mut("select side").expect("No select side button").text_color = BLACK;
            },
            Side::Black => {
                buttons.get_mut("select side").expect("No select side button").text = "Black to move".to_string();
                buttons.get_mut("select side").expect("No select side button").color = BLACK;
                buttons.get_mut("select side").expect("No select side button").text_color = WHITE;
            }
        }

        // draw the state of affairs
        draw_screen(&game_mode, &board_state, &squares, &perspective, &buttons, &piece_textures, &dummy_squares, &dummy_pieces);
        next_frame().await
    }
}


// Testing for legality
fn perft(board: &mut BoardState, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let legal_moves = board.get_legal_moves();
    let mut nodes = 0;

    for m in legal_moves {

        let unmove = board.apply_move_unchecked(&m);

        nodes += perft(board, depth - 1);
        assert!(board.un_move_unchecked(unmove), "Failed to unmake move: {:?}",m);
    }
    nodes
}

/*
fn kiwipete() -> BoardState {
    let mut res: BoardState = BoardState {
        piece_arr: [None; 64],
        side_to_move: Side::White,
        white_short_castle: true,
        white_long_castle: true,
        black_short_castle: true,
        black_long_castle: true,
        en_passant: None,
        white_king: 60,
        black_king: 4,
        material: 0.,
        total_pieces: 32,
    };

    // pieces for white
    res.piece_arr[48] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[49] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[50] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[27] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[36] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[53] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[54] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[55] = Some(Piece{side: Side::White, kind: PieceKind::Pawn});
    res.piece_arr[42] = Some(Piece{side: Side::White, kind: PieceKind::Knight});
    res.piece_arr[28] = Some(Piece{side: Side::White, kind: PieceKind::Knight});
    res.piece_arr[51] = Some(Piece{side: Side::White, kind: PieceKind::Bishop});
    res.piece_arr[52] = Some(Piece{side: Side::White, kind: PieceKind::Bishop});
    res.piece_arr[56] = Some(Piece{side: Side::White, kind: PieceKind::Rook});
    res.piece_arr[63] = Some(Piece{side: Side::White, kind: PieceKind::Rook});
    res.piece_arr[45] = Some(Piece{side: Side::White, kind: PieceKind::Queen});
    res.piece_arr[60] = Some(Piece{side: Side::White, kind: PieceKind::King});

    // pieces for black
    res.piece_arr[8] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[33] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[10] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[11] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[20] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[13] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[22] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[47] = Some(Piece{side: Side::Black, kind: PieceKind::Pawn});
    res.piece_arr[17] = Some(Piece{side: Side::Black, kind: PieceKind::Knight});
    res.piece_arr[21] = Some(Piece{side: Side::Black, kind: PieceKind::Knight});
    res.piece_arr[16] = Some(Piece{side: Side::Black, kind: PieceKind::Bishop});
    res.piece_arr[14] = Some(Piece{side: Side::Black, kind: PieceKind::Bishop});
    res.piece_arr[0] = Some(Piece{side: Side::Black, kind: PieceKind::Rook});
    res.piece_arr[7] = Some(Piece{side: Side::Black, kind: PieceKind::Rook});
    res.piece_arr[12] = Some(Piece{side: Side::Black, kind: PieceKind::Queen});
    res.piece_arr[4] = Some(Piece{side: Side::Black, kind: PieceKind::King});

    return res
}

*/

// Drawing on the screen

fn draw_screen(
    game_mode: &GameMode,
    board_state: &BoardState, 
    squares: &[[Square;8];8], 
    perspective: &Side, 
    buttons: &HashMap<&str, Button>,
    piece_textures: &ChessTextures,
    dummy_squares: &[[Square;5];2],
    dummy_pieces: &[[Piece;5];2],
) {

    // only draw a chess board if we are not in the main menu
    if *game_mode != GameMode::MainMenu {
        draw_chess_board(&board_state, &squares, &piece_textures, &perspective);
    }

    // draw the buttons of the state
    draw_buttons(buttons);

    // only draw the dummy pieces and their dummy squares if we are in the explore mode
    if *game_mode == GameMode::Exploration {

        let offset_x = SMALL_BOARD_X * screen_width() - 2.0 * SQUARE_SIZE * SMALL_SCALE;
        let offset_y = SMALL_BOARD_Y * screen_height() - 5.0 * SQUARE_SIZE * SMALL_SCALE;

        // draw the squares
        for i in 0..2 {
            for j in 0..5 {
                draw_rectangle(
                    offset_x + (i as f32) * SQUARE_SIZE * SMALL_SCALE,
                    offset_y + (j as f32) * SQUARE_SIZE * SMALL_SCALE, 
                    dummy_squares[i as usize][j as usize].side_length, 
                    dummy_squares[i as usize][j as usize].side_length, 
                    dummy_squares[i as usize][j as usize].get_color()
                );
            }
        }

        // draw all the pieces
        for i in 0..2 {
            for j in 0..5 {

                draw_texture_ex(texture(&dummy_pieces[i][j], piece_textures), 
                                        offset_x + (i as f32) * SQUARE_SIZE * SMALL_SCALE+ (dummy_squares[i as usize][j as usize].side_length - PIECE_SIZE * SMALL_SCALE)/2., 
                                        offset_y + (j as f32) * SQUARE_SIZE * SMALL_SCALE + (dummy_squares[i as usize][j as usize].side_length - PIECE_SIZE * SMALL_SCALE)/2.,
                                        WHITE,
                                        DrawTextureParams{
                                            dest_size: Some(vec2(PIECE_SIZE * SMALL_SCALE, PIECE_SIZE * SMALL_SCALE)),
                                            ..Default::default()
                                        });
            }
        }

    }
}

fn draw_buttons(buttons: &HashMap<&str, Button>) {
    for (_, but) in buttons {
        but.draw();
    }
}

fn draw_chess_board(
    board_state: &BoardState, 
    squares: &[[Square;8];8], 
    piece_textures: &ChessTextures,
    perspective: &Side,
) {
    // the empty board is the same from both persepctives
    draw_empty_board(squares);
    
    let offset_x: f32 = screen_width()/2.0 - 4.0 * SQUARE_SIZE + (SQUARE_SIZE - PIECE_SIZE)/2.0;
    let offset_y: f32 = screen_height()/2.0 - 4.0 * SQUARE_SIZE + (SQUARE_SIZE - PIECE_SIZE)/2.0;

    // draw all the pieces, with respect to perspective
    
    for i in 0..64 {
        if let Some(piece) = board_state.piece_arr[i] {
            let sq = i as i8;
            let mut p_x: f32 = (sq%8) as f32;
            let mut p_y: f32 = (sq/8) as f32;

            if *perspective == Side::Black {
                p_x = 7. - p_x;
                p_y = 7. - p_y;
            }

            draw_texture_ex(texture(&piece, 
                                    piece_textures), 
                                    offset_x + p_x * SQUARE_SIZE, 
                                    offset_y + p_y * SQUARE_SIZE, 
                                    WHITE,
                                    DrawTextureParams{
                                        dest_size: Some(vec2(PIECE_SIZE, PIECE_SIZE)),
                                        ..Default::default()
                                    });
        }
    }
}

fn draw_empty_board(squares: &[[Square;8];8]) {

    let offset_x = screen_width()/2.0 - 4.0 * SQUARE_SIZE;
    let offset_y = screen_height()/2. - 4.0 * SQUARE_SIZE;

    for file in 0..8 {
        for rank in 0..8 {
            draw_rectangle(
                offset_x + (file as f32) * SQUARE_SIZE,
                offset_y + (rank as f32) * SQUARE_SIZE,
                squares[file][rank].side_length, 
                squares[file][rank].side_length, 
                squares[file][rank].get_color()
            );
        }
    }
}

#[derive(PartialEq)]
enum GameMode {
    MainMenu,
    Friend,
    Computer,
    Exploration,
}

// -------------------------------------
// Handling the squares of the board

#[derive(Clone, Copy, Debug)]
struct Square {
    file: i8,
    rank: i8,
    side_length: f32,
    color: Color,
    highlight_color: Color,
    highlight: bool,
}

impl Square {

    // checks if the mouse is on a square
    fn mouse_is_on(&self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();

        if self.file < 8 && self.rank < 8 {
            let offset_x = 0.5 * screen_width() - 4.0 * SQUARE_SIZE;
            let offset_y = 0.5 * screen_height() - 4.0 * SQUARE_SIZE;
            return 
                offset_x + (self.file as f32) * self.side_length < mouse_x && 
                offset_y + (self.rank as f32) * self.side_length < mouse_y && 
                offset_x + ((self.file + 1) as f32) * self.side_length > mouse_x && 
                offset_y + ((self.rank + 1) as f32) * self.side_length > mouse_y;
        }
        else{
            let offset_x = SMALL_BOARD_X * screen_width() - 2.0 * SQUARE_SIZE * SMALL_SCALE;
            let offset_y = SMALL_BOARD_Y * screen_height() - 5.0 * SQUARE_SIZE * SMALL_SCALE;
            return 
                offset_x + ((self.file - 8) as f32)*self.side_length < mouse_x && 
                offset_y + ((self.rank - 8) as f32)*self.side_length < mouse_y && 
                offset_x + ((self.file - 7) as f32)*self.side_length > mouse_x && 
                offset_y + ((self.rank - 7) as f32)*self.side_length > mouse_y;
        }
    }

    fn get_color(&self) -> Color {
        if self.highlight {
            return self.highlight_color.clone()
        }
        else {
            return self.color.clone();
        }
    }
}

// -------------------------------------
// buttons

struct Button {
    relative_x: f32,
    relative_y: f32,
    width: f32,
    height: f32,
    text: String,
    text_size: u16,
    text_centered: bool,
    color: Color,
    text_color: Color,
    visible: bool,
    shaded: bool,
}

impl Button{
    
    fn draw(&self) {

        if self.visible {
            let corner_x: f32 = self.relative_x * screen_width();
            let corner_y: f32 = self.relative_y * screen_height();
            let eps: f32 = 0.05;
            let delta: f32 = eps * self.width.min(self.height);
            let color: Color = if self.shaded {
                    Color{
                        r: 0.5 + self.color.r/2., 
                        g: 0.5 + self.color.g/2., 
                        b: 0.5 + self.color.b/2., 
                        a: self.color.a}
                }
                else {
                    self.color
                };

            // draw the first rectangle
            draw_rectangle(corner_x + delta,
                        corner_y, 
                        self.width - 2.*delta, 
                        self.height, 
                        color);

            // draw the second rectangle
            draw_rectangle(corner_x,
                        corner_y + delta, 
                        self.width, 
                        self.height - 2.*delta, 
                        color);
            
            // draw the four corner circles
            draw_circle(corner_x + delta, corner_y + delta, delta, color);
            draw_circle(corner_x + self.width - delta, corner_y + delta, delta, color);
            draw_circle(corner_x + self.width - delta, corner_y + self.height - delta, delta, color);
            draw_circle(corner_x + delta, corner_y + self.height - delta, delta, color);
            
            // write the desired text
            let dimensions = measure_text(&self.text, None, self.text_size, 1.0);
            if self.text_centered {
                draw_text(
                    &self.text,
                    corner_x + (self.width - dimensions.width)/2.,
                    corner_y + self.height/2. + dimensions.height/3.,
                    self.text_size as f32,
                    self.text_color,
                );
            }
            else {
                draw_text(
                    &self.text,
                    corner_x + self.width/12.,
                    corner_y + self.height/2. + dimensions.height/3.,
                    self.text_size as f32,
                    self.text_color,
                );
            }
            
        }
    }

    fn mouse_is_on(&self) -> bool {
        let (mouse_x, mouse_y) = mouse_position();

        return 
            self.visible &&
            !self.shaded &&
            self.relative_x * screen_width() < mouse_x && 
            self.relative_y * screen_height() < mouse_y && 
            self.relative_x * screen_width() + self.width > mouse_x &&
            self.relative_y * screen_height() + self.height > mouse_y;

    }

}

fn spawn_friend_mode_button() -> Button {
    Button {
        relative_x: 0.1,
        relative_y: 0.2,
        width: 420.,
        height: 80.,
        text: String::from("Play against a friend"),
        text_size: FONT_SIZE,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_evaluation_button() -> Button {
    Button {
        relative_x: 0.01,
        relative_y: 0.75,
        width: 200.,
        height: 50.,
        text: String::from(""),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_computer_mode_button() -> Button {
    Button {
        relative_x: 0.1,
        relative_y: 0.4,
        width: 420.,
        height: 80.,
        text: String::from("Play against a bot"),
        text_size: FONT_SIZE,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_black_mode_button() -> Button {
    Button {
        relative_x: 0.5,
        relative_y: 0.6,
        width: 420.,
        height: 80.,
        text: String::from("Play as black"),
        text_size: FONT_SIZE,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_random_mode_button() -> Button {
    Button {
        relative_x: 0.5,
        relative_y: 0.4,
        width: 420.,
        height: 80.,
        text: String::from("Play random color"),
        text_size: FONT_SIZE,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_white_mode_button() -> Button {
    Button {
        relative_x: 0.5,
        relative_y: 0.2,
        width: 420.,
        height: 80.,
        text: String::from("Play as white"),
        text_size: FONT_SIZE,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_explore_mode_button() -> Button {
    Button {
        relative_x: 0.1,
        relative_y: 0.6,
        width: 420.,
        height: 80.,
        text: String::from("Explore and analyze"),
        text_size: FONT_SIZE,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_go_to_main_button() -> Button { 
    Button {
        relative_x: 0.01,
        relative_y: 0.85,
        width: 200.,
        height: 50.,
        text: String::from("Go to main menu"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_change_perspective_button() -> Button {
    Button {
        relative_x: 0.01,
        relative_y: 0.65,
        width: 200.,
        height: 50.,
        text: String::from("Change perspective"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_clear_board_button() -> Button {
    Button {
        relative_x: 0.01,
        relative_y: 0.55,
        width: 200.,
        height: 50.,
        text: String::from("Clear board"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_reset_board_button() -> Button {
    Button {
        relative_x: 0.01,
        relative_y: 0.35,
        width: 200.,
        height: 50.,
        text: String::from("Reset board"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_show_analysis_button() -> Button {
    Button {
        relative_x: 0.01,
        relative_y: 0.25,
        width: 200.,
        height: 50.,
        text: String::from("Show analysis"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_select_side_button() -> Button {
    Button {
        relative_x: 0.01,
        relative_y: 0.45,
        width: 200.,
        height: 50.,
        text: String::from("White to move"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: WHITE,
        text_color: BLACK,
        visible: false,
        shaded: false,
    }
}

fn spawn_game_winner_button() -> Button {
    Button {
        relative_x: 0.35,
        relative_y: 0.45,
        width: 300.,
        height: 100.,
        text: String::from(""),
        text_size: FONT_SIZE,
        text_centered: true,
        color: Color::new(200.0/256.0, 200.0/256.0, 200.0/256.0, 0.6),
        text_color: Color::new(0.0/256.0, 0.0/256.0, 0.0/256.0, 1.),
        visible: false,
        shaded: false,
    }
}

fn spawn_wsc_button() -> Button {
    Button {
        relative_x: 0.805,
        relative_y: 0.55,
        width: 200.,
        height: 50.,
        text: String::from("White short castle"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_wlc_button() -> Button {
    Button {
        relative_x: 0.805,
        relative_y: 0.65,
        width: 200.,
        height: 50.,
        text: String::from("White long castle"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_bsc_button() -> Button {
    Button {
        relative_x: 0.805,
        relative_y: 0.75,
        width: 200.,
        height: 50.,
        text: String::from("Black short castle"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

fn spawn_blc_button() -> Button {
    Button {
        relative_x: 0.805,
        relative_y: 0.85,
        width: 200.,
        height: 50.,
        text: String::from("Black long castle"),
        text_size: FONT_SIZE/2,
        text_centered: true,
        color: BUTTON_COLOR,
        text_color: BUTTON_TEXT_COLOR,
        visible: false,
        shaded: false,
    }
}

// -------------------------------------
// the below are helper things for drawing the chess board's state

struct ChessTextures {
    black_pawn_texture: Texture2D, 
    white_pawn_texture: Texture2D,
    black_knight_texture: Texture2D, 
    white_knight_texture: Texture2D,
    black_bishop_texture: Texture2D, 
    white_bishop_texture: Texture2D,
    black_rook_texture: Texture2D, 
    white_rook_texture: Texture2D,
    black_queen_texture: Texture2D, 
    white_queen_texture: Texture2D,
    black_king_texture: Texture2D, 
    white_king_texture: Texture2D,
}

fn texture<'a>(piece: &'a Piece, textures: &'a ChessTextures) -> &'a Texture2D {
    match (piece.side, piece.kind) {
        (Side::White, PieceKind::Pawn) => &textures.white_pawn_texture,
        (Side::Black, PieceKind::Pawn) => &textures.black_pawn_texture,
        (Side::White, PieceKind::Bishop) => &textures.white_bishop_texture,
        (Side::Black, PieceKind::Bishop) => &textures.black_bishop_texture,
        (Side::White, PieceKind::Knight) => &textures.white_knight_texture,
        (Side::Black, PieceKind::Knight) => &textures.black_knight_texture,
        (Side::White, PieceKind::Rook) => &textures.white_rook_texture,
        (Side::Black, PieceKind::Rook) => &textures.black_rook_texture,
        (Side::White, PieceKind::Queen) => &textures.white_queen_texture,
        (Side::Black, PieceKind::Queen) => &textures.black_queen_texture,
        (Side::White, PieceKind::King) => &textures.white_king_texture,
        (Side::Black, PieceKind::King) => &textures.black_king_texture,
    }
}
