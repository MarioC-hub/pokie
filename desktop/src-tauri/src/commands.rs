use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
};

use app_api::{
    sample_river_request as sample_river_request_api, solve_river_spot as solve_river_spot_api,
    validate_config as validate_config_api, AppErrorDto, RiverSolveRequestDto,
    RiverSolveResponseDto, ValidateConfigResponseDto,
};

#[tauri::command]
pub fn sample_river_request() -> RiverSolveRequestDto {
    append_e2e_trace("sample_river_request:start");
    let response = sample_river_request_api();
    append_e2e_trace("sample_river_request:end");
    response
}

#[tauri::command]
pub fn validate_config(
    request: RiverSolveRequestDto,
) -> Result<ValidateConfigResponseDto, AppErrorDto> {
    append_e2e_trace("validate_config:start");
    let response = validate_config_api(&request);
    append_e2e_trace("validate_config:end");
    response
}

#[tauri::command]
pub fn solve_river_spot(
    request: RiverSolveRequestDto,
) -> Result<RiverSolveResponseDto, AppErrorDto> {
    append_e2e_trace("solve_river_spot:start");
    let response = solve_river_spot_api(&request);
    append_e2e_trace("solve_river_spot:end");
    response
}

#[tauri::command]
pub fn write_e2e_smoke_report(report: String) -> Result<(), String> {
    append_e2e_trace("write_e2e_smoke_report:start");
    if let Some(path) = env::var_os("POKIE_E2E_REPORT_PATH") {
        fs::write(path, report).map_err(|error| error.to_string())?;
    }
    append_e2e_trace("write_e2e_smoke_report:end");
    Ok(())
}

fn append_e2e_trace(event: &str) {
    let Some(path) = env::var_os("POKIE_E2E_TRACE_PATH") else {
        return;
    };

    let mut file = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => file,
        Err(_) => return,
    };
    let _ = writeln!(file, "{event}");
}
