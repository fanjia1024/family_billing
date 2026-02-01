mod application;
mod commands;
mod domain;
mod infrastructure;
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

            let app_handle = app.app_handle().clone();

            // Initialize database
            match infrastructure::persistence::database::init_database(&app_handle) {
                Ok(_) => info!("数据库初始化成功"),
                Err(e) => {
                    error!("数据库初始化失败: {:?}", e);
                    return Err(e.into());
                }
            }

            // Dependency injection: Application layer with Infrastructure implementations
            let bill_repo = Box::new(infrastructure::persistence::sqlite_bill_repo::SqliteBillRepository::new(
                app_handle.clone(),
            ));
            let ocr_engine = Box::new(infrastructure::ocr::tesseract_engine::TesseractOcrEngine::new());
            let bill_app_service = application::bill_app_service::BillAppService::new(
                bill_repo,
                ocr_engine,
            );
            app.manage(bill_app_service);

            let bill_repo_stats = Box::new(infrastructure::persistence::sqlite_bill_repo::SqliteBillRepository::new(
                app_handle.clone(),
            ));
            let category_repo_stats = Box::new(infrastructure::persistence::sqlite_category_repo::SqliteCategoryRepository::new(
                app_handle.clone(),
            ));
            let statistics_calculator = Box::new(domain::services::statistics_calculator::DefaultStatisticsCalculator::new());
            let statistics_app_service = application::statistics_app_service::StatisticsAppService::new(
                bill_repo_stats,
                category_repo_stats,
                statistics_calculator,
            );
            app.manage(statistics_app_service);

            let family_repo = Box::new(infrastructure::persistence::sqlite_family_repo::SqliteFamilyRepository::new(
                app_handle.clone(),
            ));
            let family_app_service = application::family_app_service::FamilyAppService::new(family_repo);
            app.manage(family_app_service);

            let member_repo = Box::new(infrastructure::persistence::sqlite_member_repo::SqliteMemberRepository::new(
                app_handle.clone(),
            ));
            let family_repo_member = Box::new(infrastructure::persistence::sqlite_family_repo::SqliteFamilyRepository::new(
                app_handle.clone(),
            ));
            let bill_repo_member = Box::new(infrastructure::persistence::sqlite_bill_repo::SqliteBillRepository::new(
                app_handle.clone(),
            ));
            let member_app_service = application::member_app_service::MemberAppService::new(
                member_repo,
                family_repo_member,
                bill_repo_member,
            );
            app.manage(member_app_service);

            let category_repo = Box::new(infrastructure::persistence::sqlite_category_repo::SqliteCategoryRepository::new(
                app_handle.clone(),
            ));
            let bill_repo_category = Box::new(infrastructure::persistence::sqlite_bill_repo::SqliteBillRepository::new(
                app_handle.clone(),
            ));
            let category_app_service = application::category_app_service::CategoryAppService::new(
                category_repo,
                bill_repo_category,
            );
            app.manage(category_app_service);

            let family_repo_export = Box::new(infrastructure::persistence::sqlite_family_repo::SqliteFamilyRepository::new(
                app_handle.clone(),
            ));
            let member_repo_export = Box::new(infrastructure::persistence::sqlite_member_repo::SqliteMemberRepository::new(
                app_handle.clone(),
            ));
            let category_repo_export = Box::new(infrastructure::persistence::sqlite_category_repo::SqliteCategoryRepository::new(
                app_handle.clone(),
            ));
            let bill_repo_export = Box::new(infrastructure::persistence::sqlite_bill_repo::SqliteBillRepository::new(
                app_handle.clone(),
            ));
            let export_app_service = application::export_app_service::ExportAppService::new(
                family_repo_export,
                member_repo_export,
                category_repo_export,
                bill_repo_export,
            );
            app.manage(export_app_service);

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
