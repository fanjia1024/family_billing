use crate::application::category_app_service::CategoryAppService;
use crate::commands::dto::category::CategoryDto;
use anyhow::Result;
use log::{debug, info, warn};
use tauri::AppHandle;

#[tauri::command]
pub fn get_categories(app: AppHandle) -> Result<Vec<CategoryDto>, String> {
    info!("[get_categories] 开始获取分类列表");

    let service = app.state::<CategoryAppService>();
    let categories = service.get_categories()?;
    let dtos: Vec<CategoryDto> = categories
        .into_iter()
        .map(|c| CategoryDto {
            id: c.id,
            name: c.name,
            r#type: c.r#type,
            icon: c.icon,
            created_at: c.created_at,
        })
        .collect();
    info!("[get_categories] 成功获取 {} 个分类", dtos.len());
    for c in &dtos {
        debug!("[get_categories] 分类: id={}, name={}, type={}", c.id, c.name, c.r#type);
    }
    Ok(dtos)
}

#[tauri::command]
pub fn create_category(
    app: AppHandle,
    name: String,
    r#type: String,
    icon: Option<String>,
) -> Result<i64, String> {
    info!("[create_category] 开始创建分类: name={}, type={}", name, r#type);

    if name.trim().is_empty() {
        warn!("[create_category] 分类名称为空");
        return Err("分类名称不能为空".to_string());
    }
    if r#type != "income" && r#type != "expense" {
        warn!("[create_category] 无效的分类类型: {}", r#type);
        return Err("类型必须是 income 或 expense".to_string());
    }

    let service = app.state::<CategoryAppService>();
    let id = service.create_category(name, r#type, icon)?;
    info!("[create_category] 分类创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_category(
    app: AppHandle,
    id: i64,
    name: String,
    icon: Option<String>,
) -> Result<(), String> {
    info!("[update_category] 开始更新分类: id={}, name={}", id, name);

    if id <= 0 {
        warn!("[update_category] 无效的分类ID: {}", id);
        return Err("无效的分类ID".to_string());
    }
    if name.trim().is_empty() {
        warn!("[update_category] 分类名称为空");
        return Err("分类名称不能为空".to_string());
    }

    let service = app.state::<CategoryAppService>();
    service.update_category(id, name, icon)?;
    info!("[update_category] 分类更新成功, id={}", id);
    Ok(())
}

#[tauri::command]
pub fn delete_category(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_category] 开始删除分类, id={}", id);

    if id <= 0 {
        warn!("[delete_category] 无效的分类ID: {}", id);
        return Err("无效的分类ID".to_string());
    }

    let service = app.state::<CategoryAppService>();
    service.delete_category(id)?;
    info!("[delete_category] 分类删除成功, id={}", id);
    Ok(())
}
