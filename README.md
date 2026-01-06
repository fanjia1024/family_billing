# 家庭账单收支管理 APP

使用 Tauri 2.0 + Rust + Vue 3 构建的跨平台家庭账单收支管理应用。

## 功能特性

- 📱 **跨平台支持**：支持桌面端（Windows/macOS/Linux）和移动端（iOS/Android）
- 👨‍👩‍👧‍👦 **家庭管理**：管理家庭成员和角色
- 💰 **账单管理**：手动添加、编辑、删除账单记录
- 📸 **OCR 识别**：支持微信/支付宝账单截图自动识别（开发中）
- 📊 **数据统计**：收支统计、月度趋势、分类占比等可视化图表
- 💾 **数据导入导出**：支持 JSON 格式的数据导入导出

## 技术栈

### 前端

- Vue 3 + TypeScript
- Vite
- Pinia (状态管理)
- Vue Router
- Vant 4 (UI 组件库)
- ECharts (数据可视化)

### 后端

- Rust
- Tauri 2.0
- SQLite (本地数据库)
- rusqlite (数据库驱动)

## 开发环境要求

- Node.js 18+
- Rust 1.70+
- Tauri CLI 2.0+
- Tesseract OCR (用于账单截图识别)

### 安装 Tesseract OCR

OCR 功能需要安装 Tesseract 及中文语言包：

**macOS:**

```bash
brew install tesseract
brew install tesseract-lang  # 安装所有语言包（包含中文）
```

**Ubuntu/Debian:**

```bash
sudo apt-get install tesseract-ocr
sudo apt-get install tesseract-ocr-chi-sim  # 中文简体
```

**Windows:**

1. 从 [UB-Mannheim/tesseract](https://github.com/UB-Mannheim/tesseract/wiki) 下载安装程序
2. 安装时勾选 "Chinese Simplified" 语言包
3. 将 Tesseract 安装目录添加到系统 PATH 环境变量

验证安装：

```bash
tesseract --version
tesseract --list-langs  # 应包含 chi_sim
```

## 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在首次构建时自动安装
```

## 开发运行

```bash
# 桌面端开发
npm run tauri:dev

# 移动端开发（需要先初始化）
npm run tauri android init  # Android
npm run tauri ios init     # iOS

npm run tauri android dev  # Android 开发
npm run tauri ios dev      # iOS 开发
```

## 构建

```bash
# 构建桌面端应用
npm run tauri:build

# 构建移动端应用
npm run tauri android build
npm run tauri ios build
```

## 项目结构

```
Household_Billing_Expense_Management_System/
├── src/                    # Vue 3 前端
│   ├── views/             # 页面视图
│   ├── components/         # 组件
│   ├── stores/            # Pinia 状态管理
│   ├── api/               # Tauri 命令封装
│   └── types/             # TypeScript 类型定义
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── commands/      # Tauri 命令
│   │   ├── models/        # 数据模型
│   │   ├── services/      # 业务逻辑
│   │   └── utils/         # 工具函数
│   └── Cargo.toml
└── package.json
```

## 数据库结构

- `family`: 家庭信息
- `member`: 家庭成员
- `category`: 账单分类
- `bill`: 账单记录
- `bill_image`: 账单截图

## 注意事项

1. OCR 功能使用本地 Tesseract 引擎，需要先安装 Tesseract 和中文语言包
2. 数据存储在应用数据目录的 SQLite 数据库中
3. 移动端构建需要配置相应的开发环境（Android SDK、Xcode 等）
4. 账单截图识别支持微信/支付宝常见格式，识别准确率取决于图片质量

## 许可证

MIT
