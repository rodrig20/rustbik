mod constants;
mod solver;
mod tables;
mod utils;

pub use constants::{LOW_MEMORY, TABLE_DIR};
pub(crate) use utils::{G1_MOVE_LIST, KociembaCube};

pub use solver::solve;
pub use tables::gen_tables;

#[cfg(test)]
mod tests {
    use super::utils::KociembaCube;
    use crate::Cube;

    #[test]
    fn test_kociemba_api_integration() {
        let cube = KociembaCube(Cube::new());
        // Ensure the coordinate API is accessible and correct for a user
        assert_eq!(cube.get_eo_coord(), 0);
    }
}
