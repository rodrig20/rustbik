use bytemuck::cast_slice;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Write;
use std::path::Path;

use bytemuck::{Pod, Zeroable};
use memmap2::Mmap;
use std::fs::File;
use std::io::{BufWriter, Result};

use super::{G1_MOVE_LIST, KociembaCube, TABLE_DIR};
use crate::MOVE_LIST;

/// Writes a slice of data to a binary file efficiently using a buffered writer
pub(crate) fn write_table<T>(data: &[T], path: impl AsRef<Path>) -> Result<()>
where
    T: Pod + Zeroable,
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(bytemuck::cast_slice(data))?;

    writer.flush()?;
    Ok(())
}

/// Memory-maps a file and returns a static reference to the data
pub(crate) fn map_file<T: bytemuck::Pod>(path: String) -> &'static [T] {
    let file = File::open(path).expect("Failed to open file");
    let mmap = unsafe { Mmap::map(&file).expect("Failed to map file") };

    let mmap_ref: &'static Mmap = Box::leak(Box::new(mmap));

    bytemuck::cast_slice(mmap_ref)
}
/// Checks if all necessary lookup tables for Kociemba's algorithm exist
fn check_g1_tables() -> bool {
    Path::new(&format!("{}/eo_co_uds_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/eo_move.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/co_move.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/uds_move.bin", TABLE_DIR)).exists()
}

/// Checks if all necessary G2 lookup tables exist
fn check_g2_tables() -> bool {
    Path::new(&format!("{}/ep_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/cp_uds_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/cp_move.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/ep_no_uds_move.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/ep_uds_move.bin", TABLE_DIR)).exists()
}

/// Generates Phase 1 (G0 to G1) lookup tables.
///
/// First, it precomputes the move transition tables for Edge Orientation (EO),
/// Corner Orientation (CO), and UD-slice coordinate spaces (the G1 coordinate set).
/// Then, it uses BFS (if the pruning table is missing) to generate the pruning table
/// that stores the minimum distance to G1, leveraging symmetry reduction to compress the state space.
fn gen_g1_tables() -> std::io::Result<()> {
    if !Path::new(TABLE_DIR).exists() {
        fs::create_dir_all(TABLE_DIR)?;
    }

    let mut eo_move = [0u16; 2048 * 18];
    let mut co_move = [0u16; 2187 * 18];
    let mut uds_move = [0u16; 495 * 18];
    // Precompute transition move tables for EO, CO, and UDS coordinates
    // These tables allow the solver to quickly determine the next state for any of the 18 moves.
    for eo in 0..2048 {
        let cube = KociembaCube::from_eo(eo as u16);
        for (m, mv) in MOVE_LIST.iter().enumerate() {
            let mut next = cube.clone();
            next.turn(mv); // Apply rotation
            eo_move[(eo as usize * 18) + m] = next.get_eo_coord(); // Store new coordinate
        }
    }

    for co in 0..2187 {
        let cube = KociembaCube::from_co(co as u16);
        for (m, mv) in MOVE_LIST.iter().enumerate() {
            let mut next = cube.clone();
            next.turn(mv); // Apply rotation
            co_move[(co as usize * 18) + m as usize] = next.get_co_coord(); // Store new coordinate
        }
    }

    for uds in 0..495 {
        let cube = KociembaCube::from_uds(uds as u16);
        for (m, mv) in MOVE_LIST.iter().enumerate() {
            let mut next = cube.clone();
            next.turn(mv); // Apply rotation
            uds_move[(uds as usize) * 18 + m as usize] = next.get_uds_coord(); // Store new coordinate
        }
    }

    // Save transition move tables
    write_table(&eo_move, format!("{}/eo_move.bin", TABLE_DIR))?;
    write_table(&co_move, format!("{}/co_move.bin", TABLE_DIR))?;
    write_table(&uds_move, format!("{}/uds_move.bin", TABLE_DIR))?;

    let co_sym_map: &[u16] = map_file(format!("{}/co_sym_map.bin", TABLE_DIR));
    let eo_uds_sym_move: &[u32] = map_file(format!("{}/eo_uds_sym_move.bin", TABLE_DIR));

    // BFS initialization: pruning_table stores distances (255 represents unvisited/infinity)
    let mut pruning_table: Box<[u8]> = vec![255u8; 64430 * 2187].into_boxed_slice();
    pruning_table[0] = 0;

    let mut queue = VecDeque::from([(0u32, 0u16)]);

    let mut visited_count = 1;
    let total_states = 64430 * 2187;

    // Explore state space layer-by-layer (BFS) to find shortest path to solved state
    'global: while let Some((curr_class, curr_co)) = queue.pop_front() {
        let curr_class_offset = curr_class as usize * 18;
        let curr_co_offset = curr_co as usize * 18;

        let curr_idx = (curr_class as usize * 2187) + curr_co as usize;
        let curr_dist = pruning_table[curr_idx];

        for i in 0..18 {
            let packed_move = eo_uds_sym_move[curr_class_offset + i];
            let next_class = packed_move >> 4;
            let sym = (packed_move & 0xF) as usize;
            if total_states == visited_count {
                break 'global;
            }
            let raw_next_co = co_move[curr_co_offset + i] as usize;
            let next_co = co_sym_map[raw_next_co * 16 + sym];

            let next_idx = (next_class as usize * 2187) + next_co as usize;

            // Mark unvisited state and queue for further exploration
            if pruning_table[next_idx] == 255 {
                pruning_table[next_idx] = curr_dist + 1;
                queue.push_back((next_class, next_co));
                visited_count += 1;
                if total_states == visited_count {
                    break 'global;
                }
            }
        }
    }

    write_table(&pruning_table, format!("{}/eo_co_uds_table.bin", TABLE_DIR))?;

    Ok(())
}

fn gen_g1_sym_tables() -> std::io::Result<()> {
    if !Path::new(TABLE_DIR).exists() {
        fs::create_dir_all(TABLE_DIR)?;
    }
    // Generate CO symmetry map: relates corner orientation to their transformed states under 16 symmetries
    {
        let mut co_sym_map = [0; 2187 * 16];
        for co in 0..2187u16 {
            let cube = KociembaCube::from_co(co as u16);
            for sym in 0..16 {
                let sym_cube = cube.apply_uds_symmetry(sym);

                co_sym_map[(co * 16 + sym as u16) as usize] = sym_cube.get_co_coord();
            }
        }

        write_table(&co_sym_map, format!("{}/co_sym_map.bin", TABLE_DIR))?;
    }
    // Generate EO+UDS symmetry mapping and move table: canonicalizes combined EO/UDS coordinates
    {
        let mut eo_uds_sym_map = [u32::MIN; 2048 * 495];
        let mut raw_to_compact_eo_uds: HashMap<(u16, u16), u32> = HashMap::new();
        let mut id_to_example_eo_uds = [(u16::MIN, u16::MIN); 64430];

        let mut next_id: u32 = 0;
        // Iterate through all possible EO and UDS states
        for eo in 0..2048 {
            for uds in 0..495 {
                let cube = KociembaCube::from_eo_uds(eo, uds);

                // Identify canonical representative using 16 symmetries and map to compressed state
                let mut canon_coords = (cube.get_eo_coord(), cube.get_uds_coord());
                let mut canon_sym = 0;

                // Find the lexicographically smallest (EO, UDS) state among all 16 symmetries
                for i in 1..16 {
                    let sym_cube = cube.apply_uds_symmetry(i);

                    let sym_eo = sym_cube.get_eo_coord();

                    if sym_eo < canon_coords.0 {
                        let sym_uds = sym_cube.get_uds_coord();
                        canon_coords = (sym_eo, sym_uds);
                        canon_sym = i;
                    } else if sym_eo == canon_coords.0 {
                        let sym_uds = sym_cube.get_uds_coord();
                        if sym_uds < canon_coords.1 {
                            canon_coords = (sym_eo, sym_uds);
                            canon_sym = i;
                        }
                    }
                }

                // Assign or retrieve a unique ID for the canonical state
                if let Some(id) = raw_to_compact_eo_uds.get(&canon_coords) {
                    eo_uds_sym_map[(eo as usize) * 495 + uds as usize] =
                        (id << 4) | canon_sym as u32;
                } else {
                    raw_to_compact_eo_uds.insert(canon_coords, next_id);
                    id_to_example_eo_uds[next_id as usize] = (eo, uds);

                    eo_uds_sym_map[(eo as usize) * 495 + uds as usize] =
                        (next_id << 4) | canon_sym as u32;

                    next_id += 1;
                }
            }
        }

        // Store the symmetry mapping for state lookup
        write_table(&eo_uds_sym_map, format!("{}/eo_uds_sym_map.bin", TABLE_DIR))?;

        // Precompute move transitions in the canonicalized state space
        let mut eo_uds_sym_move = vec![u32::MIN; 64430 * 18];

        // Calculate the next canonical state for every possible move from each canonical representative
        for (canon_id, (eo, uds)) in id_to_example_eo_uds.iter().enumerate() {
            let cube = KociembaCube::from_eo_uds(*eo, *uds);

            for (i, mv) in MOVE_LIST.iter().enumerate() {
                let mut n_cube = cube.clone();
                n_cube.turn(&mv);

                let n_eo = n_cube.get_eo_coord() as usize;
                let n_uds = n_cube.get_uds_coord() as usize;

                // Lookup the canonical state for the result of this move
                let n_canon = eo_uds_sym_map[(n_eo as usize) * 495 + n_uds as usize];

                eo_uds_sym_move[canon_id * 18 + i] = n_canon
            }
        }

        // Save the canonicalized transition move table
        write_table(
            &eo_uds_sym_move,
            format!("{}/eo_uds_sym_move.bin", TABLE_DIR),
        )?;
    }
    Ok(())
}

/// Generates phase 2 (G1 to solved) lookup tables using BFS
/// This phase only uses the 10 moves allowed in the G1 group:
/// U, U', U2, D, D', D2, F2, B2, R2, L2
fn gen_g2_tables() -> std::io::Result<()> {
    if !Path::new(TABLE_DIR).exists() {
        fs::create_dir_all(TABLE_DIR)?;
    }

    // Distances for CP (Corner Permutation) and EP (Edge Permutation)
    let mut ep_dists = [255u8; 40320 * 24];
    let mut cp_uds_dists = [255u8; 40320 * 24];

    // Move tables for phase 2 coordinates
    let mut cp_move = [[0u16; 10]; 40320];
    let mut ep_no_uds_move = [[0u16; 10]; 40320];
    let mut ep_uds_move = [[0u16; 10]; 24];

    let start_cube = KociembaCube::new();

    ep_dists[0] = 0;
    cp_uds_dists[0] = 0;

    let mut queue: VecDeque<(KociembaCube, usize, usize, usize)> = VecDeque::new();
    queue.push_back((start_cube, 0, 0, 0));

    while let Some((current, d_ep_no_uds, d_ep_uds, d_cp)) = queue.pop_front() {
        // Try only the 10 allowed G1 moves
        for (i, mv) in G1_MOVE_LIST.iter().enumerate() {
            let mut next = current.clone();
            next.turn(&mv);

            let n_ep_no_uds = next.get_ep_no_uds_coord() as usize;
            let n_ep_uds = next.get_ep_uds_coord() as usize;
            let n_cp = next.get_cp_coord() as usize;

            // Fill phase 2 move tables
            cp_move[d_cp][i] = n_cp as u16;
            ep_no_uds_move[d_ep_no_uds][i] = n_ep_no_uds as u16;
            ep_uds_move[d_ep_uds][i] = n_ep_uds as u16;

            let mut discovered = false;
            if ep_dists[(n_ep_uds * 40320) + n_ep_no_uds] == 255 {
                ep_dists[(n_ep_uds * 40320) + n_ep_no_uds] =
                    ep_dists[(d_ep_uds * 40320) + d_ep_no_uds] + 1;
                discovered = true;
            }
            if cp_uds_dists[(n_ep_uds * 40320) + n_cp] == 255 {
                cp_uds_dists[(n_ep_uds * 40320) + n_cp] =
                    cp_uds_dists[(d_ep_uds * 40320) + d_cp] + 1;
                discovered = true;
            }

            if discovered {
                queue.push_back((next, n_ep_no_uds, n_ep_uds, n_cp));
            }
        }
    }

    // Save phase 2 pruning tables
    write_table(&ep_dists, format!("{}/ep_table.bin", TABLE_DIR))?;
    write_table(&cp_uds_dists, format!("{}/cp_uds_table.bin", TABLE_DIR))?;

    // Save phase 2 move tables
    write_table(&cp_move, format!("{}/cp_move.bin", TABLE_DIR))?;
    write_table(&ep_no_uds_move, format!("{}/ep_no_uds_move.bin", TABLE_DIR))?;
    write_table(&ep_uds_move, format!("{}/ep_uds_move.bin", TABLE_DIR))?;

    Ok(())
}

/// Ensures all Kociemba lookup tables are present, generating them if necessary
pub fn gen_tables() -> std::io::Result<()> {
    gen_g1_sym_tables()?;
    if !check_g1_tables() {
        gen_g1_tables()?;
    }
    if !check_g2_tables() {
        gen_g2_tables()?;
    }

    Ok(())
}
