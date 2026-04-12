# Vibemux 产品需求文档

Vibemux 是一个全新的、键盘优先的跨平台终端多路复用器，面向同时运行多个 shell、开发服务器、测试任务和编程 Agent 的开发者。它不是普通 Terminal，也不是 tmux 的简单 GUI 包装；它要用 GUI 帮用户构建和理解 session，用键盘完成高频操作，并通过水平 Deck、2.5D 彩色终端框和后台 Shelf 把“正在专注、旁路观察、后台运行、已归档”的终端任务生命周期清晰表达出来。

## 1. 文档信息

- 产品名称：Vibemux
- 文档类型：PRD + 技术架构说明
- 目标读者：产品设计、前端工程、Rust 后端工程、桌面应用工程、测试工程
- 技术栈约束：Tauri + Rust + xterm.js
- 文档状态：初版完整方案
- 核心灵感：水平排布的终端 Deck、2.5D 彩色边框、键盘优先的终端工作流

## 2. 背景与问题

tmux、screen、Zellij、WezTerm mux 等工具证明了终端多路复用的价值：用户可以让多个会话长时间运行，可以从一个界面管理多个命令，也可以在服务器或本地环境中保留任务状态。

但传统终端多路复用器存在明显体验问题：

- session、window、pane 的概念抽象，用户很难快速理解“哪个任务在做什么”。
- 状态主要藏在 status line、编号、短标题里，可视化弱。
- 新建 session、切换 session、整理任务、恢复上下文依赖记忆和命令。
- 后台任务虽然还活着，但缺少“是否完成、是否失败、是否等待输入”的清晰反馈。
- 多个 Agent 并行工作时，用户很难判断哪个 Agent 正在思考、哪个卡住了、哪个需要确认。
- 普通 Terminal app 关注窗口和标签页，缺少真正的 session 生命周期管理。

Vibemux 要解决的问题是：让终端多路复用从“记住隐藏编号和命令”变成“管理一组活着的任务”。

## 3. 产品定位

### 3.1 一句话定位

Vibemux 是一个 GUI 构建、键盘操作、Rust 驱动的终端任务多路复用器。

### 3.2 产品类别

Vibemux 同时属于以下类别：

- 终端模拟器前端
- 终端多路复用器
- 键盘优先工作区
- 本地任务管理器
- Agentic coding session manager

### 3.3 与普通 Terminal 的区别

普通 Terminal 的核心对象是窗口、标签页和 shell。

Vibemux 的核心对象是 session task。每个 session 都有：

- 名称
- 颜色身份
- 工作目录
- 启动命令
- 所属工作区
- 前台或后台状态
- 进程状态
- 最近输出
- 是否需要用户注意

### 3.4 与 tmux 的区别

tmux 的核心优势是强大的持久 session 和键盘操作。Vibemux 要保留这些优势，但用 GUI 和可视化状态降低认知成本。

对比：

| 维度 | tmux | Vibemux |
| --- | --- | --- |
| session 创建 | 命令优先 | GUI 构建 + 键盘启动 |
| session 识别 | 名字、编号、status line | 彩色框、任务卡、状态、最近输出 |
| 后台任务 | 隐藏在 session/window 列表 | Shelf 中持续可见 |
| 状态感知 | 用户自己判断 | 进程状态 + 注意力状态 |
| 新手可发现性 | 较弱 | 较强 |
| 高频操作 | 很强 | 必须同样强 |

### 3.5 非目标

Vibemux 第一阶段不做：

- IDE
- 代码编辑器
- 云端协作
- 远程同步服务
- 完整 Agent 聊天界面
- 从零实现终端渲染引擎
- tmux 协议兼容层
- 插件市场
- 复杂项目管理系统

## 4. 产品目标

### 4.1 核心目标

1. 提供流畅、键盘优先的多 session 终端体验。
2. 用水平 Deck 展示当前活跃 session，保留主任务专注和旁路感知。
3. 用后台 Shelf 管理仍在运行但暂时不需要完整渲染的 session。
4. 用 GUI 降低 session 创建、命名、模板、工作目录选择和颜色分配成本。
5. 用 Rust mux core 管理真实 session 生命周期，而不是把多个 terminal widget 简单塞进一个窗口。
6. 使用 xterm.js 复用成熟终端渲染能力，避免从零踩终端兼容性坑。
7. 支持 macOS、Linux、Windows。
8. 在多个 hot session 和多个 warm session 同时存在时保持性能可接受。

### 4.2 成功标准

MVP 成功时，用户应该能够：

1. 通过 GUI 创建一个 shell 或命令 session。
2. 同时打开多个 session，并在水平 Deck 中瞬间切换。
3. 把当前 session 一键 Park 到后台 Shelf，进程继续运行。
4. 从 Shelf 一键 Recall 一个 session，最近输出被 replay，随后接入实时输出。
5. 通过键盘完成 80% 以上高频操作。
6. 设置字体、字号、主题、pane 颜色。
7. 关闭应用后保留配置和可恢复的 session 元数据。
8. 在三大桌面平台上运行。

## 5. 目标用户

### 5.1 主要用户

主要用户是高频使用终端的开发者，尤其是同时运行多个本地任务或多个编程 Agent 的用户。

典型场景：

- 一个 session 跑 dev server。
- 一个 session 跑测试。
- 一个 session 跑 Claude Code。
- 一个 session 跑 Codex。
- 一个 session 看日志。
- 一个 session 用 shell 操作 git。

用户希望主任务清晰可读，同时不失去对其他任务的感知。

### 5.2 次要用户

- tmux 用户，但觉得 tmux 对 session 的可视化和管理太弱。
- 使用 WezTerm、iTerm2、Kitty、Alacritty 的高级用户。
- 需要长期运行实验、构建、脚本、服务的研究者。
- 喜欢 tiling window manager 或 keyboard launcher 的用户。

## 6. 核心产品概念

### 6.1 Session

Session 是 Vibemux 的核心实体。它不是一个普通标签页，而是一个长期存在的终端任务。

Session 包含：

- 唯一 ID
- 名称
- 工作目录
- 启动命令
- shell 类型
- 环境变量
- 颜色
- 所属 workspace
- thermal state
- process state
- attention state
- 最近输出
- 创建时间
- 最近活跃时间
- 退出状态

### 6.2 Deck

Deck 是当前活跃工作区的水平排布区域。

特征：

- 包含 hot sessions。
- 一个 session 是 focused session。
- focused session 大、清晰、可输入。
- 其他 session 是 peripheral sessions，压缩展示。
- 每个 session 有 2.5D 彩色终端框。
- 切换时有短而流畅的滑动动画。

Deck 的产品含义：这些 session 正在当前注意力范围内。

### 6.3 Shelf

Shelf 是后台任务区域。

特征：

- 包含 warm sessions。
- 不展示完整 terminal。
- 每个 session 显示为任务卡。
- 任务卡显示名称、颜色、状态、最近输出、工作目录、活跃时间。
- session 的 PTY 继续运行。
- recall 时重新创建终端渲染面并 replay 最近输出。

Shelf 的产品含义：这些 session 仍在运行，但用户暂时不需要完整观察。

### 6.4 Archive

Archive 是已完成或已关闭 session 的历史区域。

特征：

- 不再有活进程。
- 可以查看日志和元数据。
- 可以删除。
- 可以基于历史 session 重新启动一个新 session。

Archive 的产品含义：这些任务已经结束，但上下文仍可追溯。

### 6.5 Workspace

Workspace 是一组 session 的容器。

MVP 可以只有一个默认 workspace。后续版本支持多个 workspace，例如：

- 当前项目
- 后端
- 前端
- 实验
- 远程服务器
- Agent 队列

### 6.6 Session Thermal Model

Vibemux 使用 Hot / Warm / Cold 模型管理 session。

```text
Hot Session    在 Deck 中，完整 xterm 实例常驻，瞬间切换
Warm Session   在 Shelf 中，PTY 继续运行，终端实例销毁，Recall 时 replay
Cold Session   已归档或退出，不再运行，只保留日志和元数据
```

这条规则是产品和技术实现的核心：

```text
Deck sessions are live surfaces.
Shelf sessions are running tasks.
Archived sessions are retained history.
```

## 7. 核心交互

### 7.1 GUI 创建 Session

用户可以通过 New Session 面板创建 session。

字段：

- session 名称
- 工作目录
- 启动类型
- shell 或命令
- session 模板
- 颜色
- workspace
- 启动位置：前台、Deck、后台 Shelf

启动类型：

- Shell
- Custom Command
- Dev Server
- Test Command
- Agent
- SSH
- Docker Logs

MVP 至少支持 Shell 和 Custom Command。

### 7.2 键盘导航

Vibemux 默认有两种模式：

- Insert Mode：输入进入当前终端。
- Navigation Mode：按键控制 Vibemux。

默认键位建议：

```text
Ctrl+B             进入 Navigation Mode
Esc                退出 Navigation Mode
H / Left           聚焦上一个 Hot Session
L / Right          聚焦下一个 Hot Session
J / Down           选择下一个 Shelf Card
K / Up             选择上一个 Shelf Card
Enter              聚焦或 Recall 当前选中项
B                  Park 当前 session
F                  Recall 当前 shelf session 到前台
N                  新建 session
R                  重命名 session
X                  关闭 session
Shift+X            Kill session 进程
D                  切换 Deck Layout
M                  切换 Monocle Layout
G                  切换 Grid Layout
/                  搜索 session
?                  打开快捷键帮助
```

### 7.3 Park Session

Park 的含义是：把 hot session 放入后台 Shelf，进程继续运行。

行为：

1. 用户聚焦一个 Deck 中的 session。
2. 用户触发 Park。
3. session 从 Deck 中移除。
4. 前端销毁该 session 的 xterm 实例。
5. Rust mux core 继续持有 PTY。
6. Rust mux core 保存输出到 ring buffer。
7. Shelf 出现任务卡。
8. Deck 重新布局。

Park 不等于 kill，也不等于 suspend。

### 7.4 Recall Session

Recall 的含义是：把 warm session 从 Shelf 拉回 Deck。

行为：

1. 用户选中 Shelf card。
2. 用户触发 Recall。
3. session 状态从 Warm 变为 Hot。
4. 前端创建新的 xterm.js 实例。
5. mux core 发送最近输出 replay。
6. 前端分块写入 xterm。
7. replay 完成后接入实时输出。
8. session 进入 focused 或 peripheral 状态。

默认建议：Recall 后成为 focused session。

### 7.5 Hot Session 切换

Deck 内 session 切换必须瞬间恢复。

要求：

- 不销毁 xterm 实例。
- 不重新 spawn PTY。
- 不 replay。
- 不丢 selection 和 scrollback。
- 切换动画保持流畅。

### 7.6 Shelf 任务卡

每个 Shelf card 显示：

- 颜色
- 名称
- 状态 badge
- 工作目录或项目名
- 最近输出
- 最近活跃时间
- 是否需要输入
- 是否失败

示例：

```text
[orange] api-server
~/repo/backend
running · 14m · "Listening on :8000"
```

示例：

```text
[red] codex-fix-ci
needs input · "Apply this patch?"
```

### 7.7 Search

用户可以搜索：

- session 名称
- cwd
- workspace
- 最近输出摘要
- 状态
- 命令

搜索结果应支持键盘选择和 Recall。

## 8. 视觉设计要求

### 8.1 总体视觉

Vibemux 应该表现为一个冷静、高密度、键盘优先的开发者工具。

视觉重点：

- 主终端可读性最高。
- 旁路终端保持存在感，但不能抢注意力。
- Shelf card 紧凑、清晰、有状态。
- 颜色用于身份识别，不只是装饰。

### 8.2 2.5D 彩色终端框

Deck 中每个 terminal pane 都应该有彩色边框或框体感。

要求：

- 颜色稳定绑定 session。
- focused pane 更亮、更实。
- peripheral pane 可降低透明度。
- 颜色不应影响终端 ANSI 主题的可读性。
- 框体应帮助用户记忆任务位置。

### 8.3 Deck Layout

Deck 默认水平排布：

```text
┌───────────────┬──────────────────────────────┬───────────────┐
│ preview       │ focused terminal             │ preview       │
│ red frame     │ green frame                  │ blue frame    │
└───────────────┴──────────────────────────────┴───────────────┘
```

底部或侧边显示 Shelf：

```text
┌──────────────────────────────────────────────────────────────┐
│ [build ✓] [agent …] [server running] [tests failed]          │
└──────────────────────────────────────────────────────────────┘
```

### 8.4 响应式规则

不同窗口宽度下：

- 宽屏：focused pane + 多个 preview pane + shelf。
- 中等宽度：focused pane + 少量 preview pane + 可折叠 shelf。
- 小窗口：monocle 优先，shelf 通过 overlay 或抽屉显示。

### 8.5 字体和主题

MVP 必须支持：

- 字体族
- 字号
- 行高
- 终端前景色
- 终端背景色
- 光标颜色
- 选择颜色
- ANSI 基础 16 色
- pane 框颜色
- shelf card 颜色

## 9. 功能需求

### 9.1 Session 管理

MVP 必须实现：

- 创建 session。
- 创建 shell session。
- 创建 command session。
- 设置 cwd。
- 设置名称。
- 设置颜色。
- 聚焦 session。
- 重命名 session。
- 关闭 session。
- kill session。
- 检测 session 退出。
- Park session。
- Recall session。
- 搜索 session。

后续实现：

- Duplicate session。
- Restart session。
- Move session to workspace。
- Pin session。
- Archive session。
- Delete archived session。

### 9.2 Deck 管理

MVP 必须实现：

- 水平 Deck。
- focused pane。
- peripheral pane。
- hot session 顺序。
- 键盘切换。
- 鼠标点击聚焦。
- 拖拽重排。
- resize 后重新布局。

后续实现：

- Monocle layout。
- Grid layout。
- 自定义 layout。
- 保存 workspace layout。

### 9.3 Shelf 管理

MVP 必须实现：

- Park 后创建 Shelf card。
- Shelf card 状态更新。
- Shelf card 最近输出更新。
- 键盘选择 Shelf card。
- Recall Shelf card。
- Warm session 退出后更新 card。

后续实现：

- Shelf 分组。
- Shelf 排序。
- Shelf 过滤。
- Shelf card 预览。
- Shelf card 详情弹窗。

### 9.4 终端能力

MVP 必须实现：

- PTY 输入输出。
- xterm.js 渲染。
- 终端 resize。
- scrollback。
- copy。
- paste。
- 右键菜单。
- 链接打开。
- terminal title 更新。
- 基础 ANSI 颜色。

后续实现：

- 终端搜索。
- 超链接识别增强。
- bell 通知。
- alternate screen 状态优化。
- 只读日志查看器。

### 9.5 配置能力

MVP 必须实现：

- 配置持久化。
- GUI 设置面板。
- 配置文件加载。
- 配置校验。
- 配置错误回退。

配置项包括：

- 字体
- 字号
- 行高
- 主题
- 默认 shell
- 默认工作目录
- Deck 宽度
- preview 透明度
- 动画时间
- max hot sessions
- scrollback 大小
- replay buffer 大小

后续实现：

- keymap 配置。
- session templates。
- workspace templates。
- watch rules。
- agent detectors。

### 9.6 Session Templates

MVP 模板：

- Shell
- Command
- Dev Server
- Test Command

后续模板：

- Claude Code
- Codex
- Docker Logs
- SSH
- WSL
- Git Worktree
- Custom Agent

模板字段：

- name
- command
- args
- cwd
- env
- color
- workspace
- start mode
- watch rules

### 9.7 注意力状态

MVP attention states：

```text
normal
active
needs_input
failed
done
exited
```

状态来源：

- 进程退出码。
- 最近输出。
- 正则规则。
- 用户手动标记。

默认规则示例：

- 非 0 exit code -> failed。
- 0 exit code -> done。
- warm session 有新输出 -> active。
- 输出包含 `error`、`failed`、`panic` -> failed。
- 输出包含 `continue?`、`do you want`、`press enter` -> needs_input。

后续增强：

- Claude Code detector。
- Codex detector。
- npm/test/build detector。
- 自定义 watch rules。

## 10. 技术架构

### 10.1 明确技术栈

Vibemux 明确采用：

- 桌面壳：Tauri
- 后端核心：Rust
- 终端渲染：xterm.js
- 前端语言：TypeScript
- 前端框架：Svelte 或 Solid，推荐 Svelte
- PTY：portable-pty
- 异步运行时：tokio
- 配置序列化：serde + TOML
- 事件协议：Tauri commands + events，后续可升级到独立 mux daemon IPC

### 10.2 架构总览

```text
┌──────────────────────────────────────────────────────────┐
│ Tauri Desktop Client                                     │
│                                                          │
│  TypeScript / Svelte UI                                  │
│  ├─ Deck UI                                              │
│  ├─ Shelf UI                                             │
│  ├─ Settings UI                                          │
│  ├─ Keymap / Navigation Mode                             │
│  └─ xterm.js Terminal Renderer                           │
│                                                          │
├──────────────────── Tauri Command/Event Bridge ──────────┤
│                                                          │
│ Rust Application Layer                                   │
│  ├─ Command handlers                                     │
│  ├─ Event dispatcher                                     │
│  ├─ Config manager                                       │
│  └─ Window integration                                   │
│                                                          │
├──────────────────────────────────────────────────────────┤
│ Rust Mux Core                                            │
│  ├─ Session model                                        │
│  ├─ Workspace model                                      │
│  ├─ Thermal lifecycle                                    │
│  ├─ Ring buffer                                          │
│  ├─ Attention state                                      │
│  └─ Persistence                                          │
│                                                          │
├──────────────────────────────────────────────────────────┤
│ Rust PTY Host                                            │
│  ├─ portable-pty                                         │
│  ├─ PTY reader                                           │
│  ├─ PTY writer                                           │
│  ├─ resize                                               │
│  └─ process lifecycle                                    │
└──────────────────────────────────────────────────────────┘
```

### 10.3 关键架构原则

1. 前端不直接管理进程。
2. 前端不直接访问文件系统。
3. Rust mux core 是 session truth source。
4. xterm.js 只负责 hot session 渲染。
5. warm session 输出进入 Rust ring buffer。
6. Tauri bridge 只传递明确的 typed command 和 event。
7. 后续独立 daemon 化时，不改变前端核心协议。

### 10.4 为什么不用纯 Rust 渲染

不建议第一版使用纯 Rust GUI 重写终端渲染，因为：

- 终端模拟器兼容性复杂。
- 字体、CJK、emoji、IME、selection、alternate screen、鼠标事件、OSC hyperlink 都是深坑。
- xterm.js 成熟且生态稳定。
- 产品创新点在 mux 和 session workflow，不在重写 terminal renderer。

因此：Rust 负责 mux，xterm.js 负责 terminal surface。

## 11. 目标项目结构

建议项目结构：

```text
vibemux/
  apps/
    desktop/
      package.json
      src/
        main.ts
        App.svelte
        terminal/
          TerminalPane.svelte
          xtermLifecycle.ts
          xtermTheme.ts
          terminalReplay.ts
          terminalResize.ts
        deck/
          Deck.svelte
          DeckPane.svelte
          deckLayout.ts
          deckNavigation.ts
        shelf/
          Shelf.svelte
          ShelfCard.svelte
          shelfNavigation.ts
        session/
          NewSessionDialog.svelte
          SessionSearch.svelte
          sessionStore.ts
          sessionTypes.ts
        settings/
          SettingsPanel.svelte
          themeEditor.ts
          configSchema.ts
        keymap/
          keymap.ts
          modes.ts
          commandRegistry.ts
        bridge/
          commands.ts
          events.ts
          protocol.ts
      src-tauri/
        Cargo.toml
        tauri.conf.json
        src/
          lib.rs
          main.rs
          commands.rs
          events.rs
          config.rs
          window.rs

  crates/
    mux-core/
      Cargo.toml
      src/
        lib.rs
        model.rs
        session.rs
        workspace.rs
        lifecycle.rs
        commands.rs
        events.rs
        ring_buffer.rs
        attention.rs
        persistence.rs

    pty-host/
      Cargo.toml
      src/
        lib.rs
        pty.rs
        platform.rs
        reader.rs
        writer.rs
        resize.rs
        process.rs

    mux-protocol/
      Cargo.toml
      src/
        lib.rs
        command.rs
        event.rs
        snapshot.rs

    agent-detect/
      Cargo.toml
      src/
        lib.rs
        generic.rs
        claude.rs
        codex.rs

  docs/
    PRD.md
    ARCHITECTURE.md
    CONFIG.md
    KEYMAP.md
```

## 12. 数据模型

### 12.1 Rust Session

```rust
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub cwd: PathBuf,
    pub command: SessionCommand,
    pub color: ColorToken,
    pub workspace_id: WorkspaceId,
    pub thermal_state: ThermalState,
    pub process_state: ProcessState,
    pub attention_state: AttentionState,
    pub terminal_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub ring_buffer: OutputRingBuffer,
}
```

### 12.2 ThermalState

```rust
pub enum ThermalState {
    Hot,
    Warm,
    Cold,
}
```

### 12.3 ProcessState

```rust
pub enum ProcessState {
    Starting,
    Running,
    Exited { code: Option<i32> },
    FailedToStart { message: String },
    Killed,
}
```

### 12.4 AttentionState

```rust
pub enum AttentionState {
    Normal,
    Active,
    NeedsInput,
    Failed,
    Done,
    Exited,
}
```

### 12.5 Workspace

```rust
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub hot_session_ids: Vec<SessionId>,
    pub warm_session_ids: Vec<SessionId>,
    pub focused_session_id: Option<SessionId>,
    pub layout: LayoutKind,
}
```

## 13. Command / Event 协议

### 13.1 前端命令

```ts
type ClientCommand =
  | { type: 'session.create'; payload: CreateSessionPayload }
  | { type: 'session.write'; sessionId: string; data: string }
  | { type: 'session.resize'; sessionId: string; cols: number; rows: number }
  | { type: 'session.focus'; sessionId: string }
  | { type: 'session.park'; sessionId: string }
  | { type: 'session.recall'; sessionId: string; mode?: 'focused' | 'peripheral' }
  | { type: 'session.rename'; sessionId: string; name: string }
  | { type: 'session.close'; sessionId: string }
  | { type: 'session.kill'; sessionId: string }
  | { type: 'workspace.switch'; workspaceId: string }
  | { type: 'layout.set'; workspaceId: string; layout: 'deck' | 'monocle' | 'grid' }
  | { type: 'config.update'; patch: Partial<UserConfig> };
```

### 13.2 后端事件

```ts
type MuxEvent =
  | { type: 'session.created'; session: SessionSnapshot }
  | { type: 'session.output'; sessionId: string; data: string; seq: number }
  | { type: 'session.replay.start'; sessionId: string; fromSeq: number; toSeq: number }
  | { type: 'session.replay.chunk'; sessionId: string; data: string; seqStart: number; seqEnd: number }
  | { type: 'session.replay.end'; sessionId: string; toSeq: number }
  | { type: 'session.exited'; sessionId: string; code: number | null }
  | { type: 'session.updated'; session: SessionSnapshot }
  | { type: 'workspace.snapshot'; workspace: WorkspaceSnapshot }
  | { type: 'attention.changed'; sessionId: string; state: AttentionState }
  | { type: 'config.updated'; config: UserConfig }
  | { type: 'error'; message: string; commandId?: string };
```

### 13.3 输出顺序要求

每个 session 的输出必须有单调递增的 `seq`。

规则：

- 同一 session 的 output event 必须保持顺序。
- replay 必须明确起止范围。
- recall 时 replay 完成前，不应向前端 terminal 写入新的 live output。
- 如果前端检测到 seq gap，应请求 resync。

## 14. Replay 与 Buffer 策略

### 14.1 Hot Session

Hot session 的输出直接进入 xterm.js，同时也可以写入 mux core ring buffer。

Hot session 必须保留：

- xterm instance
- DOM node
- scrollback
- terminal visual state
- selection
- cursor state
- focused/peripheral 状态

### 14.2 Warm Session

Warm session 不保留 xterm instance。

Warm session 必须保留：

- PTY process
- ring buffer
- session metadata
- process state
- attention state
- last output summary

默认 buffer 建议：

- 最近 10,000 行
- 或最多 20MB
- 超限丢弃最旧输出

### 14.3 Recall Replay

Recall 流程：

1. 前端发送 `session.recall`。
2. Rust 将 session 标记为 Hot。
3. 前端创建 xterm.js instance。
4. Rust 发送 replay start。
5. Rust 分块发送 replay chunk。
6. 前端分帧写入 xterm。
7. Rust 发送 replay end。
8. 前端进入 live attach 状态。
9. 后续 output 正常实时写入。

### 14.4 Replay 性能要求

- replay 不能一次性写入大文本。
- 前端应使用 chunk queue。
- 每帧写入量受控。
- replay 超过 200ms 时显示轻量状态。
- replay 不得阻塞键盘导航。

### 14.5 Alternate Screen 限制

MVP 不保证完美恢复 `vim`、`top`、`less` 等 alternate screen 画面。

策略：

- 允许 Park。
- Recall 后 replay 最近 buffer。
- 保证 PTY 仍是同一个活进程。
- 后续再实现 terminal state snapshot。

## 15. 配置设计

### 15.1 配置文件示例

```toml
[terminal]
font_family = "JetBrains Mono"
font_size = 13
line_height = 1.2
scrollback_lines = 10000
replay_buffer_mb = 20

[theme]
background = "#111111"
foreground = "#d9d4c7"
cursor = "#ff6b57"
selection = "#ff6b5744"

[theme.ansi]
black = "#111111"
red = "#ff6b57"
green = "#98c379"
yellow = "#e5c07b"
blue = "#61afef"
magenta = "#c678dd"
cyan = "#56b6c2"
white = "#d9d4c7"

[layout]
default = "deck"
focused_pane_width = 720
preview_opacity = 0.8
animation_ms = 150
max_hot_sessions = 6
shelf_position = "bottom"

[shell]
default = "system"

[keys]
prefix = "Ctrl+B"
new_session = "N"
park_session = "B"
recall_session = "F"
next_hot = "L"
prev_hot = "H"
next_shelf = "J"
prev_shelf = "K"
search = "/"
```

### 15.2 配置要求

- 配置文件应按平台约定存放。
- GUI 修改配置时必须原子写入。
- 错误配置不能导致应用崩溃。
- 无法识别的配置项应 warning，但不阻止启动。
- GUI 设置和文件设置必须有一致的 schema。

## 16. 跨平台要求

### 16.1 macOS

- 支持 Apple Silicon 和 Intel。
- 默认使用用户 shell。
- 支持 login shell 配置。
- 支持剪贴板。
- 支持右键菜单。
- 支持系统打开链接。

### 16.2 Linux

- 支持主流桌面环境。
- 支持 Wayland 和 X11，具体限制需文档说明。
- 支持 bash、zsh、fish。
- 打包需覆盖 deb、AppImage 或其他常见格式。
- 需要明确 WebKitGTK 依赖。

### 16.3 Windows

- 使用 ConPTY。
- 默认 shell 为 PowerShell 或用户配置。
- 支持复制粘贴。
- 支持路径和 cwd 的 Windows 语义。
- 后续支持 WSL session。

## 17. 性能要求

### 17.1 启动性能

- 应用窗口 1.5 秒内可见。
- 默认 shell 2 秒内可交互，shell profile 自身耗时除外。

### 17.2 Hot 切换性能

- Hot session 切换感知延迟小于 50ms。
- 切换不重建 xterm。
- 切换不重新 spawn PTY。
- 6 个 hot session 内动画保持流畅。

### 17.3 输出性能

- 高频输出不能冻结 UI。
- Rust 侧应批处理 output event。
- 默认 batch interval 可从 8-16ms 起步。
- focused session 优先渲染。
- peripheral session 可以降频。
- warm session 不完整渲染，只更新 metadata。

### 17.4 内存要求

- hot session 数量默认限制。
- warm session 不保留 xterm instance。
- ring buffer 有明确上限。
- archive log 按需读取。

## 18. 安全要求

- 前端不能直接访问系统 shell、PTY、文件系统。
- 所有 Tauri command payload 必须校验。
- 用户模板命令执行前应清晰展示。
- Kill running process 需要确认，除非用户关闭确认。
- 打开链接必须使用平台安全 opener。
- 配置写入必须原子化，避免损坏。
- 日志路径和配置路径必须遵循平台约定。

## 19. 错误处理

必须处理：

- PTY 创建失败。
- shell 不存在。
- cwd 不存在或无权限。
- 配置文件损坏。
- replay buffer 不可用。
- session 在 recall 中退出。
- 前端 event stream 中断。
- WebGL 渲染失败。
- Tauri command 超时。

错误提示必须包含：

- 哪个 session 出错。
- 出错原因。
- 进程是否还活着。
- 用户可以采取什么操作。

## 20. 测试计划

### 20.1 Rust 单元测试

覆盖：

- Session lifecycle。
- Hot/Warm/Cold 转换。
- Ring buffer。
- Attention rule。
- Config parse。
- Workspace ordering。
- Event sequencing。

### 20.2 Rust 集成测试

覆盖：

- Spawn PTY。
- 写入命令。
- 读取输出。
- Resize。
- Kill。
- Exit code。
- 多 session 并发输出。

### 20.3 前端测试

覆盖：

- Deck layout。
- Shelf selection。
- Navigation Mode。
- xterm lifecycle。
- replay queue。
- settings form。
- theme mapping。

### 20.4 E2E 测试

覆盖：

- 创建 session。
- 输入命令。
- 新建多个 session。
- hot 切换。
- Park。
- Recall。
- Replay 验证。
- 修改字体。
- 修改主题。
- resize window。
- copy/paste。
- 右键菜单。

### 20.5 性能测试

场景：

- `yes` 高频输出。
- `find` 大目录输出。
- build command 大量日志。
- 6 个 hot sessions。
- 20 个 warm sessions。
- recall 最大 replay buffer。

成功标准：

- UI 不冻结。
- 内存不无限增长。
- output 顺序正确。
- hot 切换仍然流畅。

## 21. MVP 范围

MVP 必须包含：

- Tauri 桌面应用。
- Rust mux core。
- Rust PTY host。
- xterm.js terminal renderer。
- 水平 Deck。
- 2.5D 彩色 pane 框。
- Navigation Mode。
- GUI 创建 session。
- session rename。
- session color。
- hot session reorder。
- Park 到 Shelf。
- Recall 并 replay。
- Shelf card 状态。
- 字体设置。
- 主题设置。
- 配置持久化。
- copy/paste。
- 右键菜单。
- 进程退出检测。
- 基础 attention state。

MVP 可以暂缓：

- 独立 mux daemon。
- 多 workspace。
- 多窗口 attach。
- SSH/WSL domain。
- 完整 archive log 搜索。
- Agent 专用检测器。
- 插件系统。
- 完美 alternate screen snapshot。

## 22. 里程碑

### 22.1 Milestone 1：基础骨架

交付：

- Tauri app。
- Rust command bridge。
- mux-core crate。
- pty-host crate。
- 单 session 创建、输入、输出、resize。

验收：

- GUI 中可打开一个 shell。
- 可输入命令并看到输出。
- resize 正常。

### 22.2 Milestone 2：Deck 体验

交付：

- 多 hot session。
- 水平 Deck。
- focused/peripheral 状态。
- 2.5D 彩色框。
- 键盘切换。
- 鼠标聚焦。

验收：

- 多个 session 间切换不重建 terminal。
- 动画流畅。
- 当前交互体验符合产品方向。

### 22.3 Milestone 3：Shelf 和 Thermal Model

交付：

- Hot/Warm/Cold 状态。
- Park。
- Shelf card。
- Warm ring buffer。
- Recall。
- Replay。

验收：

- Park 不杀进程。
- Warm session 继续输出。
- Recall 不重启进程。
- Replay 顺序正确。

### 22.4 Milestone 4：配置和基础终端能力

交付：

- 字体设置。
- 主题设置。
- 配置持久化。
- copy/paste。
- 右键菜单。
- link open。

验收：

- 重启后设置保留。
- 常见终端操作可用。

### 22.5 Milestone 5：跨平台与发布准备

交付：

- macOS 包。
- Linux 包。
- Windows 包。
- E2E 测试。
- 性能测试。
- 用户文档。

验收：

- 三大平台可运行。
- MVP 主流程通过。
- 已知限制记录清楚。

## 23. 风险与缓解

### 23.1 xterm.js Replay 不完美

风险：Warm session recall 时，复杂 terminal 状态无法完美还原。

缓解：

- MVP 明确只保证最近输出 replay。
- 保证 PTY 不中断。
- 后续研究 terminal state snapshot。

### 23.2 Tauri WebView 平台差异

风险：Linux WebKitGTK、Windows WebView2、macOS WKWebView 表现不同。

缓解：

- 尽早做跨平台 smoke test。
- xterm renderer fallback。
- 记录平台限制。

### 23.3 高频输出性能

风险：大量 output 造成 bridge 压力和 UI 卡顿。

缓解：

- Rust 侧批处理。
- 前端分帧写入。
- peripheral 降频。
- warm session 不完整渲染。

### 23.4 产品概念过重

风险：同时做 Deck、Shelf、Workspace、Agent 检测导致 MVP 变慢。

缓解：

- MVP 只做单 workspace。
- Agent 检测先做通用规则。
- 独立 daemon 延后。

## 24. 开放问题

实现前需要确认：

- Recall 默认进入 focused 还是恢复原位置？
- Shelf 放底部、右侧还是 overlay？
- 默认 max hot sessions 是 4、6 还是用户配置？
- Warm session 默认保留多少 replay buffer？
- Archive log 是否默认开启？
- Park alternate-screen app 是否需要提示？
- session color 是自动分配、项目绑定还是用户手动优先？
- Navigation Mode 是否沿用 `Ctrl+B`，还是支持首次启动选择？

## 25. 最终验收标准

Vibemux MVP 完成时，必须满足：

1. 用户能通过 GUI 创建 session。
2. 用户能用键盘在 Deck 中切换 hot session。
3. Deck 中 session 切换必须瞬间恢复。
4. 用户能 Park 当前 session 到 Shelf。
5. Park 后进程继续运行。
6. Shelf card 能显示状态和最近输出。
7. 用户能 Recall warm session。
8. Recall 不重启进程。
9. Recall 时最近输出被 replay。
10. 用户能配置字体和主题。
11. 配置重启后保留。
12. copy/paste/right-click 可用。
13. macOS、Linux、Windows 均有可运行版本或明确打包路径。

## 26. 产品北极星

Vibemux 的长期目标不是替代所有终端，也不是复刻 tmux 的每个能力，而是重新定义本地终端多任务体验。

用户不应该再问：

- 我那个任务在哪个 tmux window？
- 那个 Agent 现在跑完了吗？
- 哪个 pane 在等我输入？
- 我能不能先把这个任务放到一边？

用户应该能一眼看到：

- 哪些任务在当前 Deck 中。
- 哪些任务在后台 Shelf 里继续运行。
- 哪些任务完成了。
- 哪些任务失败了。
- 哪些任务需要我处理。

Vibemux 的核心产品原则：

```text
用 GUI 建立任务语义。
用键盘执行高频流转。
用 Rust 保持 session 可靠。
用 xterm.js 保持终端兼容。
```

