use crate::application::member_app_service::MemberAppService;
use crate::commands::dto::member::MemberDto;
use crate::domain::error::to_user_message;
use anyhow::Result;
use log::{debug, info, warn};
use tauri::AppHandle;

#[tauri::command]
pub fn get_members(app: AppHandle) -> Result<Vec<MemberDto>, String> {
    info!("[get_members] 开始获取成员列表");

    let service = app.state::<MemberAppService>();
    let members = service.get_members().map_err(|e| to_user_message(&e))?;
    let dtos: Vec<MemberDto> = members
        .into_iter()
        .map(|m| MemberDto {
            id: m.id,
            family_id: m.family_id,
            name: m.name,
            avatar: m.avatar,
            role: m.role,
            created_at: m.created_at,
        })
        .collect();
    info!("[get_members] 成功获取 {} 个成员", dtos.len());
    for m in &dtos {
        debug!("[get_members] 成员: id={}, name={}, role={}", m.id, m.name, m.role);
    }
    Ok(dtos)
}

#[tauri::command]
pub fn create_member(
    app: AppHandle,
    name: String,
    role: String,
    avatar: Option<String>,
) -> Result<i64, String> {
    info!("[create_member] 开始创建成员: name={}, role={}", name, role);

    if name.trim().is_empty() {
        warn!("[create_member] 成员名称为空");
        return Err("成员名称不能为空".to_string());
    }
    if role != "admin" && role != "member" {
        warn!("[create_member] 无效的角色: {}", role);
        return Err("角色必须是 admin 或 member".to_string());
    }

    let service = app.state::<MemberAppService>();
    let id = service
        .create_member(name, role, avatar)
        .map_err(|e| to_user_message(&e))?;
    info!("[create_member] 成员创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_member(
    app: AppHandle,
    id: i64,
    name: String,
    role: String,
    avatar: Option<String>,
) -> Result<(), String> {
    info!("[update_member] 开始更新成员: id={}, name={}, role={}", id, name, role);

    if id <= 0 {
        warn!("[update_member] 无效的成员ID: {}", id);
        return Err("无效的成员ID".to_string());
    }
    if name.trim().is_empty() {
        warn!("[update_member] 成员名称为空");
        return Err("成员名称不能为空".to_string());
    }
    if role != "admin" && role != "member" {
        warn!("[update_member] 无效的角色: {}", role);
        return Err("角色必须是 admin 或 member".to_string());
    }

    let service = app.state::<MemberAppService>();
    service
        .update_member(id, name, role, avatar)
        .map_err(|e| to_user_message(&e))?;
    info!("[update_member] 成员更新成功, id={}", id);
    Ok(())
}

#[tauri::command]
pub fn delete_member(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_member] 开始删除成员, id={}", id);

    if id <= 0 {
        warn!("[delete_member] 无效的成员ID: {}", id);
        return Err("无效的成员ID".to_string());
    }

    let service = app.state::<MemberAppService>();
    service.delete_member(id).map_err(|e| to_user_message(&e))?;
    info!("[delete_member] 成员删除成功, id={}", id);
    Ok(())
}
