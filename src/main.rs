mod scroll;
mod screenshot;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use colored::*;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::SystemTime;

// ==================== 响应结构体 ====================

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: u64,
}

#[derive(Deserialize)]
struct GreetRequest {
    name: String,
}

#[derive(Serialize)]
struct GreetResponse {
    message: String,
    greeting: String,
}

#[derive(Serialize)]
struct ScreenshotResponse {
    status: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct CrownInput {
    delta: i64,    // 表冠旋转增量
    speed: f64,    // 旋转速度 (0.0 ~ 1.0)
}

#[derive(Serialize)]
struct CrownResponse {
    position: i64,
    step: i64,
    velocity: String,
}

// ==================== 全局滚动位置 ====================

static POSITION: AtomicI64 = AtomicI64::new(0);

// ==================== 加速度算法 ====================

/// 根据旋转速度计算步长：速度越快，步长呈指数增长
/// 模拟钢铁侠全息投影的惯性感
fn calculate_step(delta: i64, speed: f64) -> i64 {
    let base = delta.abs() as f64;
    // 指数加速度: speed^2 让高速旋转时步长急剧增大
    let accel = 1.0 + (speed * speed) * 15.0;
    let step = (base * accel).round() as i64;
    if step == 0 { 1 } else { step }
}

/// 根据 step 大小判定速度等级
fn velocity_level(step: i64) -> &'static str {
    match step {
        0..=5    => "CRAWL",
        6..=20   => "STEADY",
        21..=50  => "BOOST",
        51..=100 => "WARP",
        _        => "HYPERDRIVE",
    }

}

/// 打印科幻风终端日志
fn sci_log(delta: i64, _step: i64, position: i64, velocity: &str) {
    let header = "[SYSTEM]".cyan().bold();
    let input = format!("Input: DigitalCrown Δ{:+}", delta).yellow();
    let exec = "Execution: Success".green();
    let pos = format!("Position: {}", position).white();
    let vel = format!("Velocity: {}", velocity);
    let vel_colored = match velocity {
        "CRAWL"      => vel.white(),
        "STEADY"     => vel.blue(),
        "BOOST"      => vel.magenta(),
        "WARP"       => vel.red(),
        "HYPERDRIVE" => vel.red().bold().on_yellow(),
        _            => vel.white(),
    };

    println!("{} {} | {} | {} | {}", header, input, exec, pos, vel_colored);
}

// ==================== 接口处理函数 ====================

/// GET /health - 健康检查接口
async fn health_check() -> Json<HealthResponse> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp,
    })
}

/// GET /greet?name=xxx - 问候接口
async fn greet(
    axum::extract::Query(params): axum::extract::Query<GreetRequest>,
) -> Json<GreetResponse> {
    Json(GreetResponse {
        message: format!("Hello, {}!", params.name),
        greeting: "Welcome to Rust API!".to_string(),
    })
}

/// POST /screenshot - 触发全屏截图
async fn take_screenshot() -> Json<ScreenshotResponse> {
    screenshot::take_screenshot();
    println!(
        "{}",
        "[SYSTEM] Screenshot: TRIGGERED".magenta().bold()
    );
    Json(ScreenshotResponse {
        status: "ok".to_string(),
        message: "Screenshot triggered".to_string(),
    })
}

/// GET /ws/crown - WebSocket 表冠控制接口
async fn crown_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_crown)
}

/// 处理 WebSocket 连接
async fn handle_crown(mut socket: WebSocket) {
    println!(
        "{}",
        "[SYSTEM] DigitalCrown Controller: ONLINE".cyan().bold()
    );

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                let input: serde_json::Result<CrownInput> = serde_json::from_str(&text);

                match input {
                    Ok(crown) => {
                        // 加速度算法计算步长
                        let step = calculate_step(crown.delta, crown.speed);

                        // 更新全局位置
                        let new_pos = if crown.delta >= 0 {
                            POSITION.fetch_add(step, Ordering::SeqCst) + step
                        } else {
                            POSITION.fetch_sub(step, Ordering::SeqCst) - step
                        };

                        let velocity = velocity_level(step);

                        // 科幻日志
                        sci_log(crown.delta, step, new_pos, velocity);

                        // 触发系统滚动
                        scroll::perform_scroll(crown.delta, step, crown.speed);

                        // 返回响应
                        let resp = CrownResponse {
                            position: new_pos,
                            step,
                            velocity: velocity.to_string(),
                        };

                        if let Ok(json) = serde_json::to_string(&resp) {
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                    }
                    Err(_) => {
                        let _ = socket
                            .send(Message::Text(
                                r#"{"error":"Invalid format. Expected: {\"delta\":1,\"speed\":0.5}}"#.into(),
                            ))
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!(
                    "{}",
                    "[SYSTEM] DigitalCrown Controller: OFFLINE".red().bold()
                );
                break;
            }
            Err(_) => {
                println!(
                    "{}",
                    "[SYSTEM] DigitalCrown Controller: CONNECTION LOST".red().bold()
                );
                break;
            }
            _ => {}
        }
    }
}

// ==================== 入口 ====================

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/greet", get(greet))
        .route("/ws/crown", get(crown_ws))
        .route("/screenshot", get(take_screenshot));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    scroll::init();
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║   Iron Man Holographic UI System     ║".cyan());
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!("{}", "Server running on http://localhost:3000".green().bold());
    println!("  GET /health         - 健康检查");
    println!("  GET /greet?name=xxx - 问候接口");
    println!("  GET /ws/crown       - WebSocket 表冠控制");
    println!("  GET /screenshot     - 触发全屏截图");

    axum::serve(listener, app).await.unwrap();
}
