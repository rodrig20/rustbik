/// Directory path where precomputed Kociemba lookup tables are stored
pub const TABLE_DIR: &str = "libs/rustbik-cube/tables";

/// If true, the solver uses less memory by not loading move tables into RAM,
/// recalculating them on-the-fly instead at the cost of performance
pub const LOW_MEMORY: bool = false;
