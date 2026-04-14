# Vibemux UX 易用性改进 PRD

- 产品名称：Vibemux
- 文档版本：v0.1
- 技术栈：Tauri 2 + Rust + Svelte 5 + xterm.js
- 文档状态：待实现

## 1. Introduction / Overview

Vibemux MVP 功能已基本完成，但首次启动体验存在三个核心问题：

1. **Prefix Key 冲突**：默认 `Cmd+Space` 与 macOS Spotlight 冲突，导致 Navigation Mode 无法激活。
2. **缺少 GUI 操作入口**：所有高频操作（新建 session、设置、Park、Close）都依赖键盘快捷键，鼠标用户无法发现和使用这些功能。
3. **缺少首次启动引导**：用户打开应用只看到一个空终端，不知道 Vibemux 的核心功能（Deck、Shelf、Navigation Mode）如何使用。

本 PRD 覆盖四个改进方向：

- **A. Prefix Key 修复**：默认改为 `Ctrl+B`，支持自定义
- **B. 顶部工具栏**：集成 + (新建)、搜索、⚙ (设置) 按钮
- **C. 首次启动 Onboarding**：引导用户选择 prefix key、shell、主题、字体
- **D. Deck/Shelf 右键菜单和操作按钮**：让所有功能都可以不用键盘完成

## 2. Goals

- 首次启动用户在 30 秒内完成基础配置并理解核心操作
- 所有 MVP 功能（新建、Park、Recall、Close、Kill、Rename、Settings、Search）都可通过鼠标完成
- Prefix Key 默认值不与任何主流 OS 快捷键冲突
- 现有键盘用户的体验不受影响（工具栏不占用过多空间）

## 3. User Stories

---

### US-031: 修改默认 Prefix Key 为 Ctrl+B

**Description:** 作为 macOS 用户，我希望默认 Prefix Key 不与 Spotlight 冲突，以便我能正常使用 Navigation Mode。

**Acceptance Criteria:**
- [ ] Rust `KeysConfig` 默认值从 `cmd+space` 改为 `ctrl+b`
- [ ] 前端 `App.svelte` 的 prefix key 检测逻辑从硬编码改为读取配置
- [ ] 启动时通过 `config_get` 获取 `keys.prefix` 值，动态绑定 prefix key
- [ ] 支持的 prefix key 格式：`ctrl+b`、`cmd+space`、`ctrl+space`、`ctrl+\``（反引号）等常见组合
- [ ] prefix key 解析逻辑能正确处理 `ctrl`、`cmd`（macOS meta）、`alt`、`shift` 修饰符 + 任意单键
- [ ] Nav Mode 指示条中显示当前实际的 prefix key（而非硬编码 `Cmd+Space`）
- [ ] HelpOverlay 中显示当前实际的 prefix key
- [ ] Typecheck passes

---

### US-032: 添加顶部工具栏 (Titlebar)

**Description:** 作为用户，我希望窗口顶部有一个工具栏，包含常用操作按钮，以便我不依赖键盘也能操作。

**Acceptance Criteria:**
- [ ] 新建 `Titlebar.svelte` 组件，固定在窗口顶部
- [ ] 工具栏高度 36px，背景色与应用主题一致（`#111111` 或略浅），底部有 1px 分隔线
- [ ] 左侧显示当前 Workspace 标识：颜色圆点 + 数字编号（如 `●1 Default`），点击可重命名
- [ ] 中间留空（后续可放 session tabs 等）
- [ ] 右侧按钮组，从左到右：
  - `+` 按钮：点击打开 New Session 面板（等同于 Nav Mode + N）
  - 🔍 按钮：点击打开 Session Search（等同于 Nav Mode + /）
  - `⚙` 按钮：点击打开 Settings 面板
- [ ] 每个按钮 hover 时显示 tooltip，包含对应的键盘快捷键（如 "New Session (Ctrl+B, N)"）
- [ ] 移除原有的右下角浮动 ⚙ 按钮（`settings-btn`），功能已迁移到工具栏
- [ ] Deck 区域高度 = 100vh - titlebar高度 - shelf高度（如有）
- [ ] 工具栏不可被终端内容遮挡（z-index 合理）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-033: 首次启动 Onboarding — Prefix Key 选择

**Description:** 作为首次使用 Vibemux 的用户，我希望在第一次启动时选择我偏好的 Prefix Key，以便快捷键不与我的习惯冲突。

**Acceptance Criteria:**
- [ ] 新建 `Onboarding.svelte` 组件，全屏 overlay，分步骤展示
- [ ] 检测是否首次启动：读取配置文件中的 `onboarding_completed` 布尔字段（默认 false）
- [ ] 新增 Rust `config.rs` 中 `UserConfig` 的 `onboarding_completed: bool` 字段（`#[serde(default)]`，默认 false）
- [ ] Step 1: Prefix Key 选择
  - 标题："Choose your Navigation Key"
  - 说明文字：简短解释 prefix key 的作用（"Press this key combo to enter Navigation Mode, where you can switch sessions, park, recall, and more."）
  - 提供 4 个选项，每个选项显示组合键和简短说明：
    - `Ctrl+B` — "tmux style (Recommended)"
    - `Ctrl+Space` — "Spacemacs style"
    - `Ctrl+\`` — "Backtick (no conflicts)"
    - `Ctrl+A` — "screen style"
  - 用户点击选项即选中（高亮），底部 "Next" 按钮进入下一步
  - 选中后立即写入 `keys.prefix` 配置
- [ ] 如果用户关闭 onboarding（点 X 或 Esc），使用默认值（`ctrl+b`）并标记 `onboarding_completed = true`
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-034: 首次启动 Onboarding — Shell 和外观配置

**Description:** 作为首次使用 Vibemux 的用户，我希望在引导流程中配置默认 shell、主题和字体，以便应用从一开始就符合我的偏好。

**Acceptance Criteria:**
- [ ] Step 2: Default Shell 选择
  - 标题："Choose your default shell"
  - 自动检测系统可用 shell（通过 Tauri command `detect_shells` 返回可用 shell 列表）
  - 新增 Rust `detect_shells` Tauri command：在 macOS/Linux 上读取 `/etc/shells`，在 Windows 上检测 PowerShell 和 cmd 路径
  - 显示检测到的 shell 列表（如 `/bin/zsh`、`/bin/bash`、`/bin/fish`），用户点击选择
  - 默认高亮当前 `$SHELL` 环境变量对应的 shell
  - 选中后写入 `shell.default` 配置
- [ ] Step 3: 主题和字体
  - 标题："Customize your terminal"
  - 左侧：3 个预设主题卡片（Dark（默认）、Solarized Dark、Monokai），点击切换，实时预览
  - 每个预设主题包含完整定义：background、foreground、cursor、selection + ANSI 16 色（black/red/green/yellow/blue/magenta/cyan/white + 8 bright 变体）
  - 右侧：字体族下拉（monospace、JetBrains Mono、Fira Code、Menlo）+ 字号滑块（10-20，默认 14）
  - 底部有一个小的终端预览区域，显示几行彩色示例文本（包含 ANSI 颜色），实时反映主题和字体选择
  - 选中后写入 `theme.*`（含所有 ANSI 色）和 `terminal.font_family`、`terminal.font_size` 配置
- [ ] Step 4: 完成
  - 标题："You're all set!"
  - 简短提示核心操作："Press [prefix key] to enter Navigation Mode. Try creating a new session with [prefix key] → N."
  - "Start" 按钮关闭 onboarding
  - 点击 "Start" 后设置 `onboarding_completed = true` 并写入配置
- [ ] 每一步都有 "Back" 按钮（第一步除外）和 "Skip" 链接（跳过当前步骤使用默认值）
- [ ] Onboarding 完成后，App.svelte 正常启动（创建默认 session）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-035: Deck Pane 标题栏操作按钮

**Description:** 作为用户，我希望每个 Deck Pane 的标题栏上有操作按钮，以便我可以用鼠标 Park、Close 或 Rename session。

**Acceptance Criteria:**
- [ ] DeckPane 标题栏右侧添加操作按钮组，仅在 focused pane 且 hover 时显示
- [ ] 按钮组包含（从左到右）：
  - 📌 Park 按钮（tooltip: "Park to Shelf (Ctrl+B, B)"）：点击调用 `session_park`
  - ✏️ Rename 按钮（tooltip: "Rename (Ctrl+B, R)"）：点击进入 inline rename 模式
  - ✕ Close 按钮（tooltip: "Close (Ctrl+B, X)"）：点击调用 `session_close`
- [ ] 按钮图标使用简单的 Unicode 字符或 SVG，不引入图标库
- [ ] 按钮不影响终端区域的点击和输入（不拦截终端事件）
- [ ] 按钮 hover 时有轻微背景高亮
- [ ] 只在 focused pane 上显示按钮，peripheral pane 不显示（通过右键菜单操作）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-036: Shelf Card 右键菜单

**Description:** 作为用户，我希望右键点击 Shelf Card 时出现操作菜单，以便我可以用鼠标 Recall、Rename、Kill 后台 session。

**Acceptance Criteria:**
- [ ] 右键点击 ShelfCard 弹出上下文菜单，菜单项：
  - "Recall to Deck" — 调用 `session_recall`
  - "Rename" — 弹出 inline 输入框或小弹窗，输入新名称后调用 `session_rename`
  - "Change Color" — 展开子菜单显示 8 个颜色圆点，点击调用 `session_set_color`
  - 分隔线
  - "Close" — 调用 `session_close`
  - "Kill" — 调用 `session_kill`（红色文字）
- [ ] 菜单样式与 TerminalPane 的右键菜单一致（`#1e1e1e` 背景，`#333` 边框，圆角）
- [ ] 点击菜单外区域关闭菜单
- [ ] 左键点击 ShelfCard 仍然直接触发 Recall（保持现有行为）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-037: Deck Pane 右键菜单

**Description:** 作为用户，我希望右键点击 Deck Pane 的标题栏区域时出现操作菜单，提供完整的 session 管理选项。

**Acceptance Criteria:**
- [ ] 右键点击 DeckPane 的 `.drag-handle` 或 `.pane-header` 区域弹出上下文菜单
- [ ] 菜单项：
  - "Park to Shelf" — 调用 `session_park`
  - "Rename" — 进入 inline rename 模式
  - "Change Color" — 展开子菜单显示 8 个颜色圆点
  - 分隔线
  - "Close" — 调用 `session_close`
  - "Kill" — 调用 `session_kill`（红色文字）
- [ ] 右键点击终端内容区域仍然显示原有的 Copy/Paste/Clear 菜单（不受影响）
- [ ] 菜单样式与其他右键菜单一致
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

## 4. Functional Requirements

### Prefix Key

- FR-1: 默认 prefix key 为 `ctrl+b`，不再使用 `cmd+space`
- FR-2: prefix key 从配置文件 `keys.prefix` 字段读取，支持运行时修改
- FR-3: prefix key 格式为 `modifier+key`，支持的 modifier：`ctrl`、`cmd`（macOS meta）、`alt`、`shift`；支持组合如 `ctrl+shift+b`
- FR-4: 前端解析 prefix key 字符串为 `KeyboardEvent` 匹配条件（modifier flags + key/code）
- FR-5: Settings 面板的 Keys tab 中可修改 prefix key（下拉选择常用组合 + 自定义输入）

### Titlebar

- FR-6: Titlebar 固定在窗口顶部，高度 36px
- FR-7: Titlebar 左侧显示 Workspace 标识（颜色圆点 + 数字编号，可重命名），右侧显示操作按钮组
- FR-8: 操作按钮：New Session (+)、Search (🔍)、Settings (⚙)
- FR-9: 按钮 tooltip 显示功能名称和对应快捷键
- FR-10: Titlebar 在所有状态下可见（包括 onboarding 完成后）

### Onboarding

- FR-11: 首次启动检测基于配置文件中的 `onboarding_completed` 字段
- FR-12: Onboarding 为全屏 overlay，共 4 步：Prefix Key → Shell → Theme/Font → Done
- FR-13: 每步选择立即写入配置（不等到最后一步）
- FR-14: 用户可随时跳过（Skip）或关闭（X/Esc），使用默认值
- FR-15: Onboarding 完成后不再显示（除非用户手动重置配置）
- FR-16: `detect_shells` Tauri command 返回系统可用 shell 列表

### Context Menus & Buttons

- FR-17: DeckPane 标题栏 hover 时显示 Park / Rename / Close 按钮（仅 focused pane）
- FR-18: DeckPane 标题栏右键菜单包含 Park / Rename / Change Color / Close / Kill（所有 pane）
- FR-19: ShelfCard 右键菜单包含 Recall / Rename / Change Color / Close / Kill
- FR-20: 所有右键菜单样式统一（可抽取为共享的 `ContextMenu.svelte` 组件）
- FR-21: 颜色选择子菜单显示 8 个预设颜色圆点，点击即应用

## 5. Non-Goals (Out of Scope)

- 自定义 Navigation Mode 内部键位映射（h/l/j/k 等）— 后续版本
- 可拖拽的工具栏或自定义工具栏按钮
- 多语言 / i18n 支持
- 导入/导出配置文件
- 预设主题包管理（onboarding 中的 3 个预设主题是硬编码的）
- 交互式教学 / 引导式 tour（只做配置向导，不做功能教学）

## 6. Design Considerations

### Titlebar 布局

```
┌─────────────────────────────────────────────────────────────┐
│ ●1 Default                                 [+] [🔍] [⚙]    │  36px
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                     Deck Area                               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ SHELF  [card] [card] [card]                                 │  62px
└─────────────────────────────────────────────────────────────┘
```

### Onboarding 布局

```
┌─────────────────────────────────────────────────────────────┐
│                                                        [✕]  │
│                                                             │
│              Choose your Navigation Key                     │
│                                                             │
│    ┌──────────────┐  ┌──────────────┐                       │
│    │  Ctrl + B    │  │ Ctrl + Space │                       │
│    │  tmux style  │  │  Spacemacs   │                       │
│    │ (Recommended)│  │              │                       │
│    └──────────────┘  └──────────────┘                       │
│    ┌──────────────┐  ┌──────────────┐                       │
│    │  Ctrl + `    │  │  Ctrl + A    │                       │
│    │  Backtick    │  │ screen style │                       │
│    └──────────────┘  └──────────────┘                       │
│                                                             │
│              Step 1 of 4    ● ○ ○ ○                         │
│                                                             │
│                          [Skip]  [Next →]                   │
└─────────────────────────────────────────────────────────────┘
```

### DeckPane 标题栏（hover 状态）

```
┌─ drag handle (colored) ─────────────────────────────────────┐
│ session-name                          [📌] [✏️] [✕]         │  22px header
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                    terminal content                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 共享 ContextMenu 组件

建议抽取 `ContextMenu.svelte`，接受 `items` 数组和 `position`，统一样式。TerminalPane、DeckPane、ShelfCard 都使用同一个组件。

## 7. Technical Considerations

### Prefix Key 解析

需要一个 `parsePrefixKey(configString: string) → { ctrlKey, metaKey, altKey, shiftKey, key }` 函数，将配置字符串（如 `"ctrl+b"`）解析为 `KeyboardEvent` 匹配条件。放在 `src/lib/keymap.ts` 中。

注意事项：
- `cmd` 在 macOS 上映射到 `metaKey`，在其他平台上映射到 `ctrlKey`（或忽略）
- `ctrl+\`` 中的反引号需要匹配 `e.key === "\`"` 或 `e.code === "Backquote"`
- 配置字符串统一小写存储

### Shell 检测

- macOS/Linux: 读取 `/etc/shells` 文件，过滤存在的路径
- Windows: 检测 `powershell.exe`、`pwsh.exe`、`cmd.exe` 的存在
- 返回 `Vec<String>`，前端展示为列表

### Onboarding 预设主题

硬编码 3 个完整主题对象（含 ANSI 16 色）：

```typescript
const PRESET_THEMES = {
  dark: {
    background: "#111111", foreground: "#d9d4c7", cursor: "#ff6b57", selection: "#ff6b5744",
    black: "#111111", red: "#ff6b57", green: "#98c379", yellow: "#e5c07b",
    blue: "#61afef", magenta: "#c678dd", cyan: "#56b6c2", white: "#d9d4c7",
    bright_black: "#5c6370", bright_red: "#e06c75", bright_green: "#98c379", bright_yellow: "#e5c07b",
    bright_blue: "#61afef", bright_magenta: "#c678dd", bright_cyan: "#56b6c2", bright_white: "#ffffff",
  },
  solarized: {
    background: "#002b36", foreground: "#839496", cursor: "#cb4b16", selection: "#073642",
    black: "#073642", red: "#dc322f", green: "#859900", yellow: "#b58900",
    blue: "#268bd2", magenta: "#d33682", cyan: "#2aa198", white: "#eee8d5",
    bright_black: "#586e75", bright_red: "#cb4b16", bright_green: "#586e75", bright_yellow: "#657b83",
    bright_blue: "#839496", bright_magenta: "#6c71c4", bright_cyan: "#93a1a1", bright_white: "#fdf6e3",
  },
  monokai: {
    background: "#272822", foreground: "#f8f8f2", cursor: "#f92672", selection: "#49483e",
    black: "#272822", red: "#f92672", green: "#a6e22e", yellow: "#f4bf75",
    blue: "#66d9ef", magenta: "#ae81ff", cyan: "#a1efe4", white: "#f8f8f2",
    bright_black: "#75715e", bright_red: "#f92672", bright_green: "#a6e22e", bright_yellow: "#f4bf75",
    bright_blue: "#66d9ef", bright_magenta: "#ae81ff", bright_cyan: "#a1efe4", bright_white: "#f9f8f5",
  },
};
```

### 配置变更的实时生效

当 prefix key 在 Settings 或 Onboarding 中被修改时，`App.svelte` 需要重新读取配置并更新 prefix key 匹配逻辑。可以通过：
- `config_update` 返回更新后的完整配置
- App.svelte 监听配置变更事件或在 Settings/Onboarding 关闭时重新 `config_get`

## 8. Success Metrics

- 首次启动到完成 onboarding 并看到可交互终端：< 30 秒
- 所有 MVP 功能（新建、Park、Recall、Close、Kill、Rename、Search、Settings）都可通过鼠标完成
- Prefix Key 默认值在 macOS、Linux、Windows 上均不与系统快捷键冲突
- 现有键盘快捷键全部保持可用

## 9. Resolved Questions

- **预设主题 ANSI 色**：需要包含 ANSI 16 色的完整定义，每个预设主题都是完整的颜色方案。
- **Titlebar 显示内容**：显示当前 Workspace 的颜色 + 数字编号（如 `●1`），允许重命名。Session 也用颜色 + 数字区分。后续多 workspace 时自然扩展。
- **DeckPane 标题栏按钮**：只在 focused pane 上 hover 时显示，peripheral pane 通过右键菜单操作。
