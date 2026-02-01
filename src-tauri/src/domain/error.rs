//! Domain errors for validation and application boundaries.
//! Application/validators use these; Commands map to user-facing messages via to_user_message.
//! Infra can map rusqlite::Error etc. to PersistenceError in a future pass for full unification.

use std::fmt;

#[derive(Debug, Clone)]
pub enum DomainError {
    InvalidAmount,
    InvalidDate(String),
    InvalidBillType(String),
    InvalidMemberId,
    InvalidCategoryId,
    InvalidBillId,
    CategoryNotFound,
    MemberNotFound,
    FamilyAlreadyExists,
    CategoryAlreadyExists,
    HasDependentBills(u32),
    OcrFailed(String),
    PersistenceError(String),
    NotFound(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidAmount => write!(f, "金额必须大于0"),
            DomainError::InvalidDate(msg) => write!(f, "日期无效: {}", msg),
            DomainError::InvalidBillType(msg) => write!(f, "类型无效: {}", msg),
            DomainError::InvalidMemberId => write!(f, "无效的成员ID"),
            DomainError::InvalidCategoryId => write!(f, "无效的分类ID"),
            DomainError::InvalidBillId => write!(f, "无效的账单ID"),
            DomainError::CategoryNotFound => write!(f, "分类不存在"),
            DomainError::MemberNotFound => write!(f, "成员不存在"),
            DomainError::FamilyAlreadyExists => write!(f, "家庭已存在"),
            DomainError::CategoryAlreadyExists => write!(f, "该分类已存在"),
            DomainError::HasDependentBills(n) => {
                write!(f, "有关联的 {} 条账单，请先删除或更改关联", n)
            }
            DomainError::OcrFailed(msg) => write!(f, "OCR识别失败: {}", msg),
            DomainError::PersistenceError(msg) => write!(f, "持久化失败: {}", msg),
            DomainError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Map DomainError to user-facing message (for Command layer).
pub fn to_user_message(e: &DomainError) -> String {
    e.to_string()
}
