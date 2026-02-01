use crate::application::family_app_service::FamilyAppService;
use crate::commands::dto::family::FamilyDto;
use anyhow::Result;
use log::{debug, info, warn};
use tauri::AppHandle;

#[tauri::command]
pub fn get_family(app: AppHandle) -> Result<Option<FamilyDto>, String> {
    info!("[get_family] 开始获取家庭信息");

    let service = app.state::<FamilyAppService>();
    let family = service.get_family()?;
    let dto = family.map(|f| FamilyDto {
        id: f.id,
        name: f.name,
        created_at: f.created_at,
    });
    if let Some(ref f) = dto {
        info!("[get_family] 获取家庭成功: id={}, name={}", f.id, f.name);
    } else {
        info!("[get_family] 未找到家庭信息");
    }
    Ok(dto)
}

#[tauri::command]
pub fn create_family(app: AppHandle, name: String) -> Result<i64, String> {
    info!("[create_family] 开始创建家庭: name={}", name);

    if name.trim().is_empty() {
        warn!("[create_family] 家庭名称为空");
        return Err("家庭名称不能为空".to_string());
    }

    let service = app.state::<FamilyAppService>();
    let id = service.create_family(name)?;
    info!("[create_family] 家庭创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_family(app: AppHandle, id: i64, name: String) -> Result<(), String> {
    info!("[update_family] 开始更新家庭: id={}, name={}", id, name);

    if id <= 0 {
        warn!("[update_family] 无效的家庭ID: {}", id);
        return Err("无效的家庭ID".to_string());
    }
    if name.trim().is_empty() {
        warn!("[update_family] 家庭名称为空");
        return Err("家庭名称不能为空".to_string());
    }

    let service = app.state::<FamilyAppService>();
    service.update_family(id, name)?;
    info!("[update_family] 家庭更新成功, id={}", id);
    Ok(())
}
