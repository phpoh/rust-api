mod scroll;
mod screenshot;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use colored::*;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};

// ==================== 响应结构体 ====================

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

/// 根据 step 大小判定速度等级（中文）
fn velocity_level(step: i64) -> &'static str {
    match step {
        0..=5    => "志辉轻滑",
        6..=20   => "志辉稳滑",
        21..=50  => "志辉加速",
        51..=100 => "志辉飞驰",
        _        => "志辉极速",
    }
}

/// 打印科幻风终端日志
fn sci_log(delta: i64, _step: i64, position: i64, velocity: &str) {
    let direction = if delta >= 0 { "⬆️ 志辉上滑" } else { "⬇️ 志辉下滑" };
    let header = "[志辉系统]".cyan().bold();
    let input = format!("{} Δ{:+}", direction, delta).yellow();
    let exec = "执行成功".green();
    let pos = format!("位置: {}", position).white();
    let vel = format!("速度: {}", velocity);
    let vel_colored = match velocity {
        "志辉轻滑"  => vel.white(),
        "志辉稳滑"  => vel.blue(),
        "志辉加速"  => vel.magenta(),
        "志辉飞驰"  => vel.red(),
        "志辉极速"  => vel.red().bold().on_yellow(),
        _           => vel.white(),
    };

    println!("{} {} | {} | {} | {}", header, input, exec, pos, vel_colored);
}

// ==================== 接口处理函数 ====================

/// POST /crown - HTTP 表冠控制接口（替代 WebSocket）
async fn crown_http(Json(crown): Json<CrownInput>) -> Json<CrownResponse> {
    let step = calculate_step(crown.delta, crown.speed);

    let new_pos = if crown.delta >= 0 {
        POSITION.fetch_add(step, Ordering::SeqCst) + step
    } else {
        POSITION.fetch_sub(step, Ordering::SeqCst) - step
    };

    let velocity = velocity_level(step);
    sci_log(crown.delta, step, new_pos, velocity);
    scroll::perform_scroll(crown.delta, step, crown.speed);

    Json(CrownResponse {
        position: new_pos,
        step,
        velocity: velocity.to_string(),
    })
}

/// GET /screenshot - 触发全屏截图
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
        .route("/ws/crown", get(crown_ws))
        .route("/crown", post(crown_http))
        .route("/screenshot", get(take_screenshot));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    scroll::init();
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║        志辉全息操控系统 v1.0          ║".cyan());
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!("{}", "Server running on http://localhost:3000".green().bold());
    println!("  GET /ws/crown       - WebSocket 表冠控制");
    println!("  POST /crown         - HTTP 表冠控制");
    println!("  GET /screenshot     - 触发全屏截图");

    axum::serve(listener, app).await.unwrap();
}
