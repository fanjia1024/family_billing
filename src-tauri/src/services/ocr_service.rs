use anyhow::{Context, Result};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use regex::Regex;
use log::{info, debug, warn};

pub struct OcrResult {
    pub r#type: String,
    pub amount: f64,
    pub description: String,
    pub bill_date: String,
    pub raw_text: String,
}

pub fn recognize_image(image_path: &str) -> Result<OcrResult> {
    info!("[recognize_image] 开始识别图片: {}", image_path);
    
    // Read and preprocess image
    debug!("[recognize_image] 打开图片文件");
    let img = image::open(image_path)
        .context("Failed to open image")?;
    
    debug!("[recognize_image] 图片尺寸: {}x{}", img.width(), img.height());
    
    // Preprocess image for better OCR
    debug!("[recognize_image] 预处理图片");
    let _processed = preprocess_image(img);
    
    // TODO: Integrate actual OCR library (tesseract-rs or similar)
    // For now, we'll use a placeholder that returns basic structure
    // In production, you would:
    // 1. Use tesseract-rs or leptess to perform OCR
    // 2. Parse the text to extract bill information
    // 3. Use regex patterns to find amounts, dates, etc.
    
    warn!("[recognize_image] 注意: 当前使用占位符OCR，请手动输入账单信息");
    
    // Placeholder: Return structured result with current date
    // This allows the UI to work while OCR is being integrated
    let raw_text = "OCR识别文本 - 请手动输入账单信息".to_string();
    
    // Try to parse basic information from filename or use defaults
    let mut result = OcrResult {
        r#type: "expense".to_string(),
        amount: 0.0,
        description: "账单".to_string(),
        bill_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        raw_text: raw_text.clone(),
    };
    
    // Basic pattern matching for common bill formats
    // This is a simplified version - real OCR would extract from image text
    debug!("[recognize_image] 尝试解析账单文本");
    if let Ok(parsed) = parse_bill_text(&raw_text) {
        result = parsed;
    }
    
    info!("[recognize_image] 识别结果: type={}, amount={}, date={}", 
        result.r#type, result.amount, result.bill_date);
    
    Ok(result)
}

fn parse_bill_text(text: &str) -> Result<OcrResult> {
    debug!("[parse_bill_text] 开始解析文本: {} 字符", text.len());
    
    let mut result = OcrResult {
        r#type: "expense".to_string(),
        amount: 0.0,
        description: String::new(),
        bill_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        raw_text: text.to_string(),
    };
    
    // Try to extract amount (patterns: ¥100.00, 100元, etc.)
    debug!("[parse_bill_text] 尝试提取金额");
    let amount_patterns = vec![
        (r"¥\s*(\d+\.?\d*)", 1),
        (r"(\d+\.?\d*)\s*元", 1),
        (r"金额[：:]\s*(\d+\.?\d*)", 1),
        (r"(\d+\.?\d{2})\s*元", 1),
    ];
    
    for (pattern, group) in amount_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(text) {
                if let Some(amount_str) = captures.get(group) {
                    if let Ok(amount) = amount_str.as_str().parse::<f64>() {
                        result.amount = amount;
                        debug!("[parse_bill_text] 提取到金额: {}", amount);
                        break;
                    }
                }
            }
        }
    }
    
    // Try to extract date
    debug!("[parse_bill_text] 尝试提取日期");
    let date_patterns = vec![
        (r"(\d{4})[年\-/](\d{1,2})[月\-/](\d{1,2})", 1),
        (r"(\d{4})-(\d{2})-(\d{2})", 1),
        (r"(\d{4})/(\d{1,2})/(\d{1,2})", 1),
    ];
    
    for (pattern, _) in date_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(text) {
                if captures.len() >= 4 {
                    if let (Some(year), Some(month), Some(day)) = (
                        captures.get(1),
                        captures.get(2),
                        captures.get(3),
                    ) {
                        let month_str = format!("{:02}", month.as_str().parse::<u32>().unwrap_or(1));
                        let day_str = format!("{:02}", day.as_str().parse::<u32>().unwrap_or(1));
                        result.bill_date = format!("{}-{}-{}", year.as_str(), month_str, day_str);
                        debug!("[parse_bill_text] 提取到日期: {}", result.bill_date);
                        break;
                    }
                }
            }
        }
    }
    
    // Determine type (income/expense) based on keywords
    debug!("[parse_bill_text] 判断收支类型");
    let income_keywords = vec!["收入", "收款", "转入", "工资", "奖金"];
    let expense_keywords = vec!["支出", "付款", "转出", "消费", "支付"];
    
    let text_lower = text.to_lowercase();
    for keyword in income_keywords {
        if text_lower.contains(keyword) {
            result.r#type = "income".to_string();
            debug!("[parse_bill_text] 识别为收入 (关键词: {})", keyword);
            break;
        }
    }
    
    if result.r#type == "expense" {
        for keyword in expense_keywords {
            if text_lower.contains(keyword) {
                result.r#type = "expense".to_string();
                debug!("[parse_bill_text] 识别为支出 (关键词: {})", keyword);
                break;
            }
        }
    }
    
    // Extract description (first meaningful line or merchant name)
    debug!("[parse_bill_text] 提取描述");
    let lines: Vec<&str> = text.lines().collect();
    for line in lines {
        let line = line.trim();
        if !line.is_empty() && line.len() > 2 && !line.chars().all(|c| c.is_whitespace() || c.is_ascii_punctuation()) {
            result.description = line.to_string();
            debug!("[parse_bill_text] 提取到描述: {}", result.description);
            break;
        }
    }
    
    info!("[parse_bill_text] 解析完成: type={}, amount={}, date={}", 
        result.r#type, result.amount, result.bill_date);
    
    Ok(result)
}

fn preprocess_image(img: DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    debug!("[preprocess_image] 开始图片预处理");
    
    // Convert to grayscale
    let gray = img.to_luma8();
    debug!("[preprocess_image] 转换为灰度图");
    
    // Apply threshold to get binary image
    let mut binary = RgbImage::new(gray.width(), gray.height());
    
    for (x, y, pixel) in gray.enumerate_pixels() {
        let value = pixel[0];
        let threshold = 128u8;
        let new_value = if value > threshold { 255u8 } else { 0u8 };
        binary.put_pixel(x, y, Rgb([new_value, new_value, new_value]));
    }
    
    debug!("[preprocess_image] 二值化处理完成");
    binary
}
