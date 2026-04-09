# Rust API 项目指南

## 一、项目结构说明

```
rust-api/
├── Cargo.toml        # 项目配置文件（相当于 Java 的 pom.xml / build.gradle）
├── Cargo.lock        # 依赖锁定文件（相当于 Java 的 lock 文件，自动生成，不要手动改）
├── .gitignore        # Git 忽略规则
├── src/
│   └── main.rs       # 程序入口（相当于 Java 的 Application.java）
└── target/           # 编译输出目录（相当于 Java 的 target/ 或 build/）
    └── debug/        # debug 模式编译产物
        └── rust-api.exe   # 编译后的可执行文件
```

### 各文件详解

| 文件 | 作用 | 类比 Java |
|------|------|-----------|
| `Cargo.toml` | 声明项目名称、版本、依赖 | `pom.xml` / `build.gradle` |
| `Cargo.lock` | 锁定依赖的精确版本号 | Maven 的 `.flattened-pom.xml` |
| `src/main.rs` | 程序主入口 | `Application.java` |
| `target/` | 编译输出目录 | `target/`（Maven）或 `build/`（Gradle） |

### Cargo.toml 关键字段

```toml
[package]
name = "rust-api"          # 项目名
version = "0.1.0"           # 版本号
edition = "2021"            # Rust 语言版本（相当于 Java 8/11/17）

[dependencies]              # 第三方依赖
axum = "0.8"                # Web 框架（相当于 Spring Boot）
tokio = { version = "1", features = ["full"] }  # 异步运行时
serde = { version = "1", features = ["derive"] } # JSON 序列化/反序列化
serde_json = "1"            # JSON 处理库
```

---

## 二、本地开发环境准备

### 2.1 安装 Rust

```bash
# Windows：下载并运行 https://rustup.rs/
# 或者使用 winget
winget install Rustlang.Rustup
```

### 2.2 安装 C++ 编译工具（Windows 必须）

Rust 在 Windows 上需要 MSVC 链接器：

```bash
# 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools --override '--add Microsoft.VisualStudio.Workload.VCTools'
```

### 2.3 验证环境

```bash
rustc --version    # 应显示 rustc 1.xx.x
cargo --version    # 应显示 cargo 1.xx.x
```

---

## 三、启动项目

### 3.1 开发模式运行（Debug）

```bash
cd rust-api
cargo run
```

输出：
```
Server running on http://localhost:3000
  GET /health         - 健康检查
  GET /greet?name=xxx - 问候接口
```

### 3.2 测试接口

```bash
# 健康检查
curl http://localhost:3000/health
# 返回：{"status":"ok","timestamp":1775698354}

# 问候接口
curl "http://localhost:3000/greet?name=World"
# 返回：{"message":"Hello, World!","greeting":"Welcome to Rust API!"}
```

### 3.3 仅编译不运行

```bash
cargo build            # debug 模式
cargo build --release  # release 模式（生产用）
```

---

## 四、打包发布

### 4.1 Release 编译

```bash
cargo build --release
```

产物位置：`target/release/rust-api.exe`

### 4.2 编译产物说明

| 模式 | 路径 | 特点 |
|------|------|------|
| Debug | `target/debug/rust-api.exe` | 包含调试信息，体积大，未优化 |
| Release | `target/release/rust-api.exe` | 经过优化，体积小，性能好 |

### 4.3 打包格式

Rust 编译后是**单个可执行文件**（.exe），不需要像 Java 那样打包成 jar/war。

和 Java 的关键区别：

| 对比项 | Java | Rust |
|--------|------|------|
| 打包格式 | .jar / .war | 单个 .exe（Windows）或单个二进制文件（Linux） |
| 运行依赖 | 需要 JRE/JDK | 无需任何运行时，独立运行 |
| 文件大小 | 通常几十 MB（含依赖） | 通常几 MB 到十几 MB |
| 启动速度 | 秒级 | 毫秒级 |

### 4.4 交叉编译（在 Windows 上编译 Linux 版本）

```bash
# 添加 Linux 目标平台
rustup target add x86_64-unknown-linux-gnu

# 编译 Linux 版本
cargo build --release --target x86_64-unknown-linux-gnu
```

产物位置：`target/x86_64-unknown-linux-gnu/release/rust-api`

> 注意：Windows 上交叉编译到 Linux 需要配置 Linux 链接器，比较复杂。
> 更简单的做法是在目标 Linux 服务器上直接编译，或使用 Docker 编译。

---

## 五、服务器部署

### 5.1 Linux 服务器部署（推荐方式）

#### 步骤 1：上传二进制文件

```bash
# 编译好的二进制文件上传到服务器
scp target/release/rust-api user@server:/opt/rust-api/
```

#### 步骤 2：直接运行

```bash
# 添加执行权限
chmod +x /opt/rust-api/rust-api

# 前台运行（测试用）
./rust-api

# 后台运行
nohup ./rust-api > rust-api.log 2>&1 &
```

#### 步骤 3：配置为系统服务（生产推荐）

创建 `/etc/systemd/system/rust-api.service`：

```ini
[Unit]
Description=Rust API Service
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/rust-api
ExecStart=/opt/rust-api/rust-api
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
systemctl daemon-reload
systemctl enable rust-api     # 开机自启
systemctl start rust-api      # 启动
systemctl status rust-api     # 查看状态
systemctl stop rust-api       # 停止
systemctl restart rust-api    # 重启
```

### 5.2 Docker 部署

创建 `Dockerfile`：

```dockerfile
# 构建阶段
FROM rust:1.94 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 运行阶段（极小的镜像）
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/rust-api /usr/local/bin/rust-api
EXPOSE 3000
CMD ["rust-api"]
```

```bash
# 构建镜像
docker build -t rust-api .

# 运行容器
docker run -d -p 3000:3000 --name rust-api rust-api
```

### 5.3 Nginx 反向代理

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 六、常用命令速查

| 命令 | 作用 | 类比 Java |
|------|------|-----------|
| `cargo new project-name` | 创建新项目 | Spring Initializr |
| `cargo run` | 编译并运行 | `mvn spring-boot:run` |
| `cargo build` | 编译（debug） | `mvn compile` |
| `cargo build --release` | 编译（release） | `mvn package -Pprod` |
| `cargo check` | 快速检查语法错误（不生成二进制） | IDE 实时检查 |
| `cargo test` | 运行测试 | `mvn test` |
| `cargo clean` | 清理编译产物 | `mvn clean` |
| `cargo update` | 更新依赖版本 | `mvn versions:display-dependency-updates` |
| `cargo add package-name` | 添加依赖 | 往 pom.xml 加依赖 |
| `cargo doc --open` | 生成本地文档 | `mvn javadoc:javadoc` |

---

## 七、项目扩展建议

随着项目增大，建议调整目录结构：

```
src/
├── main.rs            # 程序入口，只做启动
├── routes/            # 路由/接口层（相当于 Controller）
│   ├── mod.rs
│   ├── health.rs
│   └── greet.rs
├── models/            # 数据结构（相当于 DTO/VO）
│   ├── mod.rs
│   └── response.rs
├── handlers/          # 业务逻辑（相当于 Service）
│   ├── mod.rs
│   └── greet_handler.rs
└── config/            # 配置
    └── mod.rs
```

---

## 八、常见问题

### Q: 修改代码后需要重新编译吗？
是的。Rust 是编译型语言，每次修改代码后需要 `cargo build` 或 `cargo run`。

### Q: 为什么编译这么慢？
首次编译需要下载和编译所有依赖，后续增量编译会快很多。可以用 `cargo check` 代替 `cargo build` 来快速检查语法。

### Q: 怎么改端口号？
修改 `src/main.rs` 中的 `bind("0.0.0.0:3000")`，把 `3000` 改成你想要的端口。

### Q: 怎么添加新接口？
在 `src/main.rs` 中：
1. 定义请求/响应结构体
2. 写一个 `async fn` 处理函数
3. 在 `Router::new().route()` 中注册路由
