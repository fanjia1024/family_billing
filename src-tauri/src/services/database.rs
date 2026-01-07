use anyhow::{Context, Result as AnyhowResult};
use log::{debug, info};
use rusqlite::{Connection, Result};
use tauri::{AppHandle, Manager};

const DB_NAME: &str = "household_billing.db";

pub fn init_database(app: &AppHandle) -> AnyhowResult<()> {
    info!("[init_database] 开始初始化数据库");

    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;

    debug!("[init_database] 应用数据目录: {:?}", app_data_dir);

    std::fs::create_dir_all(&app_data_dir).context("Failed to create app data directory")?;

    let db_path = app_data_dir.join(DB_NAME);
    debug!("[init_database] 数据库路径: {:?}", db_path);

    let conn = Connection::open(&db_path).context("Failed to open database connection")?;

    info!("[init_database] 数据库连接成功");

    create_tables(&conn)?;

    // 执行数据库迁移（如新增字段等）
    migrate_bill_month(&conn)?;

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
            bill_month TEXT,
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

/// 迁移逻辑：为已有数据库添加 bill_month 字段并回填数据
fn migrate_bill_month(conn: &Connection) -> Result<()> {
    debug!("[migrate_bill_month] 开始检查并迁移 bill_month 字段");

    // 检查 bill 表中是否已经存在 bill_month 字段
    let mut stmt =
        conn.prepare("SELECT 1 FROM pragma_table_info('bill') WHERE name = 'bill_month' LIMIT 1")?;
    let exists = stmt.exists([])?;

    if !exists {
        debug!("[migrate_bill_month] bill_month 字段不存在，开始执行 ALTER TABLE");
        conn.execute("ALTER TABLE bill ADD COLUMN bill_month TEXT", [])?;

        debug!("[migrate_bill_month] 开始根据 bill_date 回填 bill_month 数据");
        conn.execute(
            "UPDATE bill SET bill_month = strftime('%Y-%m', bill_date) WHERE bill_month IS NULL OR bill_month = ''",
            [],
        )?;
    } else {
        debug!("[migrate_bill_month] bill_month 字段已存在，检查是否需要回填数据");
        // 即使字段存在，也可能有旧数据需要回填
        conn.execute(
            "UPDATE bill SET bill_month = strftime('%Y-%m', bill_date) WHERE bill_month IS NULL OR bill_month = ''",
            [],
        )?;
    }

    // 检查索引是否存在，如果不存在则创建
    let mut index_stmt = conn.prepare(
        "SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_bill_month' LIMIT 1"
    )?;
    let index_exists = index_stmt.exists([])?;

    if !index_exists {
        debug!("[migrate_bill_month] 创建 bill_month 索引");
        conn.execute(
            "CREATE INDEX idx_bill_month ON bill(bill_month)",
            [],
        )?;
    } else {
        debug!("[migrate_bill_month] bill_month 索引已存在");
    }

    info!("[migrate_bill_month] bill_month 字段迁移完成");
    Ok(())
}

fn init_default_data(conn: &Connection) -> Result<()> {
    info!("[init_default_data] 开始初始化分类数据");

    // 定义所有需要初始化的分类
    // 支出分类（消费场景，适用于微信和支付宝）
    let expense_scene_categories = vec![
        ("餐饮美食", "expense", "🍽️"),
        ("交通出行", "expense", "🚗"),
        ("购物消费", "expense", "🛒"),
        ("生活服务", "expense", "💇"),
        ("充值缴费", "expense", "📱"),
        ("医疗健康", "expense", "🏥"),
        ("旅行住宿", "expense", "✈️"),
        ("爱车养车", "expense", "🚙"),
        ("母婴亲子", "expense", "👶"),
        ("教育培训", "expense", "📚"),
        ("娱乐休闲", "expense", "🎮"),
        ("住房物业", "expense", "🏠"),
        ("通讯物流", "expense", "📦"),
        ("运动健康", "expense", "🏃"),
        ("其他支出", "expense", "📝"),
    ];

    // 收入分类（交易类型）
    let income_categories = vec![
        ("工资", "income", "💰"),
        ("奖金", "income", "🎁"),
        ("转账收入", "income", "💸"),
        ("红包收入", "income", "🧧"),
        ("退款收入", "income", "↩️"),
        ("投资收益", "income", "📈"),
        ("理财收益", "income", "💳"),
        ("其他收入", "income", "📝"),
    ];

    // 支出分类（交易类型）
    let expense_transaction_categories = vec![
        ("转账支出", "expense", "💸"),
        ("红包支出", "expense", "🧧"),
        ("商户消费", "expense", "🏪"),
        ("群收款", "expense", "📨"),
        ("信用卡还款", "expense", "💳"),
        ("生活缴费", "expense", "💡"),
        ("充值提现", "expense", "💰"),
        ("金融理财", "expense", "📊"),
    ];

    // 合并所有分类
    let all_categories: Vec<(&str, &str, &str)> = expense_scene_categories
        .iter()
        .chain(income_categories.iter())
        .chain(expense_transaction_categories.iter())
        .cloned()
        .collect();

    // 准备检查分类是否存在的语句
    let mut check_stmt =
        conn.prepare("SELECT COUNT(*) FROM category WHERE name = ?1 AND type = ?2")?;

    let mut added_count = 0;
    let mut skipped_count = 0;

    // 遍历所有分类，只添加不存在的
    for (name, type_, icon) in &all_categories {
        let exists: i32 = check_stmt.query_row([*name, *type_], |row| row.get(0))?;

        if exists == 0 {
            conn.execute(
                "INSERT INTO category (name, type, icon) VALUES (?1, ?2, ?3)",
                [*name, *type_, *icon],
            )?;
            debug!("[init_default_data] 添加分类: {} ({})", name, type_);
            added_count += 1;
        } else {
            debug!("[init_default_data] 分类已存在，跳过: {} ({})", name, type_);
            skipped_count += 1;
        }
    }

    info!(
        "[init_default_data] 分类数据初始化完成 (新增: {}, 已存在: {}, 总计: {})",
        added_count,
        skipped_count,
        all_categories.len()
    );

    Ok(())
}

pub fn get_connection(app: &AppHandle) -> AnyhowResult<Connection> {
    let db_path = get_db_path(app)?;
    debug!("[get_connection] 打开数据库连接: {:?}", db_path);
    Connection::open(&db_path).context("Failed to open database connection")
}
