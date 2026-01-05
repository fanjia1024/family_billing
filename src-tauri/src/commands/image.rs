use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use log::{info, debug, error, warn};

#[tauri::command]
pub fn save_uploaded_image(
    app: AppHandle,
    image_data: Vec<u8>,
    filename: String,
) -> Result<String, String> {
    info!("[save_uploaded_image] 开始保存上传的图片: filename={}, data_size={} bytes", 
        filename, image_data.len());
    
    if image_data.is_empty() {
        warn!("[save_uploaded_image] 图片数据为空");
        return Err("图片数据为空".to_string());
    }
    
    if filename.is_empty() {
        warn!("[save_uploaded_image] 文件名为空");
        return Err("文件名不能为空".to_string());
    }
    
    // Get app data directory for images
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        error!("[save_uploaded_image] 获取应用数据目录失败: {}", e);
        e.to_string()
    })?;
    debug!("[save_uploaded_image] 应用数据目录: {:?}", app_data_dir);

    let images_dir = app_data_dir.join("images");
    fs::create_dir_all(&images_dir).map_err(|e| {
        error!("[save_uploaded_image] 创建图片目录失败: {}", e);
        e.to_string()
    })?;
    debug!("[save_uploaded_image] 图片目录: {:?}", images_dir);

    // Generate unique filename
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path_buf = PathBuf::from(&filename);
    let extension = path_buf
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg")
        .to_string();

    let uuid_str = uuid::Uuid::new_v4().to_string();
    let short_uuid: String = uuid_str.chars().take(8).collect();
    let unique_filename = format!("{}_{}.{}", timestamp, short_uuid, extension);
    let file_path = images_dir.join(&unique_filename);
    debug!("[save_uploaded_image] 生成的文件路径: {:?}", file_path);

    // Save image
    let mut file = fs::File::create(&file_path).map_err(|e| {
        error!("[save_uploaded_image] 创建文件失败: {:?}, error: {}", file_path, e);
        e.to_string()
    })?;
    
    file.write_all(&image_data).map_err(|e| {
        error!("[save_uploaded_image] 写入文件失败: {}", e);
        e.to_string()
    })?;

    let result_path = file_path.to_string_lossy().to_string();
    info!("[save_uploaded_image] 图片保存成功: {}", result_path);
    
    // Return relative path for storage in database
    Ok(result_path)
}

#[tauri::command]
pub fn get_image_path(app: AppHandle, image_path: String) -> Result<String, String> {
    info!("[get_image_path] 获取图片完整路径: {}", image_path);
    
    if image_path.is_empty() {
        warn!("[get_image_path] 图片路径为空");
        return Err("图片路径不能为空".to_string());
    }
    
    // If it's already an absolute path, return it
    if PathBuf::from(&image_path).is_absolute() {
        debug!("[get_image_path] 已是绝对路径，直接返回");
        return Ok(image_path);
    }

    // Otherwise, construct full path from app data directory
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        error!("[get_image_path] 获取应用数据目录失败: {}", e);
        e.to_string()
    })?;

    let full_path = app_data_dir.join(&image_path);
    let result = full_path.to_string_lossy().to_string();
    
    info!("[get_image_path] 完整路径: {}", result);
    Ok(result)
}
