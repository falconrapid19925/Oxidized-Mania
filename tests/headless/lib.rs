// Compile the production modules directly; no GPU, audio libraries or copied logic.
#[path = "../../src/config.rs"]
pub mod config;
#[path = "../../src/chart_timing.rs"]
pub mod chart_timing;
#[path = "../../src/game/note_index.rs"]
pub mod note_index;
