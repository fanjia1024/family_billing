# 家庭账单收支管理 APP

使用 Tauri 2.0 + Rust + Vue 3 构建的跨平台家庭账单收支管理应用。

## 功能特性

- 📱 **跨平台支持**：支持桌面端（Windows/macOS/Linux）
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

## 使用说明

- **启动应用**：开发时执行 `npm run tauri:dev` 启动桌面应用；使用正式版时直接打开安装后的应用即可。
- **家庭与成员**：在「家庭」页创建或编辑家庭信息，添加家庭成员并设置角色。
- **账单管理**：在「账单」页添加、编辑或删除账单记录，选择分类、关联成员、填写金额与日期等。
- **OCR 识别**：上传微信/支付宝账单截图，使用本地 Tesseract 进行识别并自动填入账单信息（需先安装 Tesseract，详见下方开发环境要求）。后续将支持在线 OCR，详见 Roadmap。
- **统计与报表**：在「统计」页查看收支汇总、月度趋势、分类占比等可视化图表。
- **数据备份**：在「设置」页进行 JSON 格式的数据导入或导出，便于备份与迁移。

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
│   │   ├── commands/      # Tauri 命令入口，DTO 与调用 Application
│   │   ├── application/   # 应用服务（用例编排）
│   │   ├── domain/        # 实体、值对象、领域服务、端口
│   │   ├── infrastructure/# SQLite 持久化、OCR 实现等
│   │   ├── services/      # 遗留（database 等再导出，后续可收口）
│   │   └── utils/         # 通用工具
│   └── Cargo.toml
└── package.json
```

## Architecture（后端分层）

后端采用分层架构，依赖方向为：Commands → Application → Domain；Infrastructure 实现 Domain 的端口。

- **Commands**：只做 DTO 转换与调用 Application，不包含业务规则；将 DomainError 映射为前端可读文案（如 `to_user_message`）。
- **Application**：只做编排（调 Domain 服务 + 端口），不做校验与策略选择；校验器、删除策略等在 bootstrap（lib.rs）注入。
- **Domain**：规则的唯一真相源。实体与值对象（如 `CreateBill::try_new`、Money、BillDate）负责创建侧校验；领域服务（FamilyValidator、CategoryValidator、DeletionPolicy、StatisticsCalculator）负责业务规则；端口（Repository、UnitOfWork、OcrEngine、ExpenseAnalyzer）定义持久化与外部能力接口。
- **Infrastructure**：实现 Domain 的端口（Sqlite*Repository、SqliteUnitOfWork、TesseractOcrEngine 等）；Repository 仅做存取，无统计/汇总语义。

**事务**：多步写（如「账单 + 图片」）通过 UnitOfWork（`run_bill_with_image`）在同一事务内完成；OCR 在 UoW 外执行，失败不会落库。删除成员/分类为单步写（先查账单数 → 策略 → delete）。

**错误**：Domain 使用 `DomainError`；Command 层将错误映射为用户可见文案。

整体为 Local-first、AI-ready：ExpenseAnalyzer 端口已预留，便于后续接入规则或 LLM 实现。

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
5. 后续将支持在线 OCR、移动端与 LLM 分析等能力，详见下方 [Roadmap](#roadmap)。

## Roadmap

后续版本规划（具体时间待定）：

- **在线 OCR 对接**：在现有本地 Tesseract 基础上，支持对接在线 OCR 服务（如云厂商/第三方 API），提升识别率与多格式适配；为可选功能，使用需配置相应 API Key。
- **移动端支持**：完善 Tauri Android/iOS 的构建与适配，支持真机安装、小屏与触摸操作；规划应用商店发布。
- **LLM 大模型接入**：支持接入大模型 API，用于账单内容理解、分类建议、智能摘要与分析（如异常支出提醒等）；为可选能力，涉及数据外传时请关注隐私与合规说明。

## 许可证

MIT
