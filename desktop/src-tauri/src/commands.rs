use app_api::{
    sample_river_request as sample_river_request_api, solve_river_spot as solve_river_spot_api,
    validate_config as validate_config_api, AppErrorDto, RiverSolveRequestDto,
    RiverSolveResponseDto, ValidateConfigResponseDto,
};

#[tauri::command]
pub fn sample_river_request() -> RiverSolveRequestDto {
    sample_river_request_api()
}

#[tauri::command]
pub fn validate_config(
    request: RiverSolveRequestDto,
) -> Result<ValidateConfigResponseDto, AppErrorDto> {
    validate_config_api(&request)
}

#[tauri::command]
pub fn solve_river_spot(
    request: RiverSolveRequestDto,
) -> Result<RiverSolveResponseDto, AppErrorDto> {
    solve_river_spot_api(&request)
}
