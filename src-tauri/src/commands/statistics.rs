use crate::services::database;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use log::{info, debug, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub total_income: f64,
    pub total_expense: f64,
    pub balance: f64,
    pub monthly_data: Vec<MonthlyData>,
    pub category_data: Vec<CategoryData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyData {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryData {
    pub category_id: i64,
    pub category_name: String,
    pub amount: f64,
    pub percentage: f64,
}

#[tauri::command]
pub fn get_statistics(
    app: AppHandle,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Statistics, String> {
    info!("[get_statistics] 开始获取统计数据");
    debug!("[get_statistics] 日期范围: start={:?}, end={:?}", start_date, end_date);
    
    let conn = database::get_connection(&app).map_err(|e| {
        error!("[get_statistics] 数据库连接失败: {}", e);
        e.to_string()
    })?;

    let mut where_clause = "WHERE 1=1".to_string();
    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref start) = start_date {
        where_clause.push_str(" AND bill_date >= ?");
        query_params.push(Box::new(start.clone()));
        debug!("[get_statistics] 添加开始日期筛选: {}", start);
    }
    if let Some(ref end) = end_date {
        where_clause.push_str(" AND bill_date <= ?");
        query_params.push(Box::new(end.clone()));
        debug!("[get_statistics] 添加结束日期筛选: {}", end);
    }

    let params: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

    // Get total income and expense
    debug!("[get_statistics] 查询总收入和支出");
    let mut stmt = conn
        .prepare(&format!(
            "SELECT 
                COALESCE(SUM(CASE WHEN type = 'income' THEN amount ELSE 0 END), 0) as total_income,
                COALESCE(SUM(CASE WHEN type = 'expense' THEN amount ELSE 0 END), 0) as total_expense
            FROM bill {}",
            where_clause
        ))
        .map_err(|e| {
            error!("[get_statistics] SQL准备失败 (总计): {}", e);
            e.to_string()
        })?;

    let (total_income, total_expense) = stmt
        .query_row(&params[..], |row| {
            Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|e| {
            error!("[get_statistics] 查询总计失败: {}", e);
            e.to_string()
        })?;

    let balance = total_income - total_expense;
    info!("[get_statistics] 总收入={:.2}, 总支出={:.2}, 结余={:.2}", 
        total_income, total_expense, balance);

    // Get monthly data
    // 优先使用 bill_month 字段，如果为空则从 bill_date 提取
    debug!("[get_statistics] 查询月度数据");
    let mut stmt = conn
        .prepare(&format!(
            "SELECT 
                COALESCE(NULLIF(bill_month, ''), strftime('%Y-%m', bill_date)) as month,
                COALESCE(SUM(CASE WHEN type = 'income' THEN amount ELSE 0 END), 0) as income,
                COALESCE(SUM(CASE WHEN type = 'expense' THEN amount ELSE 0 END), 0) as expense
            FROM bill {}
            GROUP BY month
            ORDER BY month",
            where_clause
        ))
        .map_err(|e| {
            error!("[get_statistics] SQL准备失败 (月度): {}", e);
            e.to_string()
        })?;

    let monthly_data = stmt
        .query_map(&params[..], |row| {
            Ok(MonthlyData {
                month: row.get(0)?,
                income: row.get(1)?,
                expense: row.get(2)?,
            })
        })
        .map_err(|e| {
            error!("[get_statistics] 查询月度数据失败: {}", e);
            e.to_string()
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("[get_statistics] 解析月度数据失败: {}", e);
            e.to_string()
        })?;
    
    info!("[get_statistics] 获取到 {} 个月的数据", monthly_data.len());

    // Get category data for expenses
    debug!("[get_statistics] 查询分类数据");
    let mut stmt = conn
        .prepare(&format!(
            "SELECT 
                b.category_id,
                c.name as category_name,
                SUM(b.amount) as amount
            FROM bill b
            JOIN category c ON b.category_id = c.id
            {} AND b.type = 'expense'
            GROUP BY b.category_id, c.name
            ORDER BY amount DESC",
            where_clause
        ))
        .map_err(|e| {
            error!("[get_statistics] SQL准备失败 (分类): {}", e);
            e.to_string()
        })?;

    let category_data = stmt
        .query_map(&params[..], |row| {
            Ok(CategoryData {
                category_id: row.get(0)?,
                category_name: row.get(1)?,
                amount: row.get(2)?,
                percentage: 0.0, // Will calculate below
            })
        })
        .map_err(|e| {
            error!("[get_statistics] 查询分类数据失败: {}", e);
            e.to_string()
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("[get_statistics] 解析分类数据失败: {}", e);
            e.to_string()
        })?;

    // Calculate percentages
    let total_expense_for_percentage = if total_expense > 0.0 {
        total_expense
    } else {
        1.0
    };
    let category_data: Vec<CategoryData> = category_data
        .into_iter()
        .map(|mut cd| {
            cd.percentage = (cd.amount / total_expense_for_percentage) * 100.0;
            cd
        })
        .collect();
    
    info!("[get_statistics] 获取到 {} 个支出分类", category_data.len());
    for cd in &category_data {
        debug!("[get_statistics] 分类: {}={:.2} ({:.1}%)", 
            cd.category_name, cd.amount, cd.percentage);
    }

    info!("[get_statistics] 统计数据获取完成");
    Ok(Statistics {
        total_income,
        total_expense,
        balance,
        monthly_data,
        category_data,
    })
}
