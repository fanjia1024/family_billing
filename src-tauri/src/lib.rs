mod commands;
mod models;
mod services;
mod utils;

use log::{info, error};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: Some("app.log".into()) },
                ))
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            info!("========================================");
            info!("家庭账单管理应用启动");
            info!("========================================");
            
            // Initialize database
            match services::database::init_database(app.app_handle()) {
                Ok(_) => info!("数据库初始化成功"),
                Err(e) => {
                    error!("数据库初始化失败: {:?}", e);
                    return Err(e.into());
                }
            }
            
            info!("应用初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Family commands
            commands::family::get_family,
            commands::family::create_family,
            commands::family::update_family,
            // Member commands
            commands::member::get_members,
            commands::member::create_member,
            commands::member::update_member,
            commands::member::delete_member,
            // Bill commands
            commands::bill::get_bills,
            commands::bill::create_bill,
            commands::bill::update_bill,
            commands::bill::delete_bill,
            // Category commands
            commands::category::get_categories,
            commands::category::create_category,
            commands::category::update_category,
            commands::category::delete_category,
            // OCR commands
            commands::ocr::ocr_recognize,
            commands::ocr::ocr_recognize_batch,
            commands::ocr::save_bill_with_image,
            // Export/Import commands
            commands::export::export_to_json,
            commands::export::export_to_excel,
            commands::export::import_from_json,
            // Statistics commands
            commands::statistics::get_statistics,
            // Image commands
            commands::image::save_uploaded_image,
            commands::image::get_image_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
