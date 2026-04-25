# Vibe99 竞品分析：值得 Vibemux 借鉴的特性

Vibe99 是一个 Electron + xterm.js + node-pty 构建的终端工作区原型，核心理念是"focus + peripheral awareness"——一个 pane 全尺寸聚焦，其余 pane 压缩为窄预览。名字来自 Tetris 99：你专注自己的棋盘，同时余光扫视周围。

## 1. 产品理念对比

| 维度 | Vibe99 | Vibemux |
|------|--------|---------|
| 核心隐喻 | 卡牌 Deck，一张大 + 多张小 | 热力生命周期 Hot/Warm/Cold |
| 注意力模型 | 空间压缩（窄预览始终可见） | 状态分层（Shelf 隐藏终端，保留卡片） |
| 技术栈 | Electron + node-pty | Tauri + Rust PTY |
| 成熟度 | 早期原型，无持久化 | 已有完整 session 生命周期 |

两者解决同一个问题——"多终端场景下注意力分配不均"——但切入角度不同。Vibe99 用空间压缩让所有 pane 始终在视野内；Vibemux 用 thermal state 把不需要的终端彻底移出 DOM。

## 2. 值得借鉴的特性

### 2.1 Pane 透明度渐变（Peripheral Dimming）

Vibe99 的非聚焦 pane 默认 opacity 0.8，聚焦 pane 强制 opacity 1.0。这个简单的视觉层级让用户不需要看边框颜色就能瞬间识别哪个是当前焦点。

**借鉴建议**：Vibemux 的 DeckPane 目前靠 2.5D 彩色边框区分聚焦态。可以叠加一层 opacity 渐变——非聚焦 pane 整体降低 10-20% 亮度。成本极低，感知收益明显。

### 2.2 Pane 宽度实时可调（Live Pane Width Slider）

Vibe99 提供 520px–1000px 的 pane 宽度滑块，拖动时所有 pane 实时重排。这让用户可以根据屏幕尺寸和任务内容动态调整"聚焦区 vs 预览区"的比例。

**借鉴建议**：Vibemux 的 `deckLayout.ts` 目前用固定算法计算 pane 尺寸。可以在 Settings 中暴露一个"聚焦 pane 宽度比例"滑块（比如 60%–90%），让用户在宽屏和窄屏场景下自行调节。

### 2.3 每个 Pane 独立 Accent Color + 色板轮转

Vibe99 预定义了 10 色色板，新建 pane 自动轮转分配颜色。颜色同时应用于：
- Tab 左侧色块（swatch）
- 聚焦 Tab 的顶部 inset box-shadow
- Pane 边框
- 终端光标颜色
- 终端选区背景色

Vibemux 已经有 ColorToken 机制，但 Vibe99 的做法更彻底——光标和选区也跟随 accent color，形成完整的颜色身份。

**借鉴建议**：让 TerminalPane 的 xterm 光标色和选区色也跟随 session 的 ColorToken，强化"颜色 = 身份"的认知。

### 2.4 Tab 拖拽重排（Drag Reorder）

Vibe99 实现了完整的 tab 拖拽重排：pointer down → move 超过 4px 阈值 → 实时显示插入指示线 → pointer up 完成排序。拖拽过程中 tab 有 `is-dragging` 和 `insert-before` 视觉反馈。

**借鉴建议**：Vibemux 目前 Deck 中 session 顺序由 `sessions` 数组决定，没有拖拽重排。对于 3+ 个 hot session 的场景，拖拽重排比键盘 move 更直觉。可以作为 P2 特性加入。

### 2.5 双击 Tab 重命名（Inline Rename）

Vibe99 的 tab 双击直接变成 input 框，Enter 确认，Escape 取消，blur 自动提交。交互非常轻量。

Vibemux 已有 rename 功能（nav mode 下按 `r`），但 Vibe99 的双击 inline 编辑作为鼠标用户的补充路径值得考虑。

### 2.6 Navigation Mode 的状态栏反馈

Vibe99 进入 navigation mode 时：
- 状态栏文字变为黄色加粗 "Navigation Mode"
- 右侧显示操作提示 "Left/Right or H/L to flip; Enter to Focus"
- 聚焦 pane 边框加粗到 4px + 外发光

这种"模式 + 提示 + 视觉强化"三重反馈很清晰。

**借鉴建议**：Vibemux 的 nav mode 已有类似设计，但可以检查是否在状态栏同时显示了可用操作的快捷键提示。新用户最需要的是"我现在能按什么键"。

### 2.7 背景微渐变（Ambient Gradient）

Vibe99 的 body 背景不是纯黑，而是叠加了两个 radial-gradient（左上暖黄、右上青色，各 8% 透明度）。这个细节让整个界面有"空间感"而不是平板黑。

**借鉴建议**：Vibemux 可以在 Deck 区域背景加一层极低透明度的 radial gradient，增加视觉层次。这是纯 CSS 改动，零性能成本。

### 2.8 Capture Mode（截图模式）

Vibe99 通过 `VIBE99_CAPTURE=1` 环境变量启动无头截图模式，自动等待渲染完成后导出 PNG。这对生成 README demo 图、CI 视觉回归测试都很有用。

**借鉴建议**：Vibemux 可以考虑类似的 headless capture 能力，用于自动化截图生成和视觉测试。Tauri 的 webview 也支持 `capturePage` 等价操作。

## 3. Vibe99 的局限（Vibemux 已超越的部分）

- **无持久化**：Vibe99 没有 config 持久化，重启丢失一切。Vibemux 已有 TOML config + atomic save。
- **无 Shelf/Warm 概念**：Vibe99 所有 pane 始终是 hot 的，没有 park/recall 机制。这意味着 10+ pane 时性能和认知都会崩溃。Vibemux 的 thermal lifecycle 是核心优势。
- **无 ring buffer**：Vibe99 没有输出缓冲，无法在 pane 销毁后恢复历史输出。Vibemux 的 OutputRingBuffer + replay 机制远超。
- **无 workspace**：Vibe99 是单一扁平 pane 列表。Vibemux 已有 workspace 概念。
- **无进程状态感知**：Vibe99 不追踪进程是否退出、是否需要注意。Vibemux 有 ProcessState + AttentionState。
- **Electron 包体**：Electron 打包体积远大于 Tauri。

## 4. 优先级建议

| 特性 | 实现成本 | 体验收益 | 优先级 |
|------|---------|---------|--------|
| Pane opacity 渐变 | 低（CSS） | 中 | P1 |
| 光标/选区跟随 accent color | 低（xterm theme） | 中 | P1 |
| 背景微渐变 | 极低（CSS） | 低 | P2 |
| 聚焦 pane 宽度可调 | 中（layout 逻辑） | 中 | P2 |
| Tab 拖拽重排 | 中（Svelte 拖拽） | 中 | P2 |
| 双击 tab inline rename | 低 | 低 | P3 |
| Capture/截图模式 | 中 | 低（开发工具） | P3 |
