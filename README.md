# 数据字段血缘与映射可视化工具（DataField Mapper）

一款面向数据架构师、数仓开发工程师与数据治理人员的**可视化字段级血缘与映射设计管理工具**。专为解决系统割接、ETL数仓建模、指标流向梳理中“字段口径不透明、文档零散易失效”的痛点而设计。

---

## 🌟 核心功能亮点

1. **表结构双模式获取**：
   - **DDL 脚本解析导入**：支持直接粘贴 SQL 文本或上传 `.sql` 文件。
     - **优先深度兼容达梦数据库（DM8/DM7）**：全面解析达梦特有类型（`VARCHAR2`, `NUMBER`, `DATETIME`, `CLOB` 等）及 `COMMENT ON TABLE` / `COMMENT ON COLUMN` 语法。
     - 兼顾 MySQL、PostgreSQL 主流方言。
   - **在线数据库逆向拉取**：输入主机 IP、端口、库名、账号密码，一键拉取数据字典并选择导入表。
2. **无限画布与字段 1 对 1 可视化拖拽连线**：
   - **表节点卡片（Table Node）**：展示英文名、中文注释、字段类型、字段列表；支持字段名与注释模糊搜索过滤。
   - **字段端点拖拽连线**：源字段锚点（Handle）拖至目标字段锚点建立血缘连接，支持平滑贝塞尔曲线吸附，防自环，点击连线可删除。
   - **上下游全路径血缘高亮**：一键高亮选中表的上下游关联路径，无关表节点自动虚化聚焦。
   - **智能自动排版**：内置 Dagre 层次图算法，多表复杂网络一键整齐排版，避免节点堆叠。
3. **多格式交付与持久化**：
   - **Excel 映射清单（.xlsx）**：一键导出美观规范的字段映射字典表（源表/字段/注释 -> 目标表/字段/注释及创建时间）与表结构元数据。
   - **高清架构图片导出**：支持当前画布 2x 高清导出为 PNG 图片。
   - **项目备份与快照迁移**：支持导出完整 `.json` 备份包并一键恢复。
   - **内置轻量持久化**：SQLite 零配置单文件数据库，节点坐标、血缘连线、多项目随时自动保存。

---

## 🛠️ 技术架构

- **前端**：Vue 3 + Vite + TypeScript + Element Plus + Pinia + Vue Flow (`@vue-flow/core`) + Dagre + Html2canvas
- **后端**：Python 3 + FastAPI + SQLAlchemy ORM + SQLite 3 + OpenPyXL + SqlGlot + PyMySQL + Psycopg2

---

## 🚀 快速启动

### 方式一：一键双击启动（推荐）

双击运行根目录下的 `run_dev.bat` 或在 PowerShell 中执行：

```powershell
.\run_dev.ps1
```

- **前端访问地址**：[http://localhost:5173](http://localhost:5173)
- **后端 Swagger 文档**：[http://127.0.0.1:8000/docs](http://127.0.0.1:8000/docs)

---

### 方式二：手动分步启动

#### 1. 启动后端

```powershell
cd backend
.\.venv\Scripts\python.exe -m uvicorn app.main:app --host 127.0.0.1 --port 8000 --reload
```

#### 2. 启动前端

```powershell
cd frontend
npm run dev
```

---

## 📁 目录结构

```
DataFieldManager/
├── backend/
│   ├── app/
│   │   ├── api/routes/          # FastAPI 路由 (项目/表/映射/DDL/数据库/导出)
│   │   ├── core/                # 数据库连接与配置
│   │   ├── models/              # SQLite 实体模型 (Project, TableMeta, ColumnMeta, FieldMapping)
│   │   ├── schemas/             # Pydantic 传输校验对象
│   │   ├── services/
│   │   │   ├── ddl_parser.py    # 达梦/MySQL/PG DDL 与注释解析引擎
│   │   │   ├── db_connector.py  # 在线数据库元数据逆向服务
│   │   │   └── exporter.py      # Excel 规范报表与 JSON 备份生成器
│   │   └── main.py              # 应用入口
│   ├── data_field_mapper.db     # 本地轻量持久化数据库
│   └── requirements.txt
├── frontend/
│   ├── src/
│   │   ├── api/                 # Axios 接口层
│   │   ├── components/
│   │   │   ├── canvas/          # Vue Flow TableNode 卡片与画布工具栏
│   │   │   └── dialogs/         # DDL 导入、在线连接、项目管理、备份迁移弹窗
│   │   ├── stores/              # Pinia 状态管理 (项目状态、画布连线排版状态)
│   │   ├── views/               # 主工作台画布视图
│   │   └── types/               # TypeScript 类型定义
│   └── package.json
├── run_dev.ps1                  # PowerShell 启动脚本
├── run_dev.bat                  # Windows 批处理启动脚本
└── README.md
```
