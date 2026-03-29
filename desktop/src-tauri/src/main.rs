#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::{sample_river_request, solve_river_spot, validate_config};
use commands::write_e2e_smoke_report;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            sample_river_request,
            validate_config,
            solve_river_spot,
            write_e2e_smoke_report
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Pokie desktop shell");
}
