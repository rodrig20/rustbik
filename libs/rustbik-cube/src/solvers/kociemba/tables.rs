use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::{G1_MOVE_LIST, KociembaCube, LOW_MEMORY, TABLE_DIR};
use crate::MOVE_LIST;

/// Checks if all necessary lookup tables for Kociemba's algorithm exist
fn check_g1_tables() -> bool {
    Path::new(&format!("{}/eo_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/co_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/uds_table.bin", TABLE_DIR)).exists()
        && (LOW_MEMORY
            || (Path::new(&format!("{}/eo_move.bin", TABLE_DIR)).exists()
                && Path::new(&format!("{}/co_move.bin", TABLE_DIR)).exists()
                && Path::new(&format!("{}/uds_move.bin", TABLE_DIR)).exists()))
}

/// Checks if all necessary G2 lookup tables exist
fn check_g2_tables() -> bool {
    Path::new(&format!("{}/ep_no_uds_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/ep_uds_table.bin", TABLE_DIR)).exists()
        && Path::new(&format!("{}/cp_table.bin", TABLE_DIR)).exists()
        && (LOW_MEMORY
            || (Path::new(&format!("{}/cp_move.bin", TABLE_DIR)).exists()
                && Path::new(&format!("{}/ep_no_uds_move.bin", TABLE_DIR)).exists()
                && Path::new(&format!("{}/ep_uds_move.bin", TABLE_DIR)).exists()))
}

/// Generates phase 1 (G0 to G1) lookup tables using Breadth-First Search (BFS)
/// This function explores the state space starting from a solved cube to calculate:
/// Pruning tables (distances): Minimum moves to reach G1 for EO, CO, and UDS slice coordinates
/// Move tables: Precomputed coordinate transitions for all 18 standard moves
fn gen_g1_tables() -> std::io::Result<()> {
    if !Path::new(TABLE_DIR).exists() {
        fs::create_dir_all(TABLE_DIR)?;
    }

    // Initialize distance tables with 255 (representing infinity/unreached)
    let mut eo_dists = [255u8; 2048];
    let mut co_dists = [255u8; 2187];
    let mut uds_dists = [255u8; 495];

    // Initialize move tables
    let mut eo_move = [[0u16; 18]; 2048];
    let mut co_move = [[0u16; 18]; 2187];
    let mut uds_move = [[0u16; 18]; 495];

    let start_cube = KociembaCube::new();

    // The solved state has a distance of 0 for all coordinates
    eo_dists[0] = 0;
    co_dists[0] = 0;
    uds_dists[0] = 0;

    // BFS Queue stores (KociembaCube, current_eo, current_co, current_uds)
    let mut queue: VecDeque<(KociembaCube, usize, usize, usize)> = VecDeque::new();
    queue.push_back((start_cube, 0, 0, 0));

    while let Some((current, d_eo, d_co, d_es)) = queue.pop_front() {
        // Try all 18 possible moves from the current state
        for (i, mv) in MOVE_LIST.iter().enumerate() {
            let mut next = current.clone();
            next.turn(&mv);

            // Get coordinates of the new state
            let n_eo = next.get_eo_coord() as usize;
            let n_co = next.get_co_coord() as usize;
            let n_es = next.get_uds_coord() as usize;

            // Fill move tables: source_coord + move_index -> destination_coord
            eo_move[d_eo][i] = n_eo as u16;
            co_move[d_co][i] = n_co as u16;
            uds_move[d_es][i] = n_es as u16;

            let mut discovered = false;
            // Update EO distance if this coordinate was never reached before
            if eo_dists[n_eo] == 255 {
                eo_dists[n_eo] = eo_dists[d_eo] + 1;
                discovered = true;
            }
            // Update CO distance
            if co_dists[n_co] == 255 {
                co_dists[n_co] = co_dists[d_co] + 1;
                discovered = true;
            }
            // Update UDS slice distance
            if uds_dists[n_es] == 255 {
                uds_dists[n_es] = uds_dists[d_es] + 1;
                discovered = true;
            }

            // If any new coordinate was found, add the cube to the queue for further exploration
            if discovered {
                queue.push_back((next, n_eo, n_co, n_es));
            }
        }
    }

    // Save pruning tables to binary files
    fs::File::create(format!("{}/eo_table.bin", TABLE_DIR))?.write_all(&eo_dists)?;
    fs::File::create(format!("{}/co_table.bin", TABLE_DIR))?.write_all(&co_dists)?;
    fs::File::create(format!("{}/uds_table.bin", TABLE_DIR))?.write_all(&uds_dists)?;

    // Save move tables to binary files using unsafe raw memory access for performance
    let eo_move_data = unsafe {
        std::slice::from_raw_parts(
            eo_move.as_ptr() as *const u8,
            std::mem::size_of_val(&eo_move),
        )
    };
    fs::File::create(format!("{}/eo_move.bin", TABLE_DIR))?.write_all(eo_move_data)?;
    let co_move_data = unsafe {
        std::slice::from_raw_parts(
            co_move.as_ptr() as *const u8,
            std::mem::size_of_val(&co_move),
        )
    };
    fs::File::create(format!("{}/co_move.bin", TABLE_DIR))?.write_all(co_move_data)?;
    let uds_move_data = unsafe {
        std::slice::from_raw_parts(
            uds_move.as_ptr() as *const u8,
            std::mem::size_of_val(&uds_move),
        )
    };
    fs::File::create(format!("{}/uds_move.bin", TABLE_DIR))?.write_all(uds_move_data)?;

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
    let mut ep_no_uds_dists = [255u8; 40320];
    let mut ep_uds_dists = [255u8; 24];
    let mut cp_dists = [255u8; 40320];

    // Move tables for phase 2 coordinates
    let mut cp_move = [[0u16; 10]; 40320];
    let mut ep_no_uds_move = [[0u16; 10]; 40320];
    let mut ep_uds_move = [[0u16; 10]; 24];

    let start_cube = KociembaCube::new();

    ep_no_uds_dists[0] = 0;
    ep_uds_dists[0] = 0;
    cp_dists[0] = 0;

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
            if ep_no_uds_dists[n_ep_no_uds] == 255 {
                ep_no_uds_dists[n_ep_no_uds] = ep_no_uds_dists[d_ep_no_uds] + 1;
                discovered = true;
            }
            if ep_uds_dists[n_ep_uds] == 255 {
                ep_uds_dists[n_ep_uds] = ep_uds_dists[d_ep_uds] + 1;
                discovered = true;
            }
            if cp_dists[n_cp] == 255 {
                cp_dists[n_cp] = cp_dists[d_cp] + 1;
                discovered = true;
            }

            if discovered {
                queue.push_back((next, n_ep_no_uds, n_ep_uds, n_cp));
            }
        }
    }

    // Save phase 2 pruning tables
    fs::File::create(format!("{}/ep_no_uds_table.bin", TABLE_DIR))?.write_all(&ep_no_uds_dists)?;
    fs::File::create(format!("{}/ep_uds_table.bin", TABLE_DIR))?.write_all(&ep_uds_dists)?;
    fs::File::create(format!("{}/cp_table.bin", TABLE_DIR))?.write_all(&cp_dists)?;

    // Save phase 2 move tables if low-memory mode is disabled
    if !LOW_MEMORY {
        let cp_move_data = unsafe {
            std::slice::from_raw_parts(
                cp_move.as_ptr() as *const u8,
                std::mem::size_of_val(&cp_move),
            )
        };
        fs::File::create(format!("{}/cp_move.bin", TABLE_DIR))?.write_all(cp_move_data)?;
        let ep_no_uds_move_data = unsafe {
            std::slice::from_raw_parts(
                ep_no_uds_move.as_ptr() as *const u8,
                std::mem::size_of_val(&ep_no_uds_move),
            )
        };
        fs::File::create(format!("{}/ep_no_uds_move.bin", TABLE_DIR))?
            .write_all(ep_no_uds_move_data)?;
        let ep_uds_move_data = unsafe {
            std::slice::from_raw_parts(
                ep_uds_move.as_ptr() as *const u8,
                std::mem::size_of_val(&ep_uds_move),
            )
        };
        fs::File::create(format!("{}/ep_uds_move.bin", TABLE_DIR))?.write_all(ep_uds_move_data)?;
    }

    Ok(())
}

/// Ensures all Kociemba lookup tables are present, generating them if necessary
pub fn gen_tables() -> std::io::Result<()> {
    if !check_g1_tables() {
        gen_g1_tables()?;
    }
    if !check_g2_tables() {
        gen_g2_tables()?;
    }

    Ok(())
}
