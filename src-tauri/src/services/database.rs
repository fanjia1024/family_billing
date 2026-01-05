use rusqlite::{Connection, Result};
use tauri::{AppHandle, Manager};
use anyhow::{Context, Result as AnyhowResult};
use log::{info, debug};

const DB_NAME: &str = "household_billing.db";

pub fn init_database(app: &AppHandle) -> AnyhowResult<()> {
    info!("[init_database] 开始初始化数据库");
    
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;
    
    debug!("[init_database] 应用数据目录: {:?}", app_data_dir);
    
    std::fs::create_dir_all(&app_data_dir)
        .context("Failed to create app data directory")?;
    
    let db_path = app_data_dir.join(DB_NAME);
    debug!("[init_database] 数据库路径: {:?}", db_path);
    
    let conn = Connection::open(&db_path)
        .context("Failed to open database connection")?;
    
    info!("[init_database] 数据库连接成功");
    
    create_tables(&conn)?;
    init_default_data(&conn)?;
    
    info!("[init_database] 数据库初始化完成");
    Ok(())
}

pub fn get_db_path(app: &AppHandle) -> AnyhowResult<std::path::PathBuf> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;
    
    Ok(app_data_dir.join(DB_NAME))
}

fn create_tables(conn: &Connection) -> Result<()> {
    info!("[create_tables] 开始创建数据库表");
    
    // Family table
    debug!("[create_tables] 创建 family 表");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS family (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Member table
    debug!("[create_tables] 创建 member 表");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS member (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            family_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            avatar TEXT,
            role TEXT NOT NULL DEFAULT 'member',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (family_id) REFERENCES family(id)
        )",
        [],
    )?;

    // Category table
    debug!("[create_tables] 创建 category 表");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS category (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('income', 'expense')),
            icon TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Bill table
    debug!("[create_tables] 创建 bill 表");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bill (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            member_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            type TEXT NOT NULL CHECK(type IN ('income', 'expense')),
            amount REAL NOT NULL,
            description TEXT,
            source TEXT NOT NULL DEFAULT 'manual' CHECK(source IN ('wechat', 'alipay', 'manual')),
            bill_date TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (member_id) REFERENCES member(id),
            FOREIGN KEY (category_id) REFERENCES category(id)
        )",
        [],
    )?;

    // BillImage table
    debug!("[create_tables] 创建 bill_image 表");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bill_image (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bill_id INTEGER NOT NULL,
            image_path TEXT NOT NULL,
            ocr_raw_text TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (bill_id) REFERENCES bill(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create indexes
    debug!("[create_tables] 创建索引");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bill_member ON bill(member_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bill_date ON bill(bill_date)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bill_category ON bill(category_id)",
        [],
    )?;

    info!("[create_tables] 数据库表创建完成");
    Ok(())
}

fn init_default_data(conn: &Connection) -> Result<()> {
    debug!("[init_default_data] 检查是否需要初始化默认数据");
    
    // Check if default categories exist
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM category")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;
    
    if count == 0 {
        info!("[init_default_data] 初始化默认分类数据");
        
        // Insert default expense categories
        let expense_categories = vec![
            ("餐饮", "expense", "🍽️"),
            ("交通", "expense", "🚗"),
            ("购物", "expense", "🛒"),
            ("娱乐", "expense", "🎮"),
            ("医疗", "expense", "🏥"),
            ("教育", "expense", "📚"),
            ("住房", "expense", "🏠"),
            ("其他支出", "expense", "📝"),
        ];
        
        for (name, type_, icon) in &expense_categories {
            conn.execute(
                "INSERT INTO category (name, type, icon) VALUES (?1, ?2, ?3)",
                [*name, *type_, *icon],
            )?;
            debug!("[init_default_data] 添加支出分类: {}", name);
        }
        
        // Insert default income categories
        let income_categories = vec![
            ("工资", "income", "💰"),
            ("奖金", "income", "🎁"),
            ("投资", "income", "📈"),
            ("转账", "income", "💸"),
            ("其他收入", "income", "📝"),
        ];
        
        for (name, type_, icon) in &income_categories {
            conn.execute(
                "INSERT INTO category (name, type, icon) VALUES (?1, ?2, ?3)",
                [*name, *type_, *icon],
            )?;
            debug!("[init_default_data] 添加收入分类: {}", name);
        }
        
        info!("[init_default_data] 默认分类数据初始化完成 (支出: {}, 收入: {})", 
            expense_categories.len(), income_categories.len());
    } else {
        debug!("[init_default_data] 分类数据已存在 ({} 条)，跳过初始化", count);
    }
    
    Ok(())
}

pub fn get_connection(app: &AppHandle) -> AnyhowResult<Connection> {
    let db_path = get_db_path(app)?;
    debug!("[get_connection] 打开数据库连接: {:?}", db_path);
    Connection::open(&db_path)
        .context("Failed to open database connection")
}
