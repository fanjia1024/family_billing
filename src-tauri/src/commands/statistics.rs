use crate::application::statistics_app_service::StatisticsAppService;
use crate::commands::dto::statistics::{
    CategoryDataDto, MonthlyDataDto, StatisticsDto,
};
use anyhow::Result;
use log::{debug, info};
use tauri::AppHandle;

#[tauri::command]
pub fn get_statistics(
    app: AppHandle,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<StatisticsDto, String> {
    info!("[get_statistics] 开始获取统计数据");
    debug!("[get_statistics] 日期范围: start={:?}, end={:?}", start_date, end_date);

    let service = app.state::<StatisticsAppService>();
    let result = service.get_statistics(start_date, end_date)?;

    let monthly_data: Vec<MonthlyDataDto> = result
        .monthly_data
        .into_iter()
        .map(|m| MonthlyDataDto {
            month: m.month,
            income: m.income,
            expense: m.expense,
        })
        .collect();

    let category_data: Vec<CategoryDataDto> = result
        .category_data
        .into_iter()
        .map(|c| CategoryDataDto {
            category_id: c.category_id,
            category_name: c.category_name,
            amount: c.amount,
            percentage: c.percentage,
        })
        .collect();

    info!("[get_statistics] 统计数据获取完成");
    Ok(StatisticsDto {
        total_income: result.total_income,
        total_expense: result.total_expense,
        balance: result.balance,
        monthly_data,
        category_data,
    })
}
