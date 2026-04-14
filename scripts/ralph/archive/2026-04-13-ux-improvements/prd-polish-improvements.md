# Vibemux 界面打磨改进 PRD

- 产品名称：Vibemux
- 文档版本：v0.1
- 技术栈：Tauri 2 + Rust + Svelte 5 + xterm.js
- 文档状态：待实现

## 1. Introduction / Overview

Vibemux 核心功能已完成，本 PRD 覆盖一批界面打磨和体验改进，包括：

1. **Shelf 高度减半** — 当前 Shelf 占用空间过多，压缩终端可用区域
2. **Rename 被 PWD 覆盖** — 用户重命名后很快被 PWD 更新覆盖，自定义名称无法保留
3. **多 Session 文字挤压** — 水平排布时非 focused pane 的标题文字被压缩，应改为 clip/overflow
4. **Settings 缺少 Prefix Key 调整** — 设置面板没有修改 prefix key 的入口
5. **字体下拉选择** — 设置中字体应有下拉菜单，列出系统已安装的等宽字体
6. **Theme 预设选项** — 设置中 Theme 页需要内置预设主题供用户一键切换
7. **Session 间主题色边框** — 多 Session 水平排布时，每个 Session 用细主题色边框包裹
8. **窗口标题动态化** — 窗口 title 显示当前 Hot Session 的名称和状态
9. **2.5D 视觉增强** — 整体界面加深阴影、边框发光，强化立体感
10. **Session Busy 状态图标** — Session 有活动进程时，Shelf Card 和 DeckPane 标题显示旋转方形图标

## 2. Goals

- Shelf 不再占用过多垂直空间，终端区域更宽敞
- 用户重命名的 session 名称能持久保留，不被 PWD 覆盖
- 多 session 布局下标题文字清晰可读，不变形
- 所有常用配置（prefix key、字体、主题）都可在 Settings 中完成
- 界面视觉层次感更强，符合 2.5D 设计语言
- 用户能直观感知哪些 session 正在运行任务

## 3. User Stories

---

### US-044: Shelf 高度减半

**Description:** 作为用户，我希望 Shelf 区域高度减小到当前的一半，以便终端内容有更多显示空间。

**Acceptance Criteria:**
- [ ] 找到 Shelf 当前高度定义（CSS 变量或固定值），将其减半
- [ ] Shelf Card 内部布局适配新高度（文字、图标不溢出）
- [ ] Deck 区域高度自动填充剩余空间（flex 布局，无需硬编码）
- [ ] Shelf 折叠/展开状态下高度均正确
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-045: 修复 Rename 被 PWD 覆盖问题

**Description:** 作为用户，我希望重命名 session 后自定义名称能持久保留，PWD 变化不覆盖它，但 PWD 仍在副标题中显示。

**Acceptance Criteria:**
- [ ] `Session` 结构体新增 `custom_name: Option<String>` 字段，表示用户手动设置的名称
- [ ] `session_rename` 命令将名称写入 `custom_name`，同时设置 `name` 字段
- [ ] PWD 更新逻辑（`ProcessState` 变化时）只更新 `name` 字段，当 `custom_name` 存在时不覆盖 `name`
- [ ] `SessionSnapshot` 中暴露 `custom_name: Option<String>` 字段给前端
- [ ] DeckPane 标题栏：主标题显示 `custom_name`（若有）否则显示 `name`；副标题显示当前 PWD（`cwd` 字段）
- [ ] ShelfCard 同样逻辑：主标题优先显示 `custom_name`
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-046: 多 Session 水平排布标题文字 clip 而非压缩

**Description:** 作为用户，我希望多 session 水平排布时，pane 标题文字不被压缩变形，而是超出部分直接裁切（clip）。

**Acceptance Criteria:**
- [ ] DeckPane 标题栏的 session 名称元素设置 `overflow: hidden; white-space: nowrap; text-overflow: clip`（不用 ellipsis，直接裁切）
- [ ] 字体大小（`font-size`）在任何 pane 宽度下保持不变，不使用 `font-size` 缩放
- [ ] 标题栏高度固定，不因内容变化而改变
- [ ] 在 2、3、4 个 session 水平排布时验证文字不变形
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-047: Settings 增加 Prefix Key 调整入口

**Description:** 作为用户，我希望在 Settings 面板中能修改 Prefix Key，以便不需要手动编辑配置文件。

**Acceptance Criteria:**
- [ ] Settings 面板的 Keys（或 General）tab 中新增 "Navigation Prefix Key" 配置项
- [ ] 提供下拉选择框，选项包含：`Ctrl+B`、`Ctrl+Space`、`Ctrl+\``、`Ctrl+A`、`Cmd+Space`（macOS）
- [ ] 下拉框下方有一个文本输入框，允许用户输入自定义组合键（如 `ctrl+shift+x`）
- [ ] 选择或输入后，点击 Save / 失焦时调用 `config_update` 写入 `keys.prefix`
- [ ] 修改后 `App.svelte` 立即更新 prefix key 匹配逻辑（无需重启）
- [ ] 当前值从 `config_get` 读取并回显
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-048: Settings 字体下拉选择

**Description:** 作为用户，我希望在 Settings 的字体配置中有下拉菜单，列出系统已安装的等宽字体，方便选择。

**Acceptance Criteria:**
- [ ] 新增 Rust Tauri command `list_monospace_fonts`，返回系统已安装的等宽字体列表（`Vec<String>`）
  - macOS: 使用 `fc-list :spacing=mono` 或 CoreText API 枚举字体
  - 若枚举失败，返回预设列表：`["monospace", "Menlo", "Monaco", "Courier New"]`
- [ ] Settings 字体配置项从文本输入框改为下拉选择框（`<select>`）
- [ ] 下拉选项 = 系统字体列表，当前值高亮显示
- [ ] 下拉框下方保留一个文本输入框，允许手动输入不在列表中的字体名
- [ ] 选择后调用 `config_update` 写入 `terminal.font_family`，xterm 实例实时更新字体
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-049: Settings Theme 预设选项

**Description:** 作为用户，我希望在 Settings 的 Theme 页中有预设主题供一键选择，同时保留自定义颜色能力。

**Acceptance Criteria:**
- [ ] Settings Theme tab 顶部新增预设主题选择区，展示 5-8 个预设主题卡片
- [ ] 每个预设主题卡片显示：主题名称 + 小色块预览（background、foreground、accent 三色）
- [ ] 内置预设主题（至少包含）：
  - **Vibemux Dark**（默认，当前配色）
  - **Solarized Dark**
  - **Monokai**
  - **Nord**
  - **Dracula**
  - **One Dark**
  - **Gruvbox Dark**
  - **Tokyo Night**
- [ ] 点击预设主题卡片，立即将该主题的完整颜色值（含 ANSI 16 色）写入配置并实时生效
- [ ] 选中的预设主题卡片有高亮边框
- [ ] 预设主题下方保留现有的自定义颜色输入区域（用户可在预设基础上微调）
- [ ] 每个预设主题包含完整的 ANSI 16 色定义（black/red/green/yellow/blue/magenta/cyan/white + 8 bright 变体）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-050: 多 Session 间主题色细边框

**Description:** 作为用户，我希望多 session 水平排布时，每个 session 被一圈细主题色边框包裹，视觉上清晰区分各 session 区域。

**Acceptance Criteria:**
- [ ] DeckPane 外层容器添加 `1px solid` 边框，颜色使用该 session 的 `color_token` 对应色（与标题栏拖拽条颜色一致）
- [ ] 边框圆角与现有 DeckPane 圆角一致（或略小，如 `border-radius: 4px`）
- [ ] focused pane 的边框略亮/略粗（如 `2px` 或提高亮度），非 focused pane 边框半透明（`opacity: 0.5` 或降低饱和度）
- [ ] 单个 session 时也显示边框（保持一致性）
- [ ] 边框不影响终端内容区域的实际可用尺寸（使用 `box-sizing: border-box`）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-051: 窗口标题动态显示 Hot Session 信息

**Description:** 作为用户，我希望窗口标题栏显示当前 focused hot session 的名称和状态，而不是固定的 "Vibemux"。

**Acceptance Criteria:**
- [ ] 窗口标题格式：`{session_name} — Vibemux`（focused session 名称 + 破折号 + 应用名）
- [ ] 若 focused session 有 `custom_name`，使用 `custom_name`；否则使用 `name`
- [ ] 若 focused session 处于 Busy 状态（有活动进程），标题格式：`⚙ {session_name} — Vibemux`
- [ ] 无 hot session 时，标题回退为 `Vibemux`
- [ ] 通过 Tauri 的 `window.set_title()` API 在前端调用更新（或通过 `invoke("set_window_title", { title })`）
- [ ] 每次 focused session 切换、session 重命名、session busy 状态变化时更新标题
- [ ] Typecheck passes

---

### US-052: 2.5D 视觉增强

**Description:** 作为用户，我希望整体界面有更强的立体感和层次感，通过加深阴影和边框发光效果实现 2.5D 视觉风格。

**Acceptance Criteria:**
- [ ] DeckPane 容器添加多层 `box-shadow`：
  - 外发光：`0 0 0 1px {color_token}33`（主题色，低透明度）
  - 底部阴影：`0 4px 16px rgba(0,0,0,0.6)`
  - 内侧顶部高光：`inset 0 1px 0 rgba(255,255,255,0.05)`
- [ ] DeckPane 标题栏（drag handle）添加渐变背景，从 `{color_token}22` 到透明，增强色彩层次
- [ ] ShelfCard 添加 `box-shadow: 0 2px 8px rgba(0,0,0,0.4)`，hover 时阴影加深
- [ ] Titlebar 底部分隔线改为渐变线（从主题色到透明），增强层次感
- [ ] 整体背景保持深色，各层级通过阴影和微妙的背景色差异区分
- [ ] 不使用 CSS `perspective` 或 3D transform（避免性能问题和布局复杂性）
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

### US-053: Session Busy 状态旋转图标

**Description:** 作为用户，我希望当 session 中有正在执行的进程时，DeckPane 标题和 ShelfCard 上显示一个旋转的方形图标，直观表示 Busy 状态。

**Acceptance Criteria:**
- [ ] 定义 "Busy" 状态：`ProcessState` 为 `Running`（或前台有活动进程）时为 Busy
- [ ] `SessionSnapshot` 中暴露 `is_busy: bool` 字段（或前端从现有 `process_state` 字段推导）
- [ ] 新建 `BusyIndicator.svelte` 组件：一个小方形（约 10x10px），CSS animation 持续旋转（`rotate 1s linear infinite`）
- [ ] 方形颜色使用 session 的 `color_token` 对应色
- [ ] DeckPane 标题栏：session 名称左侧显示 `BusyIndicator`（仅 Busy 时可见）
- [ ] ShelfCard：卡片右上角或名称旁显示 `BusyIndicator`（仅 Busy 时可见）
- [ ] 非 Busy 时图标完全隐藏（`display: none` 或 `visibility: hidden`），不占位
- [ ] 旋转动画流畅，不影响其他 UI 性能
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

---

## 4. Functional Requirements

### Shelf 高度
- FR-1: Shelf 高度减半，具体值由当前 CSS 定义决定（找到后减半）
- FR-2: Deck 区域使用 flex 自动填充剩余高度

### Rename 持久化
- FR-3: `Session` 结构体新增 `custom_name: Option<String>` 字段
- FR-4: `session_rename` 写入 `custom_name`，PWD 更新不覆盖 `custom_name` 已设置的 `name`
- FR-5: 前端优先显示 `custom_name`，fallback 到 `name`

### 文字 Clip
- FR-6: DeckPane 标题文字使用 `overflow: hidden; white-space: nowrap; text-overflow: clip`
- FR-7: 字体大小固定，不随 pane 宽度缩放

### Prefix Key 设置
- FR-8: Settings 面板新增 Prefix Key 下拉 + 自定义输入
- FR-9: 修改后立即生效，无需重启

### 字体选择
- FR-10: 新增 `list_monospace_fonts` Tauri command
- FR-11: Settings 字体项改为下拉 + 手动输入组合

### Theme 预设
- FR-12: Settings Theme tab 新增 8 个预设主题卡片
- FR-13: 每个预设包含完整 ANSI 16 色定义
- FR-14: 点击预设立即写入配置并实时生效

### Session 边框
- FR-15: DeckPane 使用 session `color_token` 颜色的 1px 边框
- FR-16: focused pane 边框更亮，非 focused 半透明

### 窗口标题
- FR-17: 窗口标题格式 `{name} — Vibemux`，Busy 时前缀 `⚙`
- FR-18: focused session 切换、rename、busy 状态变化时更新

### 2.5D 视觉
- FR-19: DeckPane 多层 box-shadow（外发光 + 底部阴影 + 内侧高光）
- FR-20: ShelfCard hover 阴影加深
- FR-21: 不使用 CSS 3D transform

### Busy 图标
- FR-22: `BusyIndicator.svelte` 旋转方形，颜色跟随 session color_token
- FR-23: DeckPane 标题和 ShelfCard 均集成 BusyIndicator
- FR-24: 非 Busy 时完全隐藏

## 5. Non-Goals (Out of Scope)

- 亮色主题支持（所有预设主题均为深色系）
- 字体预览（下拉中不渲染字体样式）
- 自定义 ANSI 颜色的导入/导出
- Session 进程树详情（只显示 Busy/Idle，不显示具体进程名）
- 动画速度自定义
- Shelf 高度可拖拽调整（本期只做固定减半）

## 6. Design Considerations

### Shelf 高度参考

当前 Shelf 高度约 120px（含 header），减半后约 60px。ShelfCard 需适配：
```
┌─ SHELF ──────────────────────────────────────────────────────┐  60px
│  [●1 name ⚙] [●2 name] [●3 name]                            │
└──────────────────────────────────────────────────────────────┘
```

### Theme 预设卡片布局

```
┌─ Theme ──────────────────────────────────────────────────────┐
│  Presets:                                                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ ■ ■ ■   │ │ ■ ■ ■   │ │ ■ ■ ■   │ │ ■ ■ ■   │        │
│  │Vibemux  │ │Solarized │ │ Monokai  │ │  Nord    │        │
│  │  Dark   │ │  Dark    │ │          │ │          │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │ ■ ■ ■   │ │ ■ ■ ■   │ │ ■ ■ ■   │ │ ■ ■ ■   │        │
│  │ Dracula  │ │ One Dark │ │ Gruvbox  │ │  Tokyo   │        │
│  │          │ │          │ │  Dark    │ │  Night   │        │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘        │
│                                                              │
│  Custom Colors:                                              │
│  Background: [#111111] Foreground: [#d9d4c7] ...            │
└──────────────────────────────────────────────────────────────┘
```

### BusyIndicator 动画

```css
@keyframes spin-square {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.busy-indicator {
  width: 10px;
  height: 10px;
  border: 1.5px solid currentColor;
  animation: spin-square 1s linear infinite;
}
```

### DeckPane 2.5D 阴影示例

```css
.deck-pane {
  box-shadow:
    0 0 0 1px color-mix(in srgb, var(--session-color) 20%, transparent),
    0 4px 16px rgba(0, 0, 0, 0.6),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}
```

## 7. Technical Considerations

### `custom_name` 在 Rust 端

```rust
// models.rs
pub struct Session {
    // ... existing fields
    pub custom_name: Option<String>,
}

// session_manager.rs - rename command
pub fn rename_session(&mut self, id: SessionId, name: String) {
    if let Some(session) = self.sessions.get_mut(&id) {
        session.custom_name = Some(name.clone());
        session.name = name;
    }
}

// PWD update logic - do NOT overwrite name if custom_name is set
if session.custom_name.is_none() {
    session.name = derive_name_from_cwd(&cwd);
}
```

### `list_monospace_fonts` 实现思路

macOS 上可通过 `system_profiler SPFontsDataType` 或 `fc-list :spacing=mono family` 获取字体列表。若命令不可用，返回预设列表：
```rust
vec!["monospace", "Menlo", "Monaco", "Courier New", "JetBrains Mono", "Fira Code", "SF Mono"]
```

### 窗口标题更新

前端通过 `@tauri-apps/api/window` 的 `getCurrentWindow().setTitle(title)` 更新，无需新增 Tauri command。

### 预设主题完整颜色定义

每个预设主题需包含以下字段（与现有 `ThemeConfig` 结构对齐）：
`background`, `foreground`, `cursor`, `selection`, `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`

## 8. Success Metrics

- Shelf 高度减半后，终端内容区域垂直空间增加约 60px
- 重命名 session 后，名称在 PWD 变化后仍保持不变
- 多 session 布局下标题文字无变形，字体大小一致
- Settings 面板可完成所有常用配置（prefix key、字体、主题）
- 界面视觉层次感明显提升，阴影和边框清晰区分各区域

## 9. Open Questions

- `list_monospace_fonts` 在 macOS 上的最佳实现方式（`fc-list` 不一定预装，CoreText 需要 Objective-C 绑定）
- 窗口标题中 Busy 状态的图标是否用 `⚙`（齿轮）还是其他 Unicode 字符（如 `◌` 旋转点）
- 预设主题是否需要支持亮色主题（当前 Non-Goal，但可能有用户需求）
