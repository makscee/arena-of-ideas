use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("server_nodes.rs");

    // For server, we just re-export the node types from raw-nodes-v2
    // The actual implementations are in raw-nodes-v2
    let generated = r#"
// Re-export node types from raw-nodes-v2
pub use raw_nodes_v2::raw_nodes::{
    NArena, NFloorPool, NFloorBoss, NPlayer, NPlayerData, NPlayerIdentity,
    NHouse, NHouseColor, NAbilityMagic, NAbilityDescription, NAbilityEffect,
    NStatusMagic, NStatusDescription, NStatusBehavior, NStatusRepresentation,
    NTeam, NBattle, NMatch, NFusion, NFusionSlot, NUnit, NUnitDescription,
    NUnitStats, NUnitState, NUnitBehavior, NUnitRepresentation,
};
"#;

    fs::write(&dest_path, generated).expect("Failed to write generated code");
}
