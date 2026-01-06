use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Luma, Rgb, RgbImage};
use log::{debug, error, info, warn};
use regex::Regex;
use std::path::Path;

pub struct OcrResult {
    pub r#type: String,
    pub amount: f64,
    pub description: String,
    pub bill_date: String,
    pub raw_text: String,
}

/// 账单明细项
#[derive(Debug, Clone)]
pub struct BillItem {
    pub category: String,
    pub amount: f64,
    pub percentage: Option<f64>,
    pub bill_type: String, // income 或 expense
}

/// OCR 识别结果，包含多个账单项
pub struct OcrDetailResult {
    pub items: Vec<BillItem>,
    pub total_amount: f64,
    pub raw_text: String,
    pub bill_date: String,
}

/// 检查 Tesseract 是否已安装
#[allow(dead_code)]
pub fn check_tesseract_installed() -> bool {
    match tesseract::Tesseract::new(None, Some("eng")) {
        Ok(_) => {
            info!("[check_tesseract_installed] Tesseract 已安装");
            true
        }
        Err(e) => {
            warn!(
                "[check_tesseract_installed] Tesseract 未安装或配置错误: {}",
                e
            );
            false
        }
    }
}

/// 使用 Tesseract 进行 OCR 识别（优化版）
fn perform_tesseract_ocr(image_path: &str) -> Result<String> {
    info!("[perform_tesseract_ocr] 开始 Tesseract OCR: {}", image_path);

    // 验证文件存在
    if !Path::new(image_path).exists() {
        return Err(anyhow!("图片文件不存在: {}", image_path));
    }

    // 尝试使用中文简体，如果失败则使用英文
    let languages = vec!["chi_sim+eng", "chi_sim", "eng"];

    for lang in &languages {
        debug!("[perform_tesseract_ocr] 尝试语言: {}", lang);
        match tesseract::Tesseract::new(None, Some(lang)) {
            Ok(tess) => match tess.set_image(image_path) {
                Ok(mut tess) => match tess.get_text() {
                    Ok(text) => {
                        info!(
                            "[perform_tesseract_ocr] OCR 成功，语言: {}, 文本长度: {}",
                            lang,
                            text.len()
                        );
                        debug!("[perform_tesseract_ocr] 识别文本: {}", text);
                        return Ok(text);
                    }
                    Err(e) => {
                        debug!("[perform_tesseract_ocr] 获取文本失败 ({}): {}", lang, e);
                    }
                },
                Err(e) => {
                    debug!("[perform_tesseract_ocr] 设置图片失败 ({}): {}", lang, e);
                }
            },
            Err(e) => {
                debug!(
                    "[perform_tesseract_ocr] 初始化 Tesseract 失败 ({}): {}",
                    lang, e
                );
            }
        }
    }

    Err(anyhow!(
        "Tesseract OCR 失败，请确保已安装 Tesseract 和中文语言包"
    ))
}

/// 识别图片并返回单个账单结果
pub fn recognize_image(image_path: &str) -> Result<OcrResult> {
    info!("[recognize_image] 开始识别图片: {}", image_path);

    // 读取并预处理图片
    debug!("[recognize_image] 打开图片文件");
    let img = image::open(image_path).context("无法打开图片文件")?;

    debug!(
        "[recognize_image] 图片尺寸: {}x{}",
        img.width(),
        img.height()
    );

    // 预处理图片以提高 OCR 效果
    debug!("[recognize_image] 预处理图片");
    let processed_path = preprocess_for_ocr(img, image_path)?;

    // 执行 OCR
    let raw_text = match perform_tesseract_ocr(&processed_path) {
        Ok(text) => text,
        Err(e) => {
            error!("[recognize_image] OCR 失败: {}", e);
            // 如果 OCR 失败，返回占位符结果
            return Ok(OcrResult {
                r#type: "expense".to_string(),
                amount: 0.0,
                description: "OCR识别失败，请手动输入".to_string(),
                bill_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                raw_text: format!("OCR错误: {}", e),
            });
        }
    };

    // 先进行文本修正（处理 OCR 常见错误）
    let corrected_text = correct_ocr_text(&raw_text);
    debug!("[recognize_image] 修正后文本: {}", corrected_text);

    // 解析文本
    let result = parse_bill_text(&corrected_text, &raw_text)?;

    info!(
        "[recognize_image] 识别结果: type={}, amount={}, date={}, desc={}",
        result.r#type, result.amount, result.bill_date, result.description
    );

    Ok(result)
}

/// 修正 OCR 常见错误
fn correct_ocr_text(text: &str) -> String {
    let mut corrected = text.to_string();

    // 修正货币符号
    corrected = corrected.replace("Y", "¥");
    corrected = corrected.replace("y", "¥");
    corrected = corrected.replace("￡", "¥");
    corrected = corrected.replace("$", "¥");

    // 修正常见的中文 OCR 错误
    let corrections = vec![
        ("要饮", "餐饮"),
        ("要饭", "餐饮"),
        ("饮食", "餐饮"),
        ("伙食", "餐饮"),
        ("转帐", "转账"),
        ("专账", "转账"),
        ("转赃", "转账"),
        ("购勿", "购物"),
        ("狗物", "购物"),
        ("构物", "购物"),
        ("交道", "交通"),
        ("交逝", "交通"),
        ("其它", "其他"),
        ("运功", "运动"),
        ("服夯", "服务"),
        ("眼务", "服务"),
        ("医辽", "医疗"),
        ("娛乐", "娱乐"),
        ("住居", "住房"),
        ("教有", "教育"),
        ("工贤", "工资"),
        ("工员", "工资"),
        ("收人", "收入"),
        ("双入", "收入"),
        ("支山", "支出"),
        ("外売", "外卖"),
        ("趣市", "超市"),
        ("起市", "超市"),
    ];

    for (wrong, right) in corrections {
        corrected = corrected.replace(wrong, right);
    }

    // 修正金额格式：将 《212000> 这样的格式修正为 ¥2120.00
    // 模式: 《或<开头，数字，>或》结尾
    let amount_fix_re = Regex::new(r"[《<](\d+)(\d{2})[>》]").unwrap();
    corrected = amount_fix_re
        .replace_all(&corrected, |caps: &regex::Captures| {
            let main = &caps[1];
            let decimal = &caps[2];
            format!("¥{}.{}", main, decimal)
        })
        .to_string();

    // 修正连续数字中间的小数点丢失：如 212000 -> 2120.00
    // 针对常见金额格式
    let fix_decimal_re = Regex::new(r"¥(\d+)(\d{2})(?:\s|$|[^.\d])").unwrap();
    corrected = fix_decimal_re
        .replace_all(&corrected, |caps: &regex::Captures| {
            let main = &caps[1];
            let decimal = &caps[2];
            format!("¥{}.{} ", main, decimal)
        })
        .to_string();

    corrected
}

/// 模糊匹配分类名称
/// 返回数据库中的完整分类名称
fn fuzzy_match_category(text: &str) -> Option<&'static str> {
    let categories: Vec<(&str, Vec<&str>)> = vec![
        // 支出分类（消费场景）
        (
            "餐饮美食",
            vec![
                "餐饮", "美食", "要饮", "要饭", "饮食", "伙食", "拍餐", "外卖", "外売", "餐厅",
                "饭店", "食堂", "咖啡", "奶茶", "小吃", "火锅", "烧烤", "甜品", "早餐", "午餐",
                "晚餐", "快餐", "中餐", "西餐", "日料", "韩料",
            ],
        ),
        (
            "交通出行",
            vec![
                "交通",
                "交道",
                "交逝",
                "出行",
                "打车",
                "地铁",
                "公交",
                "出租车",
                "网约车",
                "火车",
                "飞机",
                "机票",
                "高铁",
                "滴滴",
                "高德",
                "百度地图",
                "共享单车",
                "摩拜",
                "哈啰",
                "青桔",
                "地铁卡",
                "公交卡",
                "一卡通",
            ],
        ),
        (
            "购物消费",
            vec![
                "购物",
                "购勿",
                "狗物",
                "构物",
                "王网",
                "超市",
                "趣市",
                "起市",
                "商场",
                "商店",
                "购买",
                "电商",
                "淘宝",
                "京东",
                "拼多多",
                "天猫",
                "苏宁",
                "唯品会",
                "考拉",
                "网易严选",
                "盒马",
                "永辉",
                "大润发",
            ],
        ),
        (
            "生活服务",
            vec![
                "服务", "服夯", "眼务", "美容", "美发", "理发", "家政", "快递", "物流", "维修",
                "洗车", "干洗", "按摩", "足疗", "SPA", "美甲", "美睫", "纹眉", "保洁",
            ],
        ),
        (
            "充值缴费",
            vec![
                "充值",
                "缴费",
                "话费",
                "手机",
                "水电",
                "水费",
                "电费",
                "燃气",
                "宽带",
                "固话",
                "话费充值",
                "流量",
                "宽带费",
                "网费",
                "固话费",
                "燃气费",
                "水电气",
            ],
        ),
        (
            "医疗健康",
            vec![
                "医疗", "医辽", "医院", "药店", "体检", "健康", "看病", "挂号", "买药", "药品",
                "诊所", "门诊",
            ],
        ),
        (
            "旅行住宿",
            vec![
                "旅行",
                "旅游",
                "住宿",
                "酒店",
                "民宿",
                "景区",
                "门票",
                "携程",
                "去哪儿",
                "飞猪",
                "同程",
                "马蜂窝",
                "途牛",
                "艺龙",
                "如家",
                "汉庭",
                "7天",
            ],
        ),
        (
            "爱车养车",
            vec![
                "加油",
                "保养",
                "停车",
                "过路",
                "车险",
                "洗车",
                "修车",
                "加油卡",
                "ETC",
                "违章",
                "罚款",
                "年检",
                "保险",
                "4S店",
                "汽修",
            ],
        ),
        (
            "母婴亲子",
            vec![
                "母婴",
                "亲子",
                "儿童",
                "宝宝",
                "早教",
                "乐园",
                "奶粉",
                "尿不湿",
                "玩具",
                "童装",
                "婴儿用品",
                "儿童乐园",
                "早教班",
            ],
        ),
        (
            "教育培训",
            vec![
                "教育",
                "教有",
                "培训",
                "课程",
                "教材",
                "考试",
                "学习",
                "学费",
                "培训费",
                "考试费",
                "报名费",
                "辅导班",
                "培训班",
            ],
        ),
        (
            "娱乐休闲",
            vec![
                "娱乐",
                "娛乐",
                "休闲",
                "电影",
                "KTV",
                "游戏",
                "演出",
                "K歌",
                "酒吧",
                "桌游",
                "密室",
                "剧本杀",
                "网吧",
                "台球",
                "保龄球",
            ],
        ),
        (
            "住房物业",
            vec![
                "住房",
                "住居",
                "房租",
                "物业",
                "装修",
                "家具",
                "租房",
                "押金",
                "物业费",
                "房租费",
                "装修费",
                "家具费",
                "水电费",
            ],
        ),
        (
            "通讯物流",
            vec![
                "通讯",
                "通信",
                "物流",
                "快递",
                "邮寄",
                "顺丰",
                "圆通",
                "中通",
                "申通",
                "韵达",
                "邮政",
                "EMS",
                "京东物流",
                "菜鸟",
            ],
        ),
        (
            "运动健康",
            vec![
                "运动",
                "运功",
                "轿动",
                "健身",
                "跑步",
                "运动装备",
                "体育",
                "健身房",
                "瑜伽",
                "游泳",
                "羽毛球",
                "乒乓球",
                "篮球",
                "足球",
            ],
        ),
        // 支出分类（交易类型）
        (
            "转账支出",
            vec![
                "转账",
                "转帐",
                "专账",
                "自第",
                "白第",
                "转出",
                "转给",
                "付款",
                "支付给",
            ],
        ),
        ("红包支出", vec!["红包", "发红包", "红包支出", "发红包给"]),
        (
            "商户消费",
            vec![
                "商户",
                "消费",
                "支付",
                "扫码支付",
                "刷卡",
                "POS",
                "商户支付",
            ],
        ),
        (
            "群收款",
            vec!["群收款", "群收", "AA收款", "AA", "群收款支付"],
        ),
        (
            "信用卡还款",
            vec!["信用卡", "还款", "还信用卡", "信用卡还款"],
        ),
        (
            "生活缴费",
            vec!["缴费", "水费", "电费", "燃气费", "物业费", "生活缴费"],
        ),
        (
            "充值提现",
            vec!["充值", "提现", "零钱充值", "提现到银行卡", "银行卡提现"],
        ),
        (
            "金融理财",
            vec!["理财", "投资", "金融", "余额宝", "基金", "股票", "理财通"],
        ),
        // 收入分类
        ("工资", vec!["工资", "工贤", "工员", "薪资", "薪水", "月薪"]),
        ("奖金", vec!["奖金", "奖励", "绩效", "年终奖", "绩效奖金"]),
        (
            "转账收入",
            vec!["转账", "转入", "收款", "收到", "转账收入", "收到转账"],
        ),
        (
            "红包收入",
            vec!["红包", "收红包", "红包收入", "抢红包", "收到红包"],
        ),
        (
            "退款收入",
            vec!["退款", "退费", "退回", "退款收入", "退货退款"],
        ),
        (
            "投资收益",
            vec!["投资", "收益", "分红", "投资收益", "股票收益"],
        ),
        (
            "理财收益",
            vec!["理财", "收益", "利息", "理财收益", "余额宝收益"],
        ),
        // 默认分类
        ("其他支出", vec!["其他", "其它"]),
        ("其他收入", vec!["其他收入", "其它收入"]),
    ];

    let text_lower = text.to_lowercase();

    for (category, variants) in &categories {
        for variant in variants {
            if text_lower.contains(&variant.to_lowercase()) {
                return Some(category);
            }
        }
    }

    None
}

/// 从文本中提取所有金额
fn extract_all_amounts(text: &str) -> Vec<f64> {
    let mut amounts = Vec::new();

    // 标准格式: ¥123.45 或 ￥123.45
    let pattern1 = Regex::new(r"[¥￥]\s*(\d+\.?\d*)").unwrap();
    for caps in pattern1.captures_iter(text) {
        if let Some(m) = caps.get(1) {
            if let Ok(amount) = m.as_str().parse::<f64>() {
                if amount > 0.0 && amount < 1000000.0 {
                    amounts.push(amount);
                }
            }
        }
    }

    // 格式: 123.45 (两位小数的数字)
    let pattern2 = Regex::new(r"(\d+\.\d{2})").unwrap();
    for caps in pattern2.captures_iter(text) {
        if let Some(m) = caps.get(1) {
            if let Ok(amount) = m.as_str().parse::<f64>() {
                if amount > 0.0 && amount < 1000000.0 && !amounts.contains(&amount) {
                    amounts.push(amount);
                }
            }
        }
    }

    // 格式: 123元
    let pattern3 = Regex::new(r"(\d+\.?\d*)\s*元").unwrap();
    for caps in pattern3.captures_iter(text) {
        if let Some(m) = caps.get(1) {
            if let Ok(amount) = m.as_str().parse::<f64>() {
                if amount > 0.0 && amount < 1000000.0 && !amounts.contains(&amount) {
                    amounts.push(amount);
                }
            }
        }
    }

    amounts.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    amounts
}

/// 识别图片并返回多个账单明细
pub fn recognize_bill_details(image_path: &str) -> Result<OcrDetailResult> {
    info!("[recognize_bill_details] 开始识别账单明细: {}", image_path);

    // 读取并预处理图片
    let img = image::open(image_path).context("无法打开图片文件")?;

    let processed_path = preprocess_for_ocr(img, image_path)?;

    // 执行 OCR
    let raw_text = perform_tesseract_ocr(&processed_path)?;

    // 修正文本
    let corrected_text = correct_ocr_text(&raw_text);

    // 解析账单明细
    let items = parse_bill_items(&corrected_text);
    let total_amount: f64 = items.iter().map(|i| i.amount).sum();

    // 尝试提取日期
    let bill_date = extract_date_from_text(&corrected_text)
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    info!(
        "[recognize_bill_details] 识别到 {} 个账单项，总金额: {}, 日期: {}",
        items.len(),
        total_amount,
        bill_date
    );

    Ok(OcrDetailResult {
        items,
        total_amount,
        raw_text,
        bill_date,
    })
}

/// 从文本中提取日期
fn extract_date_from_text(text: &str) -> Option<String> {
    let date_patterns = vec![
        r"(\d{4})[年\-/](\d{1,2})[月\-/](\d{1,2})",
        r"(\d{4})-(\d{2})-(\d{2})",
        r"(\d{4})/(\d{1,2})/(\d{1,2})",
    ];

    for pattern in &date_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if caps.len() >= 4 {
                    if let (Some(year), Some(month), Some(day)) =
                        (caps.get(1), caps.get(2), caps.get(3))
                    {
                        let year_str = year.as_str();
                        let month_val = month.as_str().parse::<u32>().unwrap_or(1);
                        let day_val = day.as_str().parse::<u32>().unwrap_or(1);

                        if month_val >= 1 && month_val <= 12 && day_val >= 1 && day_val <= 31 {
                            return Some(format!("{}-{:02}-{:02}", year_str, month_val, day_val));
                        }
                    }
                }
            }
        }
    }
    None
}

/// 判断分类是否为收入类型
fn is_income_category(category: &str) -> bool {
    let income_categories = vec![
        "工资",
        "奖金",
        "转账收入",
        "红包收入",
        "退款收入",
        "投资收益",
        "理财收益",
        "其他收入",
    ];
    income_categories
        .iter()
        .any(|c| category == *c || category.contains("收入"))
}

/// 从文本中提取所有金额及其位置
fn extract_amounts_with_positions(text: &str) -> Vec<(f64, usize)> {
    let mut results = Vec::new();

    // 标准格式: ¥123.45 或 ￥123.45
    if let Ok(re) = Regex::new(r"[¥￥]\s*(\d+\.?\d*)") {
        for caps in re.captures_iter(text) {
            if let (Some(m), Some(full)) = (caps.get(1), caps.get(0)) {
                if let Ok(amount) = m.as_str().parse::<f64>() {
                    if amount > 0.0 && amount < 1000000.0 {
                        results.push((amount, full.start()));
                    }
                }
            }
        }
    }

    // 格式: 123.45 (两位小数的数字，通常是金额)
    if let Ok(re) = Regex::new(r"(\d+\.\d{2})") {
        for caps in re.captures_iter(text) {
            if let (Some(m), Some(full)) = (caps.get(1), caps.get(0)) {
                if let Ok(amount) = m.as_str().parse::<f64>() {
                    // 排除百分比（后面跟着%）和已存在的金额
                    let pos = full.start();
                    let after = &text[full.end()..];
                    if amount > 0.0
                        && amount < 1000000.0
                        && !after.starts_with('%')
                        && !results.iter().any(|(a, _)| (*a - amount).abs() < 0.01)
                    {
                        results.push((amount, pos));
                    }
                }
            }
        }
    }

    // 按位置排序
    results.sort_by_key(|(_, pos)| *pos);
    results
}

/// 在文本中查找金额附近的分类
/// 返回数据库中的完整分类名称
fn find_category_near_amount(text: &str, amount_pos: usize) -> Option<&'static str> {
    // 定义关键词到数据库分类名称的映射
    // 格式: (数据库分类名称, 关键词列表)
    let categories: Vec<(&str, Vec<&str>)> = vec![
        // 支出分类（消费场景）
        (
            "餐饮美食",
            vec![
                "餐饮", "美食", "要饮", "要饭", "饮食", "伙食", "拍餐", "外卖", "外売", "餐厅",
                "饭店", "食堂", "咖啡", "奶茶", "小吃", "火锅", "烧烤", "甜品", "早餐", "午餐",
                "晚餐", "快餐", "中餐", "西餐", "日料", "韩料",
            ],
        ),
        (
            "交通出行",
            vec![
                "交通",
                "交道",
                "交逝",
                "出行",
                "打车",
                "地铁",
                "公交",
                "出租车",
                "网约车",
                "火车",
                "飞机",
                "机票",
                "高铁",
                "滴滴",
                "高德",
                "百度地图",
                "共享单车",
                "摩拜",
                "哈啰",
                "青桔",
                "地铁卡",
                "公交卡",
                "一卡通",
            ],
        ),
        (
            "购物消费",
            vec![
                "购物",
                "购勿",
                "狗物",
                "构物",
                "王网",
                "超市",
                "趣市",
                "起市",
                "商场",
                "商店",
                "购买",
                "电商",
                "淘宝",
                "京东",
                "拼多多",
                "天猫",
                "苏宁",
                "唯品会",
                "考拉",
                "网易严选",
                "盒马",
                "永辉",
                "大润发",
            ],
        ),
        (
            "生活服务",
            vec![
                "服务", "服夯", "眼务", "美容", "美发", "理发", "家政", "快递", "物流", "维修",
                "洗车", "干洗", "按摩", "足疗", "SPA", "美甲", "美睫", "纹眉", "保洁",
            ],
        ),
        (
            "充值缴费",
            vec![
                "充值",
                "缴费",
                "话费",
                "手机",
                "水电",
                "水费",
                "电费",
                "燃气",
                "宽带",
                "固话",
                "话费充值",
                "流量",
                "宽带费",
                "网费",
                "固话费",
                "燃气费",
                "水电气",
            ],
        ),
        (
            "医疗健康",
            vec![
                "医疗", "医辽", "医院", "药店", "体检", "健康", "看病", "挂号", "买药", "药品",
                "诊所", "门诊",
            ],
        ),
        (
            "旅行住宿",
            vec![
                "旅行",
                "旅游",
                "住宿",
                "酒店",
                "民宿",
                "景区",
                "门票",
                "携程",
                "去哪儿",
                "飞猪",
                "同程",
                "马蜂窝",
                "途牛",
                "艺龙",
                "如家",
                "汉庭",
                "7天",
            ],
        ),
        (
            "爱车养车",
            vec![
                "加油",
                "保养",
                "停车",
                "过路",
                "车险",
                "洗车",
                "修车",
                "加油卡",
                "ETC",
                "违章",
                "罚款",
                "年检",
                "保险",
                "4S店",
                "汽修",
            ],
        ),
        (
            "母婴亲子",
            vec![
                "母婴",
                "亲子",
                "儿童",
                "宝宝",
                "早教",
                "乐园",
                "奶粉",
                "尿不湿",
                "玩具",
                "童装",
                "婴儿用品",
                "儿童乐园",
                "早教班",
            ],
        ),
        (
            "教育培训",
            vec![
                "教育",
                "教有",
                "培训",
                "课程",
                "教材",
                "考试",
                "学习",
                "学费",
                "培训费",
                "考试费",
                "报名费",
                "辅导班",
                "培训班",
            ],
        ),
        (
            "娱乐休闲",
            vec![
                "娱乐",
                "娛乐",
                "休闲",
                "电影",
                "KTV",
                "游戏",
                "演出",
                "K歌",
                "酒吧",
                "桌游",
                "密室",
                "剧本杀",
                "网吧",
                "台球",
                "保龄球",
            ],
        ),
        (
            "住房物业",
            vec![
                "住房",
                "住居",
                "房租",
                "物业",
                "装修",
                "家具",
                "租房",
                "押金",
                "物业费",
                "房租费",
                "装修费",
                "家具费",
                "水电费",
            ],
        ),
        (
            "通讯物流",
            vec![
                "通讯",
                "通信",
                "物流",
                "快递",
                "邮寄",
                "顺丰",
                "圆通",
                "中通",
                "申通",
                "韵达",
                "邮政",
                "EMS",
                "京东物流",
                "菜鸟",
            ],
        ),
        (
            "运动健康",
            vec![
                "运动",
                "运功",
                "轿动",
                "健身",
                "跑步",
                "运动装备",
                "体育",
                "健身房",
                "瑜伽",
                "游泳",
                "羽毛球",
                "乒乓球",
                "篮球",
                "足球",
            ],
        ),
        // 支出分类（交易类型）
        (
            "转账支出",
            vec![
                "转账",
                "转帐",
                "专账",
                "自第",
                "白第",
                "转出",
                "转给",
                "付款",
                "支付给",
            ],
        ),
        ("红包支出", vec!["红包", "发红包", "红包支出", "发红包给"]),
        (
            "商户消费",
            vec![
                "商户",
                "消费",
                "支付",
                "扫码支付",
                "刷卡",
                "POS",
                "商户支付",
            ],
        ),
        (
            "群收款",
            vec!["群收款", "群收", "AA收款", "AA", "群收款支付"],
        ),
        (
            "信用卡还款",
            vec!["信用卡", "还款", "还信用卡", "信用卡还款"],
        ),
        (
            "生活缴费",
            vec!["缴费", "水费", "电费", "燃气费", "物业费", "生活缴费"],
        ),
        (
            "充值提现",
            vec!["充值", "提现", "零钱充值", "提现到银行卡", "银行卡提现"],
        ),
        (
            "金融理财",
            vec!["理财", "投资", "金融", "余额宝", "基金", "股票", "理财通"],
        ),
        // 收入分类
        ("工资", vec!["工资", "工贤", "工员", "薪资", "薪水", "月薪"]),
        ("奖金", vec!["奖金", "奖励", "绩效", "年终奖", "绩效奖金"]),
        (
            "转账收入",
            vec!["转账", "转入", "收款", "收到", "转账收入", "收到转账"],
        ),
        (
            "红包收入",
            vec!["红包", "收红包", "红包收入", "抢红包", "收到红包"],
        ),
        (
            "退款收入",
            vec!["退款", "退费", "退回", "退款收入", "退货退款"],
        ),
        (
            "投资收益",
            vec!["投资", "收益", "分红", "投资收益", "股票收益"],
        ),
        (
            "理财收益",
            vec!["理财", "收益", "利息", "理财收益", "余额宝收益"],
        ),
        // 默认分类
        ("其他支出", vec!["其他", "其它"]),
        ("其他收入", vec!["其他收入", "其它收入"]),
    ];

    // 搜索范围：优先在金额位置前后50字符内搜索（更精确）
    // 如果50字符内找不到，再扩大到200字符
    // 使用字符边界安全的切片方式（处理多字节UTF-8字符）

    // 首先尝试小范围搜索（前后50字符）
    let search_start_small = {
        let target = amount_pos.saturating_sub(50);
        let mut start = target;
        if start == 0 {
            0
        } else {
            while start > 0 && !text.is_char_boundary(start) {
                start -= 1;
            }
            start
        }
    };
    let search_end_small = {
        let target = (amount_pos + 50).min(text.len());
        let mut end = target;
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        end.min(text.len())
    };
    let search_start_small = search_start_small.min(search_end_small);
    let search_text_small = &text[search_start_small..search_end_small];

    // 先在小范围内搜索
    for (category, variants) in &categories {
        for variant in variants {
            if search_text_small.contains(variant) {
                debug!(
                    "[find_category_near_amount] 在小范围（50字符）内找到分类: {} (关键词: {})",
                    category, variant
                );
                return Some(category);
            }
        }
    }

    // 如果小范围没找到，再扩大范围搜索（前后200字符）
    let search_start = {
        let target = amount_pos.saturating_sub(200);
        let mut start = target;
        if start == 0 {
            0
        } else {
            while start > 0 && !text.is_char_boundary(start) {
                start -= 1;
            }
            start
        }
    };
    let search_end = {
        let target = (amount_pos + 200).min(text.len());
        let mut end = target;
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        end.min(text.len())
    };

    // 确保 search_start <= search_end
    let search_start = search_start.min(search_end);
    let search_text = &text[search_start..search_end];
    let search_text_start_pos = search_start;

    // 改进匹配算法：优先选择距离金额最近的关键词
    // 记录每个分类及其最近匹配位置
    let mut best_match: Option<(&str, usize)> = None;
    let mut min_distance = usize::MAX;

    for (category, variants) in &categories {
        // 跳过"其他支出"和"其他收入"，它们作为最后的后备选项
        if *category == "其他支出" || *category == "其他收入" {
            continue;
        }

        for variant in variants {
            // 在搜索文本中查找所有匹配位置
            let mut search_pos = 0;
            loop {
                // 确保 search_pos 是字符边界
                while search_pos < search_text.len() && !search_text.is_char_boundary(search_pos) {
                    search_pos += 1;
                }
                if search_pos >= search_text.len() {
                    break;
                }

                // 在剩余文本中查找关键词
                if let Some(relative_pos) = search_text[search_pos..].find(variant) {
                    let absolute_pos = search_text_start_pos + search_pos + relative_pos;

                    // 计算关键词中心位置到金额位置的距离
                    let variant_center = absolute_pos + variant.len() / 2;
                    let distance = if variant_center > amount_pos {
                        variant_center - amount_pos
                    } else {
                        amount_pos - variant_center
                    };

                    // 如果这个匹配更近，更新最佳匹配
                    if distance < min_distance {
                        min_distance = distance;
                        best_match = Some((category, absolute_pos));
                    }

                    // 继续搜索下一个匹配（移动到匹配位置之后）
                    search_pos += relative_pos + 1;
                } else {
                    break;
                }
            }
        }
    }

    // 返回最佳匹配的分类
    best_match.map(|(cat, _)| cat)
}

/// 在整个文本中查找分类（用于处理分类信息在文本顶部的情况）
/// 优先选择在金额位置之前的分类，但限制搜索范围避免匹配到太远的分类
fn find_category_in_text(text: &str, amount_pos: usize) -> Option<&'static str> {
    // 限制搜索范围：只在金额位置之前500字符内搜索
    // 这样可以避免匹配到文本开头太远的分类（如"购物 32.88%"）
    let max_search_range = 500;
    let search_start = {
        let target = amount_pos.saturating_sub(max_search_range);
        let mut start = target;
        if start == 0 {
            0
        } else {
            while start > 0 && !text.is_char_boundary(start) {
                start -= 1;
            }
            start
        }
    };
    let search_end = amount_pos;
    let search_text = &text[search_start..search_end];
    // 使用与 find_category_near_amount 相同的分类列表
    let categories: Vec<(&str, Vec<&str>)> = vec![
        // 支出分类（消费场景）
        (
            "餐饮美食",
            vec![
                "餐饮", "美食", "要饮", "要饭", "饮食", "伙食", "拍餐", "外卖", "外売", "餐厅",
                "饭店", "食堂", "咖啡", "奶茶", "小吃", "火锅", "烧烤", "甜品", "早餐", "午餐",
                "晚餐", "快餐", "中餐", "西餐", "日料", "韩料",
            ],
        ),
        (
            "交通出行",
            vec![
                "交通",
                "交道",
                "交逝",
                "出行",
                "打车",
                "地铁",
                "公交",
                "出租车",
                "网约车",
                "火车",
                "飞机",
                "机票",
                "高铁",
                "滴滴",
                "高德",
                "百度地图",
                "共享单车",
                "摩拜",
                "哈啰",
                "青桔",
                "地铁卡",
                "公交卡",
                "一卡通",
            ],
        ),
        (
            "购物消费",
            vec![
                "购物",
                "购勿",
                "狗物",
                "构物",
                "王网",
                "超市",
                "趣市",
                "起市",
                "商场",
                "商店",
                "购买",
                "电商",
                "淘宝",
                "京东",
                "拼多多",
                "天猫",
                "苏宁",
                "唯品会",
                "考拉",
                "网易严选",
                "盒马",
                "永辉",
                "大润发",
            ],
        ),
        (
            "生活服务",
            vec![
                "服务", "服夯", "眼务", "美容", "美发", "理发", "家政", "快递", "物流", "维修",
                "洗车", "干洗", "按摩", "足疗", "SPA", "美甲", "美睫", "纹眉", "保洁",
            ],
        ),
        (
            "充值缴费",
            vec![
                "充值",
                "缴费",
                "话费",
                "手机",
                "水电",
                "水费",
                "电费",
                "燃气",
                "宽带",
                "固话",
                "话费充值",
                "流量",
                "宽带费",
                "网费",
                "固话费",
                "燃气费",
                "水电气",
            ],
        ),
        (
            "医疗健康",
            vec![
                "医疗", "医辽", "医院", "药店", "体检", "健康", "看病", "挂号", "买药", "药品",
                "诊所", "门诊",
            ],
        ),
        (
            "旅行住宿",
            vec![
                "旅行",
                "旅游",
                "住宿",
                "酒店",
                "民宿",
                "景区",
                "门票",
                "携程",
                "去哪儿",
                "飞猪",
                "同程",
                "马蜂窝",
                "途牛",
                "艺龙",
                "如家",
                "汉庭",
                "7天",
            ],
        ),
        (
            "爱车养车",
            vec![
                "加油",
                "保养",
                "停车",
                "过路",
                "车险",
                "洗车",
                "修车",
                "加油卡",
                "ETC",
                "违章",
                "罚款",
                "年检",
                "保险",
                "4S店",
                "汽修",
            ],
        ),
        (
            "母婴亲子",
            vec![
                "母婴",
                "亲子",
                "儿童",
                "宝宝",
                "早教",
                "乐园",
                "奶粉",
                "尿不湿",
                "玩具",
                "童装",
                "婴儿用品",
                "儿童乐园",
                "早教班",
            ],
        ),
        (
            "教育培训",
            vec![
                "教育",
                "教有",
                "培训",
                "课程",
                "教材",
                "考试",
                "学习",
                "学费",
                "培训费",
                "考试费",
                "报名费",
                "辅导班",
                "培训班",
            ],
        ),
        (
            "娱乐休闲",
            vec![
                "娱乐",
                "娛乐",
                "休闲",
                "电影",
                "KTV",
                "游戏",
                "演出",
                "K歌",
                "酒吧",
                "桌游",
                "密室",
                "剧本杀",
                "网吧",
                "台球",
                "保龄球",
            ],
        ),
        (
            "住房物业",
            vec![
                "住房",
                "住居",
                "房租",
                "物业",
                "装修",
                "家具",
                "租房",
                "押金",
                "物业费",
                "房租费",
                "装修费",
                "家具费",
                "水电费",
            ],
        ),
        (
            "通讯物流",
            vec![
                "通讯",
                "通信",
                "物流",
                "快递",
                "邮寄",
                "顺丰",
                "圆通",
                "中通",
                "申通",
                "韵达",
                "邮政",
                "EMS",
                "京东物流",
                "菜鸟",
            ],
        ),
        (
            "运动健康",
            vec![
                "运动",
                "运功",
                "轿动",
                "健身",
                "跑步",
                "运动装备",
                "体育",
                "健身房",
                "瑜伽",
                "游泳",
                "羽毛球",
                "乒乓球",
                "篮球",
                "足球",
            ],
        ),
        // 支出分类（交易类型）
        (
            "转账支出",
            vec![
                "转账",
                "转帐",
                "专账",
                "自第",
                "白第",
                "转出",
                "转给",
                "付款",
                "支付给",
            ],
        ),
        ("红包支出", vec!["红包", "发红包", "红包支出", "发红包给"]),
        (
            "商户消费",
            vec![
                "商户",
                "消费",
                "支付",
                "扫码支付",
                "刷卡",
                "POS",
                "商户支付",
            ],
        ),
        (
            "群收款",
            vec!["群收款", "群收", "AA收款", "AA", "群收款支付"],
        ),
        (
            "信用卡还款",
            vec!["信用卡", "还款", "还信用卡", "信用卡还款"],
        ),
        (
            "生活缴费",
            vec!["缴费", "水费", "电费", "燃气费", "物业费", "生活缴费"],
        ),
        (
            "充值提现",
            vec!["充值", "提现", "零钱充值", "提现到银行卡", "银行卡提现"],
        ),
        (
            "金融理财",
            vec!["理财", "投资", "金融", "余额宝", "基金", "股票", "理财通"],
        ),
        // 收入分类
        ("工资", vec!["工资", "工贤", "工员", "薪资", "薪水", "月薪"]),
        ("奖金", vec!["奖金", "奖励", "绩效", "年终奖", "绩效奖金"]),
        (
            "转账收入",
            vec!["转账", "转入", "收款", "收到", "转账收入", "收到转账"],
        ),
        (
            "红包收入",
            vec!["红包", "收红包", "红包收入", "抢红包", "收到红包"],
        ),
        (
            "退款收入",
            vec!["退款", "退费", "退回", "退款收入", "退货退款"],
        ),
        (
            "投资收益",
            vec!["投资", "收益", "分红", "投资收益", "股票收益"],
        ),
        (
            "理财收益",
            vec!["理财", "收益", "利息", "理财收益", "余额宝收益"],
        ),
        // 默认分类（放在最后，优先级最低）
        ("其他支出", vec!["其他", "其它"]),
        ("其他收入", vec!["其他收入", "其它收入"]),
    ];

    // 优先查找在金额位置之前的分类（更可能是该金额的分类）
    // 记录最佳匹配：优先选择在金额之前的，距离最近的
    let mut best_match_before: Option<(&str, usize)> = None;
    let mut min_distance_before = usize::MAX;

    for (category, variants) in &categories {
        // 跳过"其他支出"和"其他收入"，它们作为最后的后备选项
        if *category == "其他支出" || *category == "其他收入" {
            continue;
        }

        for variant in variants {
            let mut search_pos = 0;
            loop {
                // 确保 search_pos 是字符边界
                while search_pos < search_text.len() && !search_text.is_char_boundary(search_pos) {
                    search_pos += 1;
                }
                if search_pos >= search_text.len() {
                    break;
                }

                // 在搜索文本中查找关键词（只在金额之前搜索）
                if let Some(relative_pos) = search_text[search_pos..].find(variant) {
                    let absolute_pos = search_start + search_pos + relative_pos;

                    // 计算距离（只考虑在金额之前的）
                    let distance = amount_pos - absolute_pos;

                    // 只记录在金额之前的匹配
                    if distance < min_distance_before {
                        min_distance_before = distance;
                        best_match_before = Some((category, absolute_pos));
                    }

                    // 继续搜索下一个匹配
                    search_pos += relative_pos + 1;
                } else {
                    break;
                }
            }
        }
    }

    // 返回在金额之前的匹配
    best_match_before.map(|(cat, _)| cat)
}

/// 提取"分类 百分比"格式的行
/// 返回 (分类名称, 行位置, 百分比)
fn extract_category_lines(text: &str) -> Vec<(&'static str, usize, f64)> {
    let mut results = Vec::new();
    let categories: Vec<(&str, Vec<&str>)> = vec![
        ("交通出行", vec!["交通", "交道", "交逝"]),
        ("购物消费", vec!["购物", "购勿", "狗物", "构物"]),
        ("餐饮美食", vec!["餐饮", "要饮", "要饭"]),
        ("其他支出", vec!["其他", "其它"]),
        ("生活服务", vec!["服务", "服夯", "眼务"]),
        ("充值缴费", vec!["充值", "缴费"]),
        ("医疗健康", vec!["医疗", "医辽"]),
        ("旅行住宿", vec!["旅行", "旅游"]),
        ("爱车养车", vec!["加油", "保养"]),
        ("母婴亲子", vec!["母婴", "亲子"]),
        ("教育培训", vec!["教育", "教有"]),
        ("娱乐休闲", vec!["娱乐", "娛乐"]),
        ("住房物业", vec!["住房", "住居"]),
        ("通讯物流", vec!["通讯", "通信"]),
        ("运动健康", vec!["运动", "运功"]),
    ];

    // 按行处理文本
    let mut current_pos = 0;
    for line in text.lines() {
        let line_start = current_pos;
        let line_text = line.trim();
        current_pos += line.len() + 1; // +1 for newline

        if line_text.is_empty() {
            continue;
        }

        // 尝试匹配"分类 百分比"格式（如"交通 10.70%"）
        for (category, variants) in &categories {
            for variant in variants {
                // 检查行是否包含分类关键词，后面跟着百分比
                if let Some(variant_pos) = line_text.find(variant) {
                    // 检查后面是否有百分比格式
                    let after_variant = &line_text[variant_pos + variant.len()..];
                    if let Ok(re) = Regex::new(r"(\d+\.?\d*)\s*%") {
                        if let Some(caps) = re.captures(after_variant) {
                            if let Some(percent_str) = caps.get(1) {
                                if let Ok(percent) = percent_str.as_str().parse::<f64>() {
                                    let absolute_pos = line_start + variant_pos;
                                    results.push((*category, absolute_pos, percent));
                                    debug!(
                                        "[extract_category_lines] 找到分类行: {} {}% 位置: {}",
                                        category, percent, absolute_pos
                                    );
                                    break; // 找到后跳出内层循环
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

/// 解析账单明细项（优化版：先提取所有金额，再关联分类）
fn parse_bill_items(text: &str) -> Vec<BillItem> {
    info!("[parse_bill_items] 开始解析账单明细（优化版）");
    let mut items = Vec::new();

    // 1. 提取所有金额及其位置
    let amounts_with_pos = extract_amounts_with_positions(text);
    info!(
        "[parse_bill_items] 提取到 {} 个金额",
        amounts_with_pos.len()
    );

    for (i, (amount, pos)) in amounts_with_pos.iter().enumerate() {
        debug!(
            "[parse_bill_items] 金额 {}: {} 位置: {}",
            i + 1,
            amount,
            pos
        );
    }

    // 2. 先尝试按行解析（识别"分类 百分比"格式的行）
    // 例如："交通 10.70%"、"购物 32.88%"等
    let category_lines = extract_category_lines(text);
    debug!(
        "[parse_bill_items] 识别到 {} 个分类行",
        category_lines.len()
    );
    for (i, (cat, pos, _)) in category_lines.iter().enumerate() {
        debug!("[parse_bill_items] 分类行 {}: {} 位置: {}", i + 1, cat, pos);
    }

    // 将分类行按位置排序，用于区间匹配
    let mut sorted_category_lines: Vec<_> = category_lines.iter().collect();
    sorted_category_lines.sort_by_key(|(_, pos, _)| *pos);

    // 3. 为每个金额尝试关联分类
    // 改进策略：
    // 1. 先尝试在金额附近（前后50字符）搜索
    // 2. 如果没找到，使用区间匹配（根据分类行的位置区间）
    // 3. 最后才在整个文本中搜索（限制范围）

    for (amount, pos) in &amounts_with_pos {
        // 首先尝试在金额附近搜索（前后50字符，更精确）
        let mut category = find_category_near_amount(text, *pos);

        // 如果附近没找到，尝试使用区间匹配
        if category.is_none() && !sorted_category_lines.is_empty() {
            // 区间匹配：找到金额所属的分类区间
            // 如果金额在某个分类行之后但在下一个分类行之前，属于前一个分类
            let mut found_interval = false;

            for i in 0..sorted_category_lines.len() {
                let (cat, line_pos, _) = sorted_category_lines[i];

                // 如果金额在这个分类行之后
                if *pos > *line_pos {
                    // 如果这是最后一个分类行，或者金额在下一个分类行之前
                    if i == sorted_category_lines.len() - 1 {
                        // 金额在最后一个分类行之后，属于最后一个分类
                        debug!(
                            "[parse_bill_items] 金额 {} 在最后一个分类行 {} 之后，匹配到: {}",
                            amount, line_pos, cat
                        );
                        category = Some(*cat);
                        found_interval = true;
                        break;
                    } else {
                        let next_line_pos = sorted_category_lines[i + 1].1;
                        // 如果金额在下一个分类行之前，属于当前分类
                        if *pos < next_line_pos {
                            debug!(
                                "[parse_bill_items] 金额 {} 在分类区间 [{}, {})，匹配到: {}",
                                amount, line_pos, next_line_pos, cat
                            );
                            category = Some(*cat);
                            found_interval = true;
                            break;
                        }
                    }
                }
            }

            // 如果区间匹配没找到，尝试找距离最近的分类行（向后兼容）
            if !found_interval {
                let mut best_category_line: Option<(&str, usize)> = None;
                let mut min_line_distance = usize::MAX;

                for (cat, line_pos, _) in &category_lines {
                    if *line_pos < *pos {
                        let distance = pos - line_pos;
                        if distance < min_line_distance {
                            min_line_distance = distance;
                            best_category_line = Some((cat, *line_pos));
                        }
                    }
                }

                if let Some((cat, _)) = best_category_line {
                    debug!(
                        "[parse_bill_items] 金额 {} 匹配到分类行: {} (距离: {})",
                        amount, cat, min_line_distance
                    );
                    category = Some(cat);
                }
            }
        }

        // 如果还是没找到，尝试在整个文本中搜索（限制范围）
        if category.is_none() {
            debug!(
                "[parse_bill_items] 金额 {} 位置 {} 附近未找到分类，尝试全局搜索",
                amount, pos
            );
            // 在整个文本中搜索，但限制搜索范围（只在金额之前500字符内）
            category = find_category_in_text(text, *pos);
        }

        let (cat_name, bill_type) = if let Some(cat) = category {
            let bt = if is_income_category(cat) {
                "income"
            } else {
                "expense"
            };
            (cat, bt)
        } else {
            // 无法识别分类，使用"其他支出"
            debug!(
                "[parse_bill_items] 金额 {} 位置 {} 未找到分类，使用默认分类",
                amount, pos
            );
            ("其他支出", "expense")
        };

        info!(
            "[parse_bill_items] 关联: {} = {} ({})",
            cat_name, amount, bill_type
        );

        items.push(BillItem {
            category: cat_name.to_string(),
            amount: *amount,
            percentage: None,
            bill_type: bill_type.to_string(),
        });
    }

    // 3. 基于百分比的验证和调整（如果有分类行）
    if !sorted_category_lines.is_empty() && !items.is_empty() {
        // 计算总金额
        let total_amount: f64 = items.iter().map(|item| item.amount).sum();
        debug!(
            "[parse_bill_items] 总金额: {}, 开始验证分类分配",
            total_amount
        );

        // 计算每个分类的实际金额总和
        let mut category_amounts: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for item in &items {
            *category_amounts.entry(item.category.clone()).or_insert(0.0) += item.amount;
        }

        // 验证每个分类行的百分比是否匹配，并尝试调整
        let mut adjustments_made = false;
        for (cat, _, percent) in &sorted_category_lines {
            let expected_amount = total_amount * (*percent / 100.0);
            let actual_amount = category_amounts.get(*cat).copied().unwrap_or(0.0);
            let diff = (expected_amount - actual_amount).abs();
            let diff_percent = if total_amount > 0.0 {
                (diff / total_amount) * 100.0
            } else {
                0.0
            };

            debug!(
                "[parse_bill_items] 分类 {}: 预期金额={:.2} ({}%), 实际金额={:.2}, 差异={:.2} ({:.2}%)",
                cat, expected_amount, percent, actual_amount, diff, diff_percent
            );

            // 如果差异超过15%，尝试重新分配
            // 策略：根据金额的顺序，将金额按顺序分配给分类行
            if diff_percent > 15.0 && total_amount > 0.0 {
                debug!(
                    "[parse_bill_items] 分类 {} 的金额分配与百分比差异较大，尝试重新分配",
                    cat
                );
                adjustments_made = true;
            }
        }

        // 如果发现较大的差异，记录警告但不强制重新分配
        // 因为重新分配可能会破坏已经正确匹配的分类
        // 用户可以在前端手动调整分类
        if adjustments_made {
            debug!("[parse_bill_items] 注意: 部分分类的金额分配与百分比差异较大，建议检查");
        }
    }

    // 4. 如果没有提取到任何金额，尝试按行处理（兼容旧逻辑）
    if items.is_empty() {
        debug!("[parse_bill_items] 金额提取失败，尝试按行识别");

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.len() < 2 {
                continue;
            }

            if let Some(category) = fuzzy_match_category(line) {
                let amounts = extract_all_amounts(line);
                if let Some(&amount) = amounts.first() {
                    if amount > 0.0 {
                        let bill_type = if is_income_category(category) {
                            "income"
                        } else {
                            "expense"
                        };
                        items.push(BillItem {
                            category: category.to_string(),
                            amount,
                            percentage: extract_percentage_from_line(line),
                            bill_type: bill_type.to_string(),
                        });
                    }
                }
            }
        }
    }

    // 4. 去除金额为0或重复的项目
    items.retain(|item| item.amount > 0.0);

    // 按金额降序排序（通常大金额更重要）
    items.sort_by(|a, b| {
        b.amount
            .partial_cmp(&a.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    info!("[parse_bill_items] 解析完成，共 {} 项", items.len());
    items
}

/// 从行中提取百分比
fn extract_percentage_from_line(line: &str) -> Option<f64> {
    if let Ok(re) = Regex::new(r"(\d+\.?\d*)\s*%") {
        if let Some(caps) = re.captures(line) {
            if let Some(m) = caps.get(1) {
                if let Ok(pct) = m.as_str().parse::<f64>() {
                    return Some(pct);
                }
            }
        }
    }
    None
}

/// 解析账单文本（单个账单）
fn parse_bill_text(text: &str, raw_text: &str) -> Result<OcrResult> {
    debug!("[parse_bill_text] 开始解析文本: {} 字符", text.len());

    let mut result = OcrResult {
        r#type: "expense".to_string(),
        amount: 0.0,
        description: String::new(),
        bill_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        raw_text: raw_text.to_string(),
    };

    // 提取所有金额，取最大值
    let amounts = extract_all_amounts(text);
    if let Some(&max_amount) = amounts.first() {
        result.amount = max_amount;
        debug!("[parse_bill_text] 提取到金额: {}", max_amount);
    }

    // 提取日期
    debug!("[parse_bill_text] 尝试提取日期");
    let date_patterns = vec![
        r"(\d{4})[年\-/](\d{1,2})[月\-/](\d{1,2})",
        r"(\d{4})-(\d{2})-(\d{2})",
        r"(\d{4})/(\d{1,2})/(\d{1,2})",
    ];

    for pattern in &date_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if caps.len() >= 4 {
                    if let (Some(year), Some(month), Some(day)) =
                        (caps.get(1), caps.get(2), caps.get(3))
                    {
                        let year_str = year.as_str();
                        let month_val = month.as_str().parse::<u32>().unwrap_or(1);
                        let day_val = day.as_str().parse::<u32>().unwrap_or(1);

                        if month_val >= 1 && month_val <= 12 && day_val >= 1 && day_val <= 31 {
                            result.bill_date =
                                format!("{}-{:02}-{:02}", year_str, month_val, day_val);
                            debug!("[parse_bill_text] 提取到日期: {}", result.bill_date);
                            break;
                        }
                    }
                }
            }
        }
    }

    // 判断收支类型
    debug!("[parse_bill_text] 判断收支类型");
    let income_keywords = vec![
        "收入", "收款", "转入", "工资", "奖金", "红包", "退款", "到账",
    ];
    let expense_keywords = vec![
        "支出", "付款", "转出", "消费", "支付", "扣款", "购买", "购物", "餐饮", "交通",
    ];

    for keyword in &income_keywords {
        if text.contains(keyword) {
            result.r#type = "income".to_string();
            debug!("[parse_bill_text] 识别为收入 (关键词: {})", keyword);
            break;
        }
    }

    if result.r#type != "income" {
        for keyword in &expense_keywords {
            if text.contains(keyword) {
                result.r#type = "expense".to_string();
                debug!("[parse_bill_text] 识别为支出 (关键词: {})", keyword);
                break;
            }
        }
    }

    // 提取描述（优先使用识别到的分类）
    debug!("[parse_bill_text] 提取描述");

    // 尝试从文本中找到分类作为描述
    if let Some(category) = fuzzy_match_category(text) {
        result.description = format!("{}", category);
        debug!("[parse_bill_text] 使用分类作为描述: {}", result.description);
    }

    // 如果没有找到分类，尝试提取商户信息
    if result.description.is_empty() {
        let merchant_patterns = vec![
            r"商户[：:]\s*(.+)",
            r"店铺[：:]\s*(.+)",
            r"收款方[：:]\s*(.+)",
            r"付款给[：:]\s*(.+)",
        ];

        for pattern in &merchant_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Some(m) = caps.get(1) {
                        let desc = m.as_str().trim();
                        if !desc.is_empty() {
                            result.description = desc.to_string();
                            debug!("[parse_bill_text] 提取到商户描述: {}", result.description);
                            break;
                        }
                    }
                }
            }
        }
    }

    // 如果还是没有，使用默认描述
    if result.description.is_empty() {
        result.description = "账单".to_string();
    }

    info!(
        "[parse_bill_text] 解析完成: type={}, amount={}, date={}, desc={}",
        result.r#type, result.amount, result.bill_date, result.description
    );

    Ok(result)
}

/// 增强预处理图片以提高 OCR 效果
fn preprocess_for_ocr(img: DynamicImage, original_path: &str) -> Result<String> {
    debug!("[preprocess_for_ocr] 开始预处理图片");

    let (width, height) = img.dimensions();

    // 1. 放大小图片
    let img = if width < 1500 || height < 1500 {
        let scale = 2.0;
        let new_width = (width as f32 * scale) as u32;
        let new_height = (height as f32 * scale) as u32;
        debug!(
            "[preprocess_for_ocr] 放大图片: {}x{} -> {}x{}",
            width, height, new_width, new_height
        );
        img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // 2. 转换为灰度图
    let gray = img.to_luma8();

    // 3. 增强对比度
    let enhanced = enhance_contrast(&gray);

    // 4. 转换回 RGB 并保存
    let (w, h) = enhanced.dimensions();
    let mut rgb_img = RgbImage::new(w, h);
    for (x, y, pixel) in enhanced.enumerate_pixels() {
        let v = pixel[0];
        rgb_img.put_pixel(x, y, Rgb([v, v, v]));
    }

    // 保存预处理后的图片
    let processed_path = format!("{}.processed.png", original_path);
    rgb_img
        .save(&processed_path)
        .context("保存预处理图片失败")?;

    debug!("[preprocess_for_ocr] 预处理完成: {}", processed_path);
    Ok(processed_path)
}

/// 增强图片对比度
fn enhance_contrast(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);

    // 计算直方图
    let mut histogram = [0u32; 256];
    for pixel in img.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    // 找到有效范围（忽略极端值）
    let total_pixels = (width * height) as u32;
    let low_threshold = total_pixels / 100; // 1%
    let high_threshold = total_pixels - low_threshold;

    let mut cumulative = 0u32;
    let mut min_val = 0u8;
    let mut max_val = 255u8;

    for (i, &count) in histogram.iter().enumerate() {
        cumulative += count;
        if cumulative < low_threshold {
            min_val = i as u8;
        }
        if cumulative < high_threshold {
            max_val = i as u8;
        }
    }

    // 避免除以零
    let range = (max_val - min_val).max(1) as f32;

    // 应用对比度拉伸
    for (x, y, pixel) in img.enumerate_pixels() {
        let v = pixel[0];
        let normalized = if v <= min_val {
            0u8
        } else if v >= max_val {
            255u8
        } else {
            ((v - min_val) as f32 / range * 255.0) as u8
        };
        output.put_pixel(x, y, Luma([normalized]));
    }

    output
}

/// 预处理图片（二值化）- 备用方法
#[allow(dead_code)]
fn preprocess_image(img: DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    debug!("[preprocess_image] 开始图片预处理");

    // 转换为灰度图
    let gray = img.to_luma8();
    debug!("[preprocess_image] 转换为灰度图");

    // 应用自适应阈值进行二值化
    let mut binary = RgbImage::new(gray.width(), gray.height());

    // 计算全局阈值（Otsu's method 简化版）
    let mut histogram = [0u32; 256];
    for pixel in gray.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    let total_pixels = (gray.width() * gray.height()) as f64;
    let mut sum = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        sum += i as f64 * count as f64;
    }

    let mut sum_b = 0.0;
    let mut w_b = 0.0;
    let mut max_variance = 0.0;
    let mut threshold = 128u8;

    for (i, &count) in histogram.iter().enumerate() {
        w_b += count as f64;
        if w_b == 0.0 {
            continue;
        }

        let w_f = total_pixels - w_b;
        if w_f == 0.0 {
            break;
        }

        sum_b += i as f64 * count as f64;

        let m_b = sum_b / w_b;
        let m_f = (sum - sum_b) / w_f;

        let variance = w_b * w_f * (m_b - m_f).powi(2);

        if variance > max_variance {
            max_variance = variance;
            threshold = i as u8;
        }
    }

    debug!("[preprocess_image] 计算阈值: {}", threshold);

    for (x, y, pixel) in gray.enumerate_pixels() {
        let value = pixel[0];
        let new_value = if value > threshold { 255u8 } else { 0u8 };
        binary.put_pixel(x, y, Rgb([new_value, new_value, new_value]));
    }

    debug!("[preprocess_image] 二值化处理完成");
    binary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_ocr_text() {
        let input = "要饮 Y139.70";
        let output = correct_ocr_text(input);
        assert!(output.contains("餐饮美食") || output.contains("餐饮"));
        assert!(output.contains("¥"));
    }

    #[test]
    fn test_fuzzy_match_category() {
        assert_eq!(fuzzy_match_category("要饮 2.80%"), Some("餐饮美食"));
        assert_eq!(fuzzy_match_category("转账 42.45%"), Some("转账支出"));
        assert_eq!(fuzzy_match_category("购物 32.88%"), Some("购物消费"));
    }

    #[test]
    fn test_extract_all_amounts() {
        let text = "转账 ¥2120.00 购物 ¥1642.38";
        let amounts = extract_all_amounts(text);
        assert!(amounts.contains(&2120.00));
        assert!(amounts.contains(&1642.38));
    }
}
