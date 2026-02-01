use crate::application::export_app_service::ExportAppService;
use crate::application::import_app_service::ImportAppService;
use crate::domain::error::to_user_message;
use log::{info, warn};
use tauri::AppHandle;

#[tauri::command]
pub fn export_to_json(app: AppHandle, file_path: String) -> Result<(), String> {
    info!("[export_to_json] 开始导出数据到JSON: {}", file_path);

    if file_path.is_empty() {
        warn!("[export_to_json] 文件路径为空");
        return Err("文件路径不能为空".to_string());
    }

    let service = app.state::<ExportAppService>();
    service
        .export_to_json(&file_path)
        .map_err(|e| to_user_message(&e))?;

    info!("[export_to_json] 数据导出成功: {}", file_path);
    Ok(())
}

#[tauri::command]
pub fn export_to_excel(app: AppHandle, file_path: String) -> Result<(), String> {
    info!("[export_to_excel] 开始导出数据到Excel: {}", file_path);
    warn!("[export_to_excel] Excel导出暂不支持，将导出为JSON格式");
    let json_path = file_path.replace(".xlsx", ".json");
    export_to_json(app, json_path)
}

#[tauri::command]
pub fn import_from_json(app: AppHandle, file_path: String) -> Result<(), String> {
    info!("[import_from_json] 开始从JSON导入数据: {}", file_path);

    let service = app.state::<ImportAppService>();
    service
        .import_from_json(&file_path)
        .map_err(|e| to_user_message(&e))?;

    info!("[import_from_json] 数据导入成功");
    Ok(())
}
