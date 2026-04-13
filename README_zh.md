# Vibemux

键盘优先、跨平台的终端多路复用器，带 GUI 界面。基于 Tauri、Rust 和 xterm.js 构建。

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)
![Version](https://img.shields.io/badge/version-0.1.0-orange)
![License](https://img.shields.io/badge/license-MIT-green)

---

Vibemux 不是普通的终端模拟器，也不是 tmux 的 GUI 包装。它把每个终端视为一个有名称、颜色、工作目录和生命周期状态的 **session 任务**，让你随时清楚哪些任务在运行、哪些在等待、哪些需要你处理。

## 核心概念

### 热度模型（Thermal Model）

每个 session 都有一个热度状态：

| 状态 | 位置 | PTY 进程 | xterm 实例 |
|------|------|----------|------------|
| **Hot（热）** | Deck | 运行中 | 存活于 DOM |
| **Warm（温）** | Shelf | 运行中 | 已销毁，输出缓冲 |
| **Cold（冷）** | 归档 | 已停止 | 无 |

**Park（停靠）**：把热 session 移入 Shelf，进程继续运行，xterm 实例销毁，输出写入环形缓冲区。
**Recall（召回）**：把温 session 拉回 Deck，先 replay 缓冲区内容到新 xterm，再接入实时输出。

### Deck（工作台）

当前活跃（Hot）session 的水平排布区域。一个 session 处于 **focused（聚焦）** 状态（全尺寸、可交互），其余为 **peripheral（旁路）** 状态（压缩展示）。切换瞬间完成，不重建 PTY，不 replay，不丢失 scrollback。

### Shelf（后台栏）

后台任务区域。每个温 session 显示为一张卡片，包含名称、颜色、状态徽章、工作目录、最近输出和注意力状态。PTY 进程全程保持运行。

### 注意力状态（Attention State）

Vibemux 自动监测温 session 的输出并标记状态：

- `Active` — 有新输出
- `NeedsInput` — 输出包含 `y/n`、`press enter`、`do you want` 等提示
- `Failed` — 输出包含 `error`、`panic`、`fatal`
- `Done` — 进程以退出码 0 结束
- `Failed` — 进程以非零退出码结束

## 功能特性

- **水平 Deck**，聚焦 + 旁路 pane 布局，带 2.5D 彩色边框
- **Shelf 后台栏**，session 进程保持运行，卡片实时显示状态
- **Park / Recall**，无需杀进程即可在前后台之间移动 session
- **Replay**，召回时先回放缓冲输出，再接入实时流
- **键盘优先导航**，前缀键激活 Navigation Mode
- **GUI 创建 session**，支持名称、工作目录、命令类型、颜色设置
- **Session 重命名**，键盘触发内联重命名
- **Session 搜索**，按名称、工作目录、状态模糊搜索
- **拖拽排序** Deck 中的热 session
- **主题与字体设置**，完整 ANSI 16 色调色板、字体族、字号、行高
- **配置持久化**，TOML 格式，原子写入，损坏自动回退
- **跨平台**，支持 macOS、Linux、Windows

## 键盘快捷键

用 `Cmd+Space`（macOS）或 `Ctrl+Space`（Linux/Windows）激活 Navigation Mode。

| 按键 | 操作 |
|------|------|
| `h` / `←` | 聚焦上一个热 session |
| `l` / `→` | 聚焦下一个热 session |
| `j` / `↓` | 选择下一个 Shelf 卡片 |
| `k` / `↑` | 选择上一个 Shelf 卡片 |
| `Enter` | 召回选中的 Shelf session |
| `n` | 新建 session |
| `b` | 将当前 session 停靠到 Shelf |
| `r` | 重命名当前 session |
| `x` | 关闭当前 session |
| `X`（Shift+x）| 强制 Kill 当前 session |
| `/` | 搜索 session |
| `?` | 显示快捷键帮助 |
| `Esc` | 退出 Navigation Mode |

## 安装

从 [Releases](../../releases) 页面下载对应平台的安装包。

| 平台 | 文件 |
|------|------|
| macOS（Apple Silicon / Intel）| `.dmg` |
| Linux | `.AppImage` 或 `.deb` |
| Windows | `.msi` 或 `.exe` |

## 从源码构建

**前置依赖：**
- [Rust](https://rustup.rs/)（stable）
- [Node.js](https://nodejs.org/) 18+
- 对应平台的 [Tauri 依赖](https://tauri.app/start/prerequisites/)

```bash
git clone https://github.com/yoko19191/vibemux
cd vibemux/apps/desktop
npm install
npm run tauri build
```

构建产物位于 `apps/desktop/src-tauri/target/release/bundle/`。

开发模式：

```bash
npm run tauri dev
```

## 配置文件

配置文件路径：

- **macOS**：`~/Library/Application Support/vibemux/config.toml`
- **Linux / Windows**：`~/.config/vibemux/config.toml`

示例：

```toml
[terminal]
font_family = "JetBrains Mono"
font_size = 14
line_height = 1.2
scrollback_lines = 10000
replay_buffer_mb = 20

[theme]
background = "#111111"
foreground = "#d9d4c7"
cursor = "#ff6b57"

[layout]
focused_pane_width = 0.6
preview_opacity = 0.8
animation_ms = 150
max_hot_sessions = 6
shelf_position = "bottom"

[shell]
default = "/bin/zsh"
```

所有字段均为可选，缺失时使用默认值。启动时若检测到配置文件损坏，会自动回退到默认配置并在界面顶部显示警告横幅。

## 技术栈

- **桌面壳**：[Tauri](https://tauri.app/) v2
- **后端**：Rust + [Tokio](https://tokio.rs/) + [portable-pty](https://github.com/wez/wezterm/tree/main/pty)
- **前端**：[Svelte](https://svelte.dev/) 5 + TypeScript
- **终端渲染**：[xterm.js](https://xtermjs.org/) v6

## License

MIT
