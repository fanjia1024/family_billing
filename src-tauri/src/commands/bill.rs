use crate::application::bill_app_service::BillAppService;
use crate::commands::dto::bill::{
    BillDto, BillFiltersDto, CreateBillDto, UpdateBillDto,
};
use crate::domain::entities::bill::{BillFilters, CreateBill, UpdateBill};
use crate::domain::error::{to_user_message, DomainError};
use anyhow::Result;
use log::{debug, info, warn};
use tauri::AppHandle;

fn to_domain_filters(f: Option<BillFiltersDto>) -> Option<BillFilters> {
    f.map(|f| BillFilters {
        member_id: f.member_id,
        category_id: f.category_id,
        start_date: f.start_date,
        end_date: f.end_date,
    })
}

fn to_domain_create_bill(b: &CreateBillDto) -> Result<CreateBill, String> {
    CreateBill::try_new(
        b.member_id,
        b.category_id,
        b.r#type.clone(),
        b.amount,
        b.description.clone(),
        b.source.clone(),
        b.bill_date.clone(),
        b.bill_month.clone(),
    )
    .map_err(|e| to_user_message(&e))
}

fn to_domain_update_bill(b: &UpdateBillDto) -> UpdateBill {
    UpdateBill {
        member_id: b.member_id,
        category_id: b.category_id,
        r#type: b.r#type.clone(),
        amount: b.amount,
        description: b.description.clone(),
        bill_date: b.bill_date.clone(),
        bill_month: b.bill_month.clone(),
    }
}

fn to_dto_bill(b: &crate::domain::entities::bill::Bill) -> BillDto {
    BillDto {
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
pub fn get_bills(app: AppHandle, filters: Option<BillFiltersDto>) -> Result<Vec<BillDto>, String> {
    info!("[get_bills] 开始获取账单列表");
    debug!("[get_bills] 筛选条件: {:?}", filters);

    let service = app.state::<BillAppService>();
    let domain_bills = service.get_bills(to_domain_filters(filters))?;
    let bills = domain_bills.iter().map(to_dto_bill).collect();

    info!("[get_bills] 成功获取 {} 条账单记录", bills.len());
    Ok(bills)
}

#[tauri::command]
pub fn create_bill(app: AppHandle, bill: CreateBillDto) -> Result<i64, String> {
    info!("[create_bill] 开始创建账单");
    debug!("[create_bill] 账单数据: member_id={}, category_id={}, type={}, amount={}",
        bill.member_id, bill.category_id, bill.r#type, bill.amount);

    let domain_bill = to_domain_create_bill(&bill)?;
    let service = app.state::<BillAppService>();
    let id = service.add_bill(domain_bill)?;
    info!("[create_bill] 账单创建成功, id={}", id);
    Ok(id)
}

#[tauri::command]
pub fn update_bill(app: AppHandle, id: i64, bill: UpdateBillDto) -> Result<(), String> {
    info!("[update_bill] 开始更新账单, id={}", id);
    debug!("[update_bill] 更新数据: {:?}", bill);

    if id <= 0 {
        return Err(to_user_message(&DomainError::InvalidBillId));
    }

    let service = app.state::<BillAppService>();
    service.update_bill(id, to_domain_update_bill(&bill))
}

#[tauri::command]
pub fn delete_bill(app: AppHandle, id: i64) -> Result<(), String> {
    info!("[delete_bill] 开始删除账单, id={}", id);

    if id <= 0 {
        return Err(to_user_message(&DomainError::InvalidBillId));
    }

    let service = app.state::<BillAppService>();
    service.delete_bill(id)
}
