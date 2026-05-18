# Ajisai（绣球花）

兼容 Apache Hop 的超轻量、极速 ETL 引擎，使用 Rust 重新构建。

> 保留 Apache Hop 强大的数据转换能力，同时大幅简化操作界面。

[English](README.md) | [日本語](README_ja.md) | 中文

---

## 特性

- **兼容 Apache Hop** — 可直接读取 `.hpl` 管道文件
- **高速低内存** — Rust 原生二进制；tokio 异步执行，rayon CPU 并行处理
- **命令行 / 图形界面两用** — CLI 工具（阶段 1），可视化编辑器（阶段 3，计划中）
- **跨平台** — Windows / macOS / Linux
- **多语言** — 日语 / 英语（阶段 4，计划中）

---

## 安装

```bash
git clone <repo>
cd ajisai
cargo build --release
```

二进制文件生成于 `target/release/ajisai-cli`。

---

## 快速开始

### 执行管道

```bash
ajisai-cli run -p path/to/pipeline.hpl
```

### 验证管道（不执行）

```bash
ajisai-cli validate -p path/to/pipeline.hpl
```

### 列出可用的 Transform

```bash
ajisai-cli list-transforms
```

### 传递环境变量

```bash
ajisai-cli run -p pipeline.hpl -e INPUT_DIR=/data -e OUTPUT_DIR=/output
```

在管道文件中通过 `${INPUT_DIR}` 引用。

---

## 支持的 Transform

| Transform | 说明 |
|---|---|
| `CsvFileInput` | 读取 CSV 文件（自动识别表头、类型转换）|
| `CsvFileOutput` | 写入 CSV 文件 |
| `JsonFileInput` | 读取 JSON 文件（数组或 JSONL 格式）|
| `JsonFileOutput` | 写入 JSON 文件（数组或 JSONL 格式）|
| `FilterRows` | 按条件表达式过滤行 |
| `SelectValues` | 字段选择、重命名及类型转换 |
| `SortRows` | 多字段排序（rayon 并行排序）|
| `AddConstants` | 为每行追加常量字段 |
| `CalculatorStep` | 字段计算：四则运算、字符串操作、类型转换、IfNull 等 |
| `StreamLookup` | 内存哈希连接（维度查找）|
| `MergeJoin` | 排序归并连接（Inner / Left / Right / Full）|
| `Deduplicate` | 去除重复行（全字段或指定键）|
| `TableInput` | 通过 SQL 读取数据库表（SQLite / PostgreSQL / MySQL）|
| `TableOutput` | 向数据库表写入数据（Insert / Upsert / Overwrite）|

---

## 管道文件（.hpl）示例

```xml
<?xml version="1.0" encoding="UTF-8"?>
<pipeline>
  <name>My Pipeline</name>
  <transform>
    <name>CSV Input</name>
    <type>CSVFileInput</type>
    <filename>${INPUT_FILE}</filename>
    <separator>,</separator>
    <header>Y</header>
  </transform>
  <transform>
    <name>Filter Adults</name>
    <type>FilterRows</type>
  </transform>
  <transform>
    <name>CSV Output</name>
    <type>CSVFileOutput</type>
    <filename>${OUTPUT_FILE}</filename>
    <separator>,</separator>
    <header>Y</header>
  </transform>
  <order>
    <hop>
      <from>CSV Input</from>
      <to>Filter Adults</to>
      <enabled>Y</enabled>
    </hop>
    <hop>
      <from>Filter Adults</from>
      <to>CSV Output</to>
      <enabled>Y</enabled>
    </hop>
  </order>
</pipeline>
```

可直接加载使用 Apache Hop 创建的 `.hpl` 文件。

---

## 架构

```
ajisai/
├── crates/
│   ├── core/          Row / RowSchema / Value 类型定义，管道执行引擎
│   ├── transforms/    内置 Transform 实现集合
│   ├── hop-compat/    .hpl / .hwf XML 解析器，Apache Hop 兼容层
│   └── cli/           ajisai-cli 二进制
└── tests/fixtures/    示例管道与测试数据
```

### 执行模型

```
[CsvInput] ──mpsc──> [FilterRows] ──mpsc──> [SortRows] ──mpsc──> [CsvOutput]
  tokio 任务           tokio 任务             tokio 任务            tokio 任务
                                           (rayon 内部)
```

- 节点间通信：有界 `tokio::sync::mpsc` 通道（内置背压控制）
- I/O 密集型：tokio async/await
- CPU 密集型（排序等）：rayon 并行

---

## 开发路线图

| 阶段 | 内容 | 状态 |
|---|---|---|
| 阶段 1 | CLI + 基础 Transform + .hpl 兼容 | 完成 |
| 阶段 2 | JSON / Calculator / StreamLookup / MergeJoin / Deduplicate / DB / .hwf | 完成 |
| 阶段 3 | GUI — egui 可视化管道编辑器 | 计划中 |
| 阶段 4 | 完整多语言 UI，多平台打包 | 计划中 |

---

## 构建与测试

```bash
# 调试构建
cargo build

# 发布构建（优化）
cargo build --release

# 运行全部测试
cargo test --workspace

# 指定日志级别运行
ajisai-cli --log-level debug run -p pipeline.hpl
```

---

## Ajisai vs Apache Hop

| | Apache Hop | Ajisai |
|---|---|---|
| 运行环境 | JVM | Rust 原生二进制 |
| 内存占用 | 数百 MB 以上 | 数 MB 起 |
| 启动时间 | 数秒 | 即时 |
| 界面 | 功能丰富、较复杂 | 简洁优先 |
| .hpl / .hwf | 原生支持 | 导入支持 |

---

## 许可证声明

> "Apache Hop" 是 Apache Software Foundation 的商标。Ajisai 是与 ASF 无关的独立项目，未获 ASF 的认可、关联或保证。

MIT OR Apache-2.0
