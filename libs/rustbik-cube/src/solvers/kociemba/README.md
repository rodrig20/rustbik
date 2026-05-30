# Kociemba Solver

This module implements Herbert Kociemba's two-phase algorithm for solving the 3x3 Rubik's Cube. It provides a high-performance solver that typically finds solutions under 20 moves.

## How it Works

The solver works by dividing the problem into two distinct phases:

### Phase 1: G0 -> G1
In this phase, the cube is transitioned from any initial state ($G0$) into the $G1$ group. The $G1$ group is defined by:
- Correct orientation of all 12 edges (EO).
- Correct orientation of all 8 corners (CO).
- The 4 edges of the middle slice (UD-slice) are in their home slice, though not necessarily in their correct positions or orientations.

**Allowed Moves in Phase 1**: All 18 standard moves ($U, D, R, L, F, B$ in all 3 directions).

### Phase 2: G1 -> Solved
Once in the $G1$ group, the cube is solved using only a subset of moves that preserve the $G1$ properties.
- **Allowed Moves in Phase 2**: $U, D, F2, B2, R2, L2$.

## Lookup Tables

To achieve high performance, the solver relies on precomputed lookup tables stored in `libs/rustbik-cube/tables`.

- **Move Tables**: Precompute how coordinates change for every possible move.
- **Pruning Tables**: Store the minimum number of moves required to reach the goal state for each coordinate (using Breadth-First Search).

### Generating Tables
If the lookup tables are missing, they can be generated using the provided utility:

```bash
cargo run --bin generate_tables
```

> **Note**: Generating all tables may take a few moments and requires approximately 100MB of disk space.

## Performance Modes

The solver behavior can be adjusted in `constants.rs`:

- **Standard Mode**: Loads all move tables into RAM for maximum speed during search.
- **Low-Memory Mode**: Recalculates state transitions on-the-fly. This significantly reduces RAM usage but increases the time required to find a solution.

## Implementation Details

- `utils.rs`: Coordinate calculations and the `KociembaCube` wrapper.
- `tables.rs`: Logic for BFS exploration and table generation.
- `solver.rs`: IDA* (Iterative Deepening A*) search implementation for both phases.
