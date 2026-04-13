# 🚀 Rust API — 钢铁侠全息控制台

> 一个用 Rust + Axum 搭建的 Web API 项目，包含 HTTP 接口和 WebSocket 实时通信。
> 带有科幻风格的 **DigitalCrown（数字表冠）** 控制器，模拟钢铁侠全息投影的惯性操作体验！

---

## 🍎 Apple Watch 客户端 / Apple Watch Client

> **[CrownScroll-WatchOS](https://github.com/phpoh/crown-scroll-watchos)** — Apple Watch 数字表冠控制器，旋转表冠即可控制电脑滚动！
> Apple Watch Digital Crown controller — rotate the crown to scroll your computer!

---

## ✨ 项目特色

| 功能 | 说明 |
|------|------|
| 🏥 健康检查接口 | `GET /health` — 返回服务状态和时间戳 |
| 👋 问候接口 | `GET /greet?name=xxx` — 返回欢迎消息 |
| 🎮 WebSocket 表冠控制 | `GET /ws/crown` — 实时控制，加速度算法 |
| 🖥️ 系统级滚动 | 跨平台支持，控制前台窗口真正滚动 |
| 🌈 科幻终端日志 | 彩色输出，速度等级可视化 |
| 📦 单文件部署 | 编译后只有一个 exe，无需运行时 |

---

## 📂 项目结构

```
rust-api/
├── 📄 Cargo.toml     ← 📋 项目配置（相当于 Java 的 pom.xml）
├── 📄 Cargo.lock     ← 🔒 依赖版本锁定（自动生成，别手动改）
├── 📄 GUIDE.md       ← 📖 工程化完整指南（部署/打包/运维）
├── 📄 test.html      ← 🧪 WebSocket 测试页面（中文界面）
├── 📁 src/
│   ├── main.rs       ← 🏠 程序入口（路由和接口处理）
│   └── scroll.rs     ← 🖥️ 跨平台系统滚动模块
├── 📁 target/        ← 🏗️ 编译输出（git 已忽略）
└── 📁 .gitignore     ← 🚫 Git 忽略规则
```

---

## ⚡ 快速开始

### 1️⃣ 环境准备

你需要安装以下工具：

- 🦀 **Rust** — [https://rustup.rs](https://rustup.rs)
- 🔧 **MSVC Build Tools** — Windows 编译必需
  ```bash
  winget install Microsoft.VisualStudio.2022.BuildTools --override '--add Microsoft.VisualStudio.Workload.VCTools'
  ```

验证安装：
```bash
rustc --version    # ✅ 能看到版本号就 OK
cargo --version    # ✅ 能看到版本号就 OK
```

### 2️⃣ 运行项目

```bash
git clone https://github.com/phpoh/rust-api.git
cd rust-api
cargo run
```

看到以下输出就说明启动成功 🎉：
```
╔══════════════════════════════════════╗
║   Iron Man Holographic UI System     ║
╚══════════════════════════════════════╝
Server running on http://localhost:3000
  GET /health         - 健康检查
  GET /greet?name=xxx - 问候接口
  GET /ws/crown       - WebSocket 表冠控制
```

---

## 🧪 接口测试

### 🏥 健康检查

```bash
curl http://localhost:3000/health
```

返回：
```json
{"status": "ok", "timestamp": 1775698354}
```

### 👋 问候接口

```bash
curl "http://localhost:3000/greet?name=Tony"
```

返回：
```json
{"message": "Hello, Tony!", "greeting": "Welcome to Rust API!"}
```

### 🎮 WebSocket 表冠控制

客户端发送：
```json
{"delta": 5, "speed": 0.3}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `delta` | 整数 | 旋转增量（正=向上滑，负=向下滑） |
| `speed` | 浮点数 | 旋转速度，范围 0.0 ~ 1.0 |

服务端返回：
```json
{"position": 42, "step": 8, "velocity": "STEADY"}
```

| 字段 | 说明 |
|------|------|
| `position` | 当前滚动位置（全局累加） |
| `step` | 实际移动步长（经过加速度计算） |
| `velocity` | 速度等级名称 |

> 发送消息后，**前台窗口会真正滚动**。支持 Windows 和 macOS，编译时自动选择平台实现。

---

## 🧠 加速度算法

> 💡 核心公式：`step = |delta| × (1 + speed² × 15)`

速度越快 → 步长呈**指数级增长** → 模拟全息投影的惯性手感！

### 🏎️ 速度等级

| 步长范围 | 等级 | 终端颜色 | 感觉 |
|----------|------|----------|------|
| 0 ~ 5 | `CRAWL` 🐢 | ⚪ 白色 | 慢速精调 |
| 6 ~ 20 | `STEADY` 🚶 | 🔵 蓝色 | 稳定滚动 |
| 21 ~ 50 | `BOOST` 🏍️ | 🟣 紫色 | 加速推进 |
| 51 ~ 100 | `WARP` 🚀 | 🔴 红色 | 高速穿越 |
| 100+ | `HYPERDRIVE` ⚡ | 🟡 红底黄字 | 超光速！ |

### 🖥️ 终端日志效果

每次操作都会打印科幻风日志：
```
[SYSTEM] Input: DigitalCrown Δ+12 | Execution: Success | Position: 87 | Velocity: BOOST
```

---

## 🧪 WebSocket 测试页面

项目自带中文测试页面 `test.html`：

1. 🖱️ 双击打开 `test.html`
2. 🔗 点击 **连接** 连接服务
3. ⚙️ 设置增量和速度参数
4. 📤 点击 **发送** 手动发送一次
5. 🚀 点击 **连发** 模拟连续加速操作
6. ⏱️ 点击 **开始定时滚动**，然后切到其他窗口观察滚动效果

---

## 🏗️ 打包部署

### 📦 编译 Release 版本

```bash
cargo build --release
```

产物位置：`target/release/rust-api.exe`

> 🎯 Rust 编译后是**单个可执行文件**，不需要安装任何运行时环境！
> 这和 Java（需要 JRE）、Python（需要解释器）完全不同。

### 🐧 部署到 Linux 服务器

```bash
# 上传二进制文件
scp target/release/rust-api user@server:/opt/rust-api/

# 服务器上运行
chmod +x /opt/rust-api/rust-api
./rust-api
```

更多部署方式（systemd 服务、Docker、Nginx 反代）详见 👉 [GUIDE.md](GUIDE.md)

---

## 🔧 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| 🦀 Rust | 2021 Edition | 编程语言 |
| 🌐 Axum | 0.8 | Web 框架（含 WebSocket） |
| ⚡ Tokio | 1.x | 异步运行时 |
| 📝 Serde | 1.x | JSON 序列化 |
| 🎨 Colored | 2.x | 终端彩色输出 |
| 🪟 windows-sys | 0.61 | Windows 系统滚动（仅 Windows） |
| 🍎 core-graphics | 0.24 | macOS 系统滚动（仅 macOS） |

---

## 📖 Java 开发者看这里

如果你是从 Java 转过来的，这里有对照表：

| Java 概念 | Rust 对应 |
|-----------|-----------|
| `pom.xml` / `build.gradle` | `Cargo.toml` |
| `mvn spring-boot:run` | `cargo run` |
| `mvn package` | `cargo build --release` |
| `mvn clean` | `cargo clean` |
| `mvn test` | `cargo test` |
| `@Controller` | Axum handler 函数 |
| `@GetMapping` | `.route("/path", get(handler))` |
| `application.yml` | 代码里直接配置 |
| `.jar` 包 | 单个可执行文件 |

---

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=phpoh/rust-api&type=Date)](https://star-history.com/#phpoh/rust-api&Date)

## 📜 License

MIT License — 随便用！

---

## 🔗 Related Projects / 相关项目

- **[CrownScroll-WatchOS](https://github.com/phpoh/crown-scroll-watchos)** — Apple Watch 数字表冠客户端 / Apple Watch Digital Crown client app
