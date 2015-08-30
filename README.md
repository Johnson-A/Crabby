# Overview
Crabby is an original UCI chess engine written in the [rust programming language](https://www.rust-lang.org/).
Crabby gets its name from the rust mascot Ferris the crab.
This project is an experiment to learn a new language and explore chess programming.
I would greatly appreciate all feedback on my progress for both!

# Building
**Crabby currently requires a nightly version of the rust compiler**

```sh
git clone https://github.com/Johnson-A/Crabby.git
cd Crabby
cargo build --release
./target/release/crabby
```
Alternatively, as specified in the file [compile](compile), to make use of certain CPU features

```sh
cargo rustc --release -- -C target-feature=+popcnt -C target-cpu=native
```

# Features
* Nega-Max alpha beta pruning
* Iterative deepening
* PVS
* Quiescence Search
* Null move pruning
* Late Move Reduction
* Killer move heuristic
* Static exchange evaluation
* Transposition Table with Zobrist hashing
* Bitboard based representation
* Magic move generation

# Planned
* Aspiration window
* Time manager
* PVS/Negascout or MTD(f) -> improvements
* Piece-square evaluation
* Evaluation -> improvements
* Improve Principal variation extraction
* 50 move rule

# Extended UCI Commands
* perft x - run perft to a depth x
* testmove - Run perft on many positions to validate move generation
* testperf - Search to a given depth in many positions to test performance

# Terms
Crabby is licensed under the **GNU General Public License** (GPLv2) as specified in the [LICENSE](LICENSE)
