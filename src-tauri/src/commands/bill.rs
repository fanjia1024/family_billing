use crate::application::bill_app_service::BillAppService;
use crate::domain::entities::bill::{BillFilters as DomainFilters, CreateBill as DomainCreateBill, UpdateBill as DomainUpdateBill};
use crate::models::bill::{Bill, BillFilters, CreateBill, UpdateBill};
use anyhow::Result;
use log::{debug, info, warn};
use tauri::AppHandle;

fn to_domain_filters(f: Option<BillFilters>) -> Option<DomainFilters> {
    f.map(|f| DomainFilters {
        member_id: f.member_id,
        category_id: f.category_id,
        start_date: f.start_date,
        end_date: f.end_date,
    })
}

fn to_domain_create_bill(b: &CreateBill) -> DomainCreateBill {
    DomainCreateBill {
        member_id: b.member_id,
        category_id: b.category_id,
        r#type: b.r#type.clone(),
        amount: b.amount,
        description: b.description.clone(),
        source: b.source.clone(),
        bill_date: b.bill_date.clone(),
        bill_month: b.bill_month.clone(),
    }
}

fn to_domain_update_bill(b: &UpdateBill) -> DomainUpdateBill {
    DomainUpdateBill {
        member_id: b.member_id,
        category_id: b.category_id,
        r#type: b.r#type.clone(),
        amount: b.amount,
        description: b.description.clone(),
        bill_date: b.bill_date.clone(),
        bill_month: b.bill_month.clone(),
    }
}

fn to_dto_bill(b: &crate::domain::entities::bill::Bill) -> Bill {
    Bill {
        id: b.id,
        member_id: b.member_id,
        category_id: b.category_id,
        r#type: b.r#type.clone(),
        amount: b.amount,
        description: b.description.clone(),
        source: b.source.clone(),
        bill_date: b.bill_date.clone(),
        bill_month: b.bill_month.clone(),
        created_at: b.created_at.clone(),
    }
}

#[tauri::command]
pub fn get_bills(app: AppHandle, filters: Option<BillFilters>) -> Result<Vec<Bill>, String> {
    info!("[get_bills] 开始获取账单列表");
    debug!("[get_bills] 筛选条件: {:?}", filters);

    let service = app.state::<BillAppService>();
    let domain_bills = service.get_bills(to_domain_filters(filters))?;
    let bills = domain_bills.iter().map(to_dto_bill).collect();

    info!("[get_bills] 成功获取 {} 条账单记录", bills.len());
    Ok(bills)
}

#[tauri::command]
pub fn create_bill(app: AppHandle, bill: CreateBill) -> Result<i64, String> {
    info!("[create_bill] 开始创建账单");
    debug!("[create_bill] 账单数据: member_id={}, category_id={}, type={}, amount={}",
        bill.member_id, bill.category_id, bill.r#type, bill.amount);

    if bill.member_id <= 0 {
        warn!("[create_bill] 无效的成员ID: {}", bill.member_id);
        return Err("无效的成员ID".to_string());
    }
    if bill.category_id <= 0 {
        warn!("[create_bill] 无效的分类ID: {}", bill.category_id);
        return Err("无效的分类ID".to_string());
    }
    if bill.amount <= 0.0 {
        warn!("[create_bill] 无效的金额: {}", bill.amount);
        return Err("金额必须大于0".to_string());
    }
    if bill.bill_date.is_empty() {
        warn!("[create_bill] 账单日期为空");
        return Err("账单日期不能为空".to_string());
    }

    let bill_month = if bill.bill_month.trim().is_empty() {
        bill.bill_date.chars().take(7).collect::<String>()
    } else {
        bill.bill_month.clone()
    };
    if bill_month.len() != 7 || !bill_month.chars().nth(4).map(|c| c == '-').unwrap_or(false) {
        warn!("[create_bill] 账单月份格式不正确: {}", bill_month);
        return Err("账单月份格式必须为 YYYY-MM".to_string());
    }

    let mut domain_bill = to_domain_create_bill(&bill);
    domain_bill.bill_month = bill_month;

    let service = app.state::<BillAppService>();
    let id = service.add_bill(domain_bill)?;
    info!("[create_bill] 账单创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_bill(app: AppHandle, id: i64, bill: UpdateBill) -> Result<(), String> {
    info!("[update_bill] 开始更新账单, id={}", id);
    debug!("[update_bill] 更新数据: {:?}", bill);

    if id <= 0 {
        warn!("[update_bill] 无效的账单ID: {}", id);
        return Err("无效的账单ID".to_string());
    }

    let service = app.state::<BillAppService>();
    service.update_bill(id, to_domain_update_bill(&bill))
}

#[tauri::command]
pub fn delete_bill(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_bill] 开始删除账单, id={}", id);

    if id <= 0 {
        warn!("[delete_bill] 无效的账单ID: {}", id);
        return Err("无效的账单ID".to_string());
    }

    let service = app.state::<BillAppService>();
    service.delete_bill(id)
}
