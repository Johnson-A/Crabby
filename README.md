# Overview
Crabby is an original UCI chess engine written in the [rust programming language](https://www.rust-lang.org/) as an experiment to learn a new language and explore chess programming.
I would greatly appreciate all feedback on my progress for both!

# Features
* Nega-Max alpha beta pruning
* Iterative deepening
* Quiescence Search
* Killer move heuristic
* Static exchange evaluation
* Transposition Table with Zobrist hashing
* Bitboard based representation
* Magic move generation

# Planned
* Null move pruning
* Aspiration window
* PVS/Negascout or MTD(f)
* Piece-square evaluation

# Extended UCI Commands
* ponder - search infinitely
* perft x - run perft to a depth x
* testperf - Search to a given depth in many positions to test performance
* testmove - Run perft on many positions to validate move generation

# Compiling
Many experimental features are currently used in Crabby, which will require a nightly version of the rust compiler.
I will hopefully remove many of these in the future as they become stable.
```sh
$ cargo build --release
```
Alternatively, as specified in the file *compile*, to make use of cpu features
```sh
$ cargo rustc --release -- -C target-feature=+popcnt -C target-cpu=haswell
```
# Terms
Crabby is licensed under the **GNU General Public License** (GPLv2)
