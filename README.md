Chess Engine in Rust

A chess program written from scratch in Rust, with a built-in engine based on Monte Carlo Tree Search.

<img width="1396" height="786" alt="ChessScreenshot" src="https://github.com/user-attachments/assets/a6ec14e3-9b60-4561-8dc3-9f0618ebb6d1" />

▶ Features
Player vs. player – play locally against a friend
Player vs. computer – play against the built-in engine
Analysis mode – explore positions and see what the engine recommends

▶ Skills and experience from this project# Chess Engine in Rust

A chess program written from scratch in Rust, with a built-in engine based on Monte Carlo Tree Search.

**▶ Play it in your browser: [benjaminmecogit.github.io/chess_engine](https://benjaminmecogit.github.io/chess_engine/)**

<!-- TODO: add a screenshot or short GIF of the GUI here, e.g. ![Screenshot](docs/screenshot.png) -->

## Features

- **Player vs. player** – play locally against a friend
- **Player vs. computer** – play against the built-in engine
- **Analysis mode** – explore positions and see what the engine recommends

## Skills and experience from this project

| Area | What I did |
|---|---|
| **Rust** | Built a large, multi-component application in Rust: board representation, move generation, game-state management, search and GUI |
| **Algorithms** | Implemented Monte Carlo Tree Search with UCB-based tree traversal from scratch |
| **Machine learning** | Building NNUE evaluation networks and a genetic algorithm to train them, without ML libraries (in progress) |
| **Software design** | Refactored the architecture several times as requirements changed, aiming for code that is modular, readable, fast and easy to extend |
| **Performance** | Designed data structures and search for speed, since an engine must evaluate a very large number of positions quickly |
| **Working with AI tools** | Used AI for discussion and research, while writing almost all of the code myself |

<!-- TODO (only if true): add a row for testing, e.g. "Testing | Verified move generation with perft tests; N unit tests run with cargo test" -->

## Goals of the project

1. **Become proficient in Rust.**
2. **Learn search and machine-learning techniques by implementing them from scratch**, including Monte Carlo Tree Search, Efficiently Updatable Neural Networks (NNUE) and genetic algorithms.
3. **Gain experience developing a large project with many interdependent parts.**

## Why Rust?

A chess engine has to analyse a very large number of positions quickly while managing complex data structures. That makes Rust a natural fit: it offers performance comparable to C and C++ together with guaranteed memory safety. Rust is also being adopted in more and more areas, and code rewritten from C/C++ into Rust often becomes safer and more stable, and sometimes faster. Building this project was my way of learning the language properly on a real, non-trivial codebase.

## The engine

### Monte Carlo Tree Search (MCTS)

MCTS searches a decision tree, such as a game tree, and its recommendation improves the longer it is allowed to run. In each iteration it selects the most promising leaf node (using an upper-confidence-bound rule, which can be tuned in many ways) and expands the tree from there.

I chose MCTS over a classical minimax search for three reasons:

1. **Anytime behaviour.** The engine often doesn't know how long it will get to think (for example while the opponent is thinking), so it is valuable that every iteration of search improves the result.
2. **Selective search.** There is no need to explore the whole game tree. Clearly bad moves should be set aside early.
3. **Broad applicability.** MCTS is used for many decision-making problems beyond chess, and I wanted to understand it by implementing it myself.

### Neural network evaluation (in progress)

Search algorithms benefit from neural networks in two main ways:

1. **Move ordering:** ranking the legal moves in a position so the most promising ones are searched first.
2. **Position evaluation:** judging how good a position is while searching the game tree.

I am currently building my own implementation of **Efficiently Updatable Neural Networks (NNUE)** with the aim of training them with a **genetic algorithm** through self-play. NNUEs are shallow networks that can be evaluated extremely quickly, because they update incrementally as moves are made, and they train faster than deeper networks. Strong engines such as Stockfish use this architecture, so it was a reasonable choice for my first attempt.

## Lessons in software design

A chess program has many design questions. How should the GUI look? How should the game loop flow? How should positions be represented internally? How should the computer think? I often tried one approach and later found that another worked, looked or performed better. Reworking the code each time taught me to write a codebase that:

- is **modular**, so one part can change without breaking the others,
- stays **flexible** for ideas I haven't had yet,
- is **conceptually clear and simple** to read,
- and is still **fast and memory safe**.

## How I used AI

Almost all of the code is written by me, with a few small exceptions where I had help from AI. I used friends and AI as discussion partners for ideas and information. I've found that for me, the most productive way to work both for the quality of what I build and for how much I learn, is to use AI for inspiration and research while writing the bulk of the code myself.

## Roadmap

- [ ] Train NNUEs through self-play using a genetic algorithm
- [ ] Train small neural networks on evaluations from a strong engine (e.g. Stockfish)

<!-- TODO: add a "Build and run locally" section, e.g.:
## Running locally
```bash
git clone https://github.com/BenjaminMecoGit/chess_engine.git
cd chess_engine
cargo run --release
```
-->

## Author

**Benjamin Meco** – PhD in mathematics, Uppsala University
[GitHub](https://github.com/BenjaminMecoGit)


Area  What I did
Rust	Built a large, multi-component application in Rust: board representation, move generation, game-state management, search and GUI
Algorithms	Implemented Monte Carlo Tree Search with UCB-based tree traversal from scratch
Machine learning	Building NNUE evaluation networks and a genetic algorithm to train them, without ML libraries (in progress)
Software design	Refactored the architecture several times as requirements changed, aiming for code that is modular, readable, fast and easy to extend
Performance	Designed data structures and search for speed, since an engine must evaluate a very large number of positions quickly
Working with AI tools	Used AI for discussion and research, while writing almost all of the code myself
<!-- TODO (only if true): add a row for testing, e.g. "Testing | Verified move generation with perft tests; N unit tests run with cargo test" -->
Goals of the project
Become proficient in Rust.
Learn search and machine-learning techniques by implementing them from scratch, including Monte Carlo Tree Search, Efficiently Updatable Neural Networks (NNUE) and genetic algorithms.
Gain experience developing a large project with many interdependent parts.
Why Rust?

A chess engine has to analyse a very large number of positions quickly while managing complex data structures. That makes Rust a natural fit: it offers performance comparable to C and C++ together with guaranteed memory safety. Rust is also being adopted in more and more areas, and code rewritten from C/C++ into Rust often becomes safer and more stable, and sometimes faster. Building this project was my way of learning the language properly on a real, non-trivial codebase.

The engine
Monte Carlo Tree Search (MCTS)

MCTS searches a decision tree, such as a game tree, and its recommendation improves the longer it is allowed to run. In each iteration it selects the most promising leaf node (using an upper-confidence-bound rule, which can be tuned in many ways) and expands the tree from there.

I chose MCTS over a classical minimax search for three reasons:

Anytime behaviour. The engine often doesn't know how long it will get to think (for example while the opponent is thinking), so it is valuable that every iteration of search improves the result.
Selective search. There is no need to explore the whole game tree. Clearly bad moves should be set aside early.
Broad applicability. MCTS is used for many decision-making problems beyond chess, and I wanted to understand it by implementing it myself.
Neural network evaluation (in progress)

Search algorithms benefit from neural networks in two main ways:

Move ordering: ranking the legal moves in a position so the most promising ones are searched first.
Position evaluation: judging how good a position is while searching the game tree.

I am currently building my own implementation of Efficiently Updatable Neural Networks (NNUE) and training them with a genetic algorithm through self-play. NNUEs are shallow networks that can be evaluated extremely quickly, because they update incrementally as moves are made, and they train faster than deeper networks. Strong engines such as Stockfish use this architecture, so it was a reasonable choice for my first attempt.

Lessons in software design

A chess program has many design questions. How should the GUI look? How should the game loop flow? How should positions be represented internally? How should the computer think? I often tried one approach and later found that another worked, looked or performed better. Reworking the code each time taught me to write a codebase that:

is modular, so one part can change without breaking the others,
stays flexible for ideas I haven't had yet,
is conceptually clear and simple to read,
and is still fast and memory safe.
How I used AI

Almost all of the code is written by me, with a few small exceptions where I had help from AI. I used friends and AI as discussion partners for ideas and information. I've found that the most productive way to work, both for the quality of what I build and for how much I learn, is to use AI for inspiration and research while writing the bulk of the code myself.

Roadmap
 Train NNUEs through self-play using a genetic algorithm
 Train small neural networks on evaluations from a strong engine (e.g. Stockfish)
<!-- TODO: add a "Build and run locally" section, e.g.: ## Running locally ```bash git clone https://github.com/BenjaminMecoGit/chess_engine.git cd chess_engine cargo run --release ``` -->
Author

Benjamin Meco – PhD in mathematics, Uppsala University GitHub
