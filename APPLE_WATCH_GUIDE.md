# Apple Watch Digital Crown 适配指南 / Apple Watch Digital Crown Integration Guide

> 本文档说明如何将 Apple Watch 的数字表冠（Digital Crown）与 Rust WebSocket 后端对接。
> This document explains how to connect the Apple Watch Digital Crown to the Rust WebSocket backend.

---

## 一、架构概览 / Architecture Overview

```
Apple Watch (WatchOS App)
    │
    │  Digital Crown 旋转事件
    │  Digital Crown rotation events
    │
    ▼
WKExtension / URLSessionWebSocketTask
    │
    │  WebSocket 连接: ws://服务器IP:3000/ws/crown
    │  WebSocket connection: ws://server-ip:3000/ws/crown
    │
    ▼
Rust 后端 (Axum WebSocket)
    │
    │  解析 JSON: {"delta": 5, "speed": 0.3}
    │  Parse JSON: {"delta": 5, "speed": 0.3}
    │
    ▼
系统级滚动 (PostMessageW / CGEvent)
System-level scroll (PostMessageW / CGEvent)
```

---

## 二、通信协议 / Communication Protocol

### WebSocket 地址 / WebSocket URL

```
ws://<服务器IP>:3000/ws/crown
```

### 请求格式 / Request Format

```json
{
  "delta": 5,
  "speed": 0.3
}
```

| 字段 Field | 类型 Type | 说明 Description |
|------------|-----------|------------------|
| `delta` | 整数 (Int) | 旋转增量，正数=向上滑，负数=向下滑 / Rotation delta, positive=scroll up, negative=scroll down |
| `speed` | 浮点数 (Double) | 旋转速度，范围 0.0 ~ 1.0 / Rotation speed, range 0.0 ~ 1.0 |

### 响应格式 / Response Format

```json
{
  "position": 42,
  "step": 8,
  "velocity": "STEADY"
}
```

| 字段 Field | 类型 Type | 说明 Description |
|------------|-----------|------------------|
| `position` | 整数 (Int) | 当前滚动位置 / Current scroll position |
| `step` | 整数 (Int) | 实际移动步长 / Actual movement step |
| `velocity` | 字符串 (String) | 速度等级 / Velocity level |

---

## 三、WatchOS 端实现 / WatchOS Implementation

### 3.1 项目配置 / Project Setup

WatchOS App 需要开启网络权限：

在 `Info.plist` 中添加：
```xml
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
```

### 3.2 监听数字表冠 / Listen to Digital Crown

WatchOS 通过 `crownSequencer` 监听表冠旋转：

```swift
import WatchKit

class CrownInterfaceController: WKInterfaceController {

    private var isRotating = false
    private var lastDelta: Double = 0

    override func awake(withContext context: Any?) {
        super.awake(withContext: context)

        crownSequencer?.delegate = self
        crownSequencer?.focus()
    }

    override func didAppear() {
        super.didAppear()
        crownSequencer?.focus()
    }
}

extension CrownInterfaceController: WKCrownDelegate {

    func crownDidRotate(_ crownSequencer: WKCrownSequencer?,
                        rotationalDelta: Double) {
        // rotationalDelta 是弧度值，通常范围 -0.5 ~ 0.5
        // rotationalDelta is in radians, typically range -0.5 ~ 0.5
        lastDelta = rotationalDelta
    }
}
```

### 3.3 WebSocket 连接 / WebSocket Connection

使用 `URLSessionWebSocketTask`：

```swift
import Foundation

class WebSocketManager: NSObject, URLSessionWebSocketDelegate {

    private var webSocketTask: URLSessionWebSocketTask?
    private var urlSession: URLSession?

    // 连接服务器 / Connect to server
    func connect(host: String, port: Int = 3000) {
        let urlString = "ws://\(host):\(port)/ws/crown"
        guard let url = URL(string: urlString) else { return }

        urlSession = URLSession(configuration: .default,
                                delegate: self,
                                delegateQueue: nil)
        webSocketTask = urlSession?.webSocketTask(with: url)
        webSocketTask?.resume()

        // 开始接收消息 / Start receiving messages
        receiveMessage()
    }

    // 发送表冠数据 / Send crown data
    func sendCrownInput(delta: Int, speed: Double) {
        let message: [String: Any] = [
            "delta": delta,
            "speed": speed
        ]

        guard let data = try? JSONSerialization.data(withJSONObject: message),
              let jsonString = String(data: data, encoding: .utf8) else { return }

        webSocketTask?.send(.string(jsonString)) { error in
            if let error = error {
                print("发送失败 / Send failed: \(error)")
            }
        }
    }

    // 接收服务端响应 / Receive server response
    private func receiveMessage() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    print("收到响应 / Received: \(text)")
                case .data(let data):
                    print("收到数据 / Received data: \(data.count) bytes")
                @unknown default:
                    break
                }
                self?.receiveMessage() // 持续接收 / Keep receiving
            case .failure(let error):
                print("接收失败 / Receive failed: \(error)")
            }
        }
    }

    // 断开连接 / Disconnect
    func disconnect() {
        webSocketTask?.cancel(with: .goingAway, reason: nil)
        webSocketTask = nil
    }
}
```

### 3.4 完整整合：表冠 → WebSocket / Full Integration: Crown → WebSocket

```swift
import WatchKit
import Foundation

class CrownScrollController: WKInterfaceController {

    private let wsManager = WebSocketManager()
    private var accumulatedDelta: Double = 0
    private var sendTimer: Timer?

    @IBOutlet weak var statusLabel: WKInterfaceLabel!
    @IBOutlet weak var hostField: WKInterfaceTextField!

    override func awake(withContext context: Any?) {
        super.awake(withContext: context)
        crownSequencer?.delegate = self
        crownSequencer?.focus()
    }

    // 连接按钮 / Connect button
    @IBAction func connectTapped() {
        let host = "192.168.1.100" // 替换为你的服务器IP / Replace with your server IP
        wsManager.connect(host: host)
        statusLabel.setText("已连接 / Connected")
        startSendLoop()
    }

    // 断开按钮 / Disconnect button
    @IBAction func disconnectTapped() {
        sendTimer?.invalidate()
        wsManager.disconnect()
        statusLabel.setText("已断开 / Disconnected")
    }

    // 定时发送累积的旋转量 / Periodically send accumulated rotation
    private func startSendLoop() {
        sendTimer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [weak self] _ in
            guard let self = self else { return }
            guard abs(self.accumulatedDelta) > 0.01 else { return }

            // 将弧度值转换为整数 delta（放大 10 倍增加灵敏度）
            // Convert radians to integer delta (scale by 10 for sensitivity)
            let delta = Int(self.accumulatedDelta * 10)

            // 计算 speed：旋转越快 speed 越大（0.0 ~ 1.0）
            // Calculate speed: faster rotation = higher speed (0.0 ~ 1.0)
            let speed = min(abs(self.accumulatedDelta) / 0.3, 1.0)

            self.wsManager.sendCrownInput(delta: delta, speed: speed)
            self.accumulatedDelta = 0 // 重置 / Reset
        }
    }
}

extension CrownScrollController: WKCrownDelegate {

    func crownDidRotate(_ crownSequencer: WKCrownSequencer?,
                        rotationalDelta: Double) {
        // 累积旋转量（正值=向上，负值=向下）
        // Accumulate rotation (positive=up, negative=down)
        accumulatedDelta += rotationalDelta
    }

    func crownDidBecomeIdle(_ crownSequencer: WKCrownSequencer?) {
        accumulatedDelta = 0
    }
}
```

---

## 四、关键参数调优 / Parameter Tuning

### 4.1 Delta 映射 / Delta Mapping

| 表冠弧度 Crown Radians | 换算公式 Formula | Delta 值 | 效果 Effect |
|------------------------|-----------------|----------|-------------|
| 0.05 | `0.05 × 10 = 0.5` → 0 | 0 | 微小旋转，忽略 Tiny rotation, ignored |
| 0.1 | `0.1 × 10 = 1` | 1 | 缓慢滚动 Slow scroll |
| 0.3 | `0.3 × 10 = 3` | 3 | 正常滚动 Normal scroll |
| 0.5 | `0.5 × 10 = 5` | 5 | 快速滚动 Fast scroll |

> 如果觉得不够灵敏，可以增大倍率（如改为 20）。如果太灵敏，减小倍率。
> Increase multiplier (e.g. 20) for more sensitivity, decrease for less.

### 4.2 Speed 映射 / Speed Mapping

```swift
// speed = min(|rotationalDelta| / 阈值, 1.0)
// 阈值越小越灵敏，推荐 0.2 ~ 0.5
// Smaller threshold = more sensitive, recommended 0.2 ~ 0.5
let speed = min(abs(rotationalDelta) / 0.3, 1.0)
```

### 4.3 发送频率 / Send Frequency

```swift
// 定时器间隔，推荐 0.05 ~ 0.15 秒
// Timer interval, recommended 0.05 ~ 0.15 seconds
Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true)
```

| 频率 Frequency | 间隔 Interval | 特点 Character |
|----------------|---------------|----------------|
| 高频 High | 50ms | 流畅，但网络负担大 Smooth, but more network load |
| 推荐 Recommended | 100ms | 平衡流畅度和性能 Balanced |
| 低频 Low | 150ms | 省网络，但稍有延迟 Less network, slight delay |

---

## 五、网络配置 / Network Configuration

### 5.1 同一局域网 / Same Local Network

Apple Watch 和电脑在同一 WiFi 下：

```
Apple Watch → WiFi → 局域网 → 电脑 (Rust 后端)
```

服务器监听 `0.0.0.0:3000`（已配置），Watch 连接电脑的局域网 IP：

```bash
# Windows 查看本机 IP / Check local IP on Windows
ipconfig

# macOS 查看本机 IP / Check local IP on macOS
ifconfig | grep inet
```

### 5.2 跨网络 / Across Networks

使用内网穿透工具（如 frp、ngrok）：

```bash
# 使用 ngrok 示例 / ngrok example
ngrok tcp 3000
```

Watch 连接 ngrok 提供的外网地址即可。

### 5.3 安全建议 / Security Recommendations

- 局域网使用可以直接用 `ws://`
- 跨网络建议在前面加 Nginx 反向代理，配置 `wss://`（TLS 加密）
- 生产环境建议加鉴权 token

---

## 六、WatchOS 项目结构建议 / WatchOS Project Structure

```
WatchApp/
├── WatchApp.swift                    # App 入口 / App entry
├── Assets.xcassets/                  # 资源文件 / Assets
├── Controllers/
│   └── CrownScrollController.swift   # 表冠控制界面 / Crown control interface
├── Services/
│   └── WebSocketManager.swift        # WebSocket 管理器 / WebSocket manager
└── Info.plist                        # 配置文件 / Configuration
```

---

## 七、测试步骤 / Testing Steps

1. 启动 Rust 后端 / Start Rust backend
   ```bash
   cargo run
   ```

2. 确认电脑 IP / Confirm computer IP
   ```bash
   ipconfig   # Windows
   ```

3. WatchOS App 中填入 IP 并连接 / Enter IP in WatchOS app and connect

4. 旋转数字表冠，观察电脑终端日志 / Rotate Digital Crown, check terminal logs

5. 前台窗口应该跟随滚动 / Foreground window should scroll accordingly

---

## 八、常见问题 / FAQ

### Q: Watch 连不上 WebSocket？
检查 Apple Watch 和电脑是否在同一 WiFi 网络。WatchOS 不支持模拟器测试 WebSocket，需要真机。

### Q: 旋转表冠没反应？
确保 `crownSequencer?.focus()` 已调用，且界面处于活跃状态。

### Q: 滚动方向反了？
将 delta 取反：`let delta = -Int(accumulatedDelta * 10)`。

### Q: 如何调试 WatchOS App？
Xcode 连接 Apple Watch 真机，查看控制台日志输出。
