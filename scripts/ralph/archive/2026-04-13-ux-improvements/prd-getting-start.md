# Vibemux MVP 开发导向 PRD                                                                                                                                                    
  - 产品名称：Vibemux                                                                                                         
  - 文档版本：v0.2（开发导向版）                                                                                              
  - 技术栈：Tauri 2 + Rust + Svelte 5 + xterm.js                                                                              
  - MVP 范围：Milestone 1–5                                                                                                   
  - 文档状态：待实现                                                                                                                                                                                                                                   
## 1. 已确认的开放问题决策                                                                                                     
┌──────────────────────────────────┬────────────────────────────────────────┐                                               
  │               问题               │                  决策                  │                                               
  ├──────────────────────────────────┼────────────────────────────────────────┤                                               
  │ Recall 后进入 focused 还是原位置 │ 进入 focused                           │                                               
  ├──────────────────────────────────┼────────────────────────────────────────┤                                               
  │ Shelf 位置                       │ 底部固定                               │                                               
  ├──────────────────────────────────┼────────────────────────────────────────┤                                               
  │ 默认 max hot sessions            │ 6                                      │
  ├──────────────────────────────────┼────────────────────────────────────────┤
  │ Warm session replay buffer       │ 10,000 行 / 20MB，超限丢弃最旧         │
  ├──────────────────────────────────┼────────────────────────────────────────┤
  │ Archive log                      │ MVP 不开启，session 关闭后只保留元数据 │                                               
  ├──────────────────────────────────┼────────────────────────────────────────┤
  │ Park alternate-screen app        │ 不提示，直接 Park                      │                                               
  ├──────────────────────────────────┼────────────────────────────────────────┤               
  │ Session color 分配               │ 系统从预设色板自动分配，用户可手动修改 │                                               
  ├──────────────────────────────────┼────────────────────────────────────────┤               
  │ Navigation Mode prefix           │ 默认 Ctrl+Space，macOS 上为 Cmd+Space  │
  └──────────────────────────────────┴────────────────────────────────────────┘
                                                                                                                              
  ---                                    
  2. 核心概念速查                                                                                                             
                                                                                              
  Hot Session   → 在 Deck 中，xterm 实例常驻，可输入，瞬间切换                                                                
  Warm Session  → 在 Shelf 中，PTY 继续运行，xterm 实例销毁，Recall 时 replay
  Cold Session  → 已退出，只保留元数据（MVP 不保留日志）                                                                      
                                                                                              
  Deck          → 水平排布的 hot session 区域，最多 6 个
  Shelf         → 底部任务卡区域，显示所有 warm session                                                                       
                                         
  ---                                                                                                                         
  3. 数据模型                                                                                 
                                                                                                                              
  3.1 Session（Rust）
                                                                                                                              
  pub struct Session {                                                                        
      pub id: SessionId,           // UUID
      pub name: String,                  
      pub cwd: PathBuf,           
      pub command: SessionCommand,
      pub color: ColorToken,       // 预设色板中的一个 token                                                                  
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
                                                                                                                              
  pub enum SessionCommand {                                                                   
      Shell { shell: String },                                                                                                
      Command { program: String, args: Vec<String> },
  }                                                                                                                           
                                                                                              
  pub enum ThermalState { Hot, Warm, Cold }

  pub enum ProcessState {                
      Starting,                   
      Running,
      Exited { code: Option<i32> },                                                                                           
      FailedToStart { message: String },
      Killed,                                                                                                                 
  }                                                                                           
                                         
  pub enum AttentionState {
      Normal,
      Active,      // warm session 有新输出
      NeedsInput,  // 输出匹配 needs_input 规则
      Failed,      // 非 0 exit 或输出匹配 failed 规则
      Done,        // 0 exit
  }                                      
                                  
  3.2 Workspace（Rust）
                                                                                                                              
  pub struct Workspace {
      pub id: WorkspaceId,                                                                                                    
      pub name: String,                                                                       
      pub hot_session_ids: Vec<SessionId>,   // 顺序即 Deck 顺序
      pub warm_session_ids: Vec<SessionId>,
      pub focused_session_id: Option<SessionId>,
      pub layout: LayoutKind,                // MVP 只有 Deck
  }                                                                                                                           
  
  3.3 OutputRingBuffer（Rust）                                                                                                
                                                                                              
  pub struct OutputRingBuffer {          
      // 最多保留 10,000 行 / 20MB，超限丢弃最旧
      // 每条记录带单调递增 seq          
      pub max_lines: usize,   // 默认 10_000
      pub max_bytes: usize,   // 默认 20 * 1024 * 1024
  }                                                                                                                           
  
  3.4 ColorToken                                                                                                              
                                                                                              
  预设 8 个颜色 token，系统按创建顺序循环分配：

  red | orange | yellow | green | cyan | blue | purple | pink

  ---
  4. Tauri Command / Event 协议
                                         
  4.1 前端 → 后端 Commands        

  // session 管理                                                                                                             
  invoke('session_create', { payload: CreateSessionPayload })
  invoke('session_write', { sessionId: string, data: string })                                                                
  invoke('session_resize', { sessionId: string, cols: number, rows: number })                 
  invoke('session_focus', { sessionId: string })
  invoke('session_park', { sessionId: string })
  invoke('session_recall', { sessionId: string })  // 总是进入 focused
  invoke('session_rename', { sessionId: string, name: string })                                                               
  invoke('session_close', { sessionId: string })   // 优雅关闭
  invoke('session_kill', { sessionId: string })    // 强制 kill                                                               
  invoke('session_set_color', { sessionId: string, color: ColorToken })                       
                                         
  // workspace
  invoke('workspace_get_snapshot')  // 返回当前 workspace 完整快照

  // config
  invoke('config_get')
  invoke('config_update', { patch: Partial<UserConfig> })

  type CreateSessionPayload = {
    name: string
    cwd: string
    command: { type: 'shell'; shell?: string }
             | { type: 'command'; program: string; args: string[] }
    color?: ColorToken  // 不传则自动分配
  }
                                                                                                                              
  4.2 后端 → 前端 Events
                                                                                                                              
  type MuxEvent =                                                                             
    | { type: 'session.created';       session: SessionSnapshot }
    | { type: 'session.output';        sessionId: string; data: string; seq: number }
    | { type: 'session.replay.start';  sessionId: string; fromSeq: number; toSeq: number }
    | { type: 'session.replay.chunk';  sessionId: string; data: string; seqStart: number; seqEnd: number }
    | { type: 'session.replay.end';    sessionId: string; toSeq: number }
    | { type: 'session.exited';        sessionId: string; code: number | null }
    | { type: 'session.updated';       session: SessionSnapshot }
    | { type: 'attention.changed';     sessionId: string; state: AttentionState }
    | { type: 'workspace.snapshot';    workspace: WorkspaceSnapshot }
    | { type: 'error';                 message: string; commandId?: string }

  type SessionSnapshot = {
    id: string
    name: string
    cwd: string
    color: ColorToken
    thermalState: 'hot' | 'warm' | 'cold'
    processState: ProcessState
    attentionState: AttentionState
    terminalTitle: string | null
    lastActivityAt: string | null  // ISO 8601
    lastOutputSummary: string | null  // 最近一行输出，用于 Shelf card
  }

  4.3 seq 顺序要求

  - 同一 session 的 session.output 事件 seq 必须单调递增
  - replay 必须明确 fromSeq / toSeq 范围
  - recall 时：replay end 到达前，不向前端写入新的 live output
  - 前端检测到 seq gap 时，调用 workspace_get_snapshot 重新同步
                                  
  ---
  5. 功能需求                                                                                                                 
  
  FR-1：创建 Session                                                                                                          
                                                                                              
  - FR-1.1：用户通过 New Session 面板创建 session，必填字段：名称、工作目录、启动类型
  - FR-1.2：启动类型 MVP 支持：Shell（使用系统默认 shell）、Custom Command（program + args）
  - FR-1.3：cwd 不存在时，命令返回错误，前端显示具体路径和原因
  - FR-1.4：shell 不存在时，命令返回错误，前端显示 shell 路径和原因
  - FR-1.5：color 不传时，系统从预设 8 色按创建顺序循环分配
  - FR-1.6：创建成功后，session 默认进入 Deck 并成为 focused session
  - FR-1.7：Deck 已有 6 个 hot session 时，新建 session 仍可创建，但自动进入 Shelf（warm 状态）

  FR-2：Deck 展示与切换

  - FR-2.1：Deck 水平排布所有 hot session，最多同时展示 6 个
  - FR-2.2：focused session 占据最大宽度，其余为 peripheral（压缩展示）
  - FR-2.3：每个 pane 有 2.5D 彩色边框，颜色与 session color 绑定
  - FR-2.4：focused pane 边框更亮，peripheral pane 透明度降低（默认 0.8）
  - FR-2.5：hot session 切换不销毁 xterm 实例，不重建 PTY，不 replay，切换感知延迟 < 50ms
  - FR-2.6：切换时有短动画（默认 150ms），动画期间键盘输入不丢失                                                              
  - FR-2.7：鼠标点击 peripheral pane 可聚焦
  - FR-2.8：支持拖拽重排 hot session 顺序                                                                                     
                                                                                              
  FR-3：Park Session                     

  - FR-3.1：用户触发 Park，session 从 Deck 移除，进入 Shelf
  - FR-3.2：Park 不 kill PTY，进程继续运行
  - FR-3.3：前端销毁该 session 的 xterm 实例
  - FR-3.4：Rust mux core 继续持有 PTY，输出写入 ring buffer
  - FR-3.5：Deck 重新布局，剩余 session 重新分配宽度
  - FR-3.6：Park alternate-screen app（vim/top 等）不弹提示，直接 Park
                                  
  FR-4：Shelf 展示
                                                                                                                              
  - FR-4.1：Shelf 固定在窗口底部，始终可见
  - FR-4.2：每个 warm session 显示为任务卡，包含：颜色标识、名称、状态 badge、工作目录、最近一行输出、最近活跃时间            
  - FR-4.3：状态 badge 对应 AttentionState：normal（无                                        
  badge）、active（蓝点）、needs_input（黄色"需要输入"）、failed（红色"失败"）、done（灰色"完成"）
  - FR-4.4：warm session 有新输出时，任务卡实时更新最近输出和活跃时间
  - FR-4.5：warm session 退出后，任务卡更新为 done 或 failed 状态
                                  
  FR-5：Recall Session
                                                                                                                              
  - FR-5.1：用户选中 Shelf card 并触发 Recall
  - FR-5.2：session 状态从 Warm 变为 Hot                                                                                      
  - FR-5.3：前端创建新的 xterm.js 实例                                                        
  - FR-5.4：Rust 发送 replay start → replay chunks → replay end
  - FR-5.5：前端分帧写入 xterm（每帧写入量受控，不一次性写入大文本）
  - FR-5.6：replay 超过 200ms 时，xterm 上方显示轻量"正在恢复..."状态
  - FR-5.7：replay end 到达后，进入 live attach 状态，后续输出实时写入
  - FR-5.8：Recall 后 session 成为 focused session
  - FR-5.9：Recall 不重启进程，PTY 是同一个活进程

  FR-6：键盘导航                                                                                                              
  
  Navigation Mode 通过 prefix key 激活：                                                                                      
  - macOS：Cmd+Space                                                                          
  - 其他平台：Ctrl+Space                 
                        
  激活后进入 Navigation Mode，再次按 prefix 或 Esc 退出。
                                         
  Navigation Mode 键位：          

  H / ←        聚焦上一个 Hot Session                                                                                         
  L / →        聚焦下一个 Hot Session
  J / ↓        选择下一个 Shelf Card                                                                                          
  K / ↑        选择上一个 Shelf Card                                                          
  Enter        Recall 当前选中的 Shelf Card
  B            Park 当前 focused session
  N            打开 New Session 面板
  R            重命名当前 focused session（inline 编辑）
  X            关闭当前 focused session（优雅退出）
  Shift+X      Kill 当前 focused session（强制）
  /            打开 Session 搜索
  ?            打开快捷键帮助

  Insert Mode（默认）：所有按键直接进入当前 focused terminal。

  FR-7：Session 搜索

  - FR-7.1：搜索范围：session 名称、cwd、最近输出摘要、状态
  - FR-7.2：搜索结果支持键盘上下选择
  - FR-7.3：选中结果按 Enter：若为 hot session 则聚焦，若为 warm session 则 Recall

  FR-8：Session 管理操作

  - FR-8.1：重命名：inline 编辑，Enter 确认，Esc 取消
  - FR-8.2：关闭（close）：向进程发送 SIGHUP / EOF，等待退出，超时后提示用户是否 kill
  - FR-8.3：Kill：立即 SIGKILL，session 进入 Cold 状态
  - FR-8.4：进程退出检测：自动更新 ProcessState 和 AttentionState                                                             
  - FR-8.5：修改颜色：通过颜色选择器从预设 8 色中选择
                                                                                                                              
  FR-9：终端能力                                                                              
                                         
  - FR-9.1：PTY 输入输出（portable-pty）
  - FR-9.2：xterm.js 渲染，支持基础 ANSI 16 色
  - FR-9.3：终端 resize：窗口 resize 或 pane 宽度变化时，通知 PTY 更新 cols/rows
  - FR-9.4：scrollback（默认 10,000 行）
  - FR-9.5：复制（鼠标选中 + Cmd/Ctrl+C）
  - FR-9.6：粘贴（Cmd/Ctrl+V）
  - FR-9.7：右键菜单：复制、粘贴、清屏
  - FR-9.8：链接识别与点击打开（使用平台安全 opener）
  - FR-9.9：terminal title 更新（OSC 0/2）

  FR-10：配置

  - FR-10.1：配置文件路径按平台约定：
    - macOS：~/Library/Application Support/vibemux/config.toml
    - Linux：~/.config/vibemux/config.toml
    - Windows：%APPDATA%\vibemux\config.toml
  - FR-10.2：GUI 设置面板支持修改所有配置项
  - FR-10.3：配置写入必须原子化（写临时文件 → rename）                                                                        
  - FR-10.4：配置文件损坏或字段无法识别时，使用默认值启动，不崩溃，显示 warning
  - FR-10.5：重启后配置保留                                                                                                   
                                                                                              
  配置项（含默认值）：                   

  [terminal]
  font_family = "monospace"
  font_size = 13
  line_height = 1.2
  scrollback_lines = 10000
  replay_buffer_lines = 10000
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
  default = "system"   # 使用系统默认 shell

  [keys]
  # macOS 自动使用 Cmd+Space，其他平台使用 Ctrl+Space
  prefix = "auto"

  FR-11：Attention State 检测规则

  默认规则（按优先级）：

  1. 进程退出，exit code = 0          → Done
  2. 进程退出，exit code ≠ 0          → Failed
  3. 进程被 kill                      → Failed
  4. 最近输出匹配（大小写不敏感）：
     - "error" | "failed" | "panic" | "fatal" | "exception"  → Failed
     - "continue?" | "do you want" | "press enter" | "y/n"   → NeedsInput
     - "[y/N]" | "[Y/n]" | "(yes/no)"                        → NeedsInput
  5. warm session 收到新输出           → Active
  6. 其他                              → Normal

  规则匹配在 Rust 侧执行，匹配后发送 attention.changed event。

  ---
  6. 非功能需求
                                         
  性能                            

  ┌───────────────────────────────┬─────────────────────────────────┐                                                         
  │             指标              │              要求               │
  ├───────────────────────────────┼─────────────────────────────────┤                                                         
  │ 应用窗口可见                  │ < 1.5s                          │                         
  ├───────────────────────────────┼─────────────────────────────────┤
  │ 默认 shell 可交互             │ < 2s（不含 shell profile 耗时） │
  ├───────────────────────────────┼─────────────────────────────────┤
  │ Hot session 切换感知延迟      │ < 50ms                          │
  ├───────────────────────────────┼─────────────────────────────────┤
  │ 6 个 hot session 切换动画     │ 流畅（60fps）                   │
  ├───────────────────────────────┼─────────────────────────────────┤
  │ 高频输出（yes 命令）时 UI     │ 不冻结                          │
  ├───────────────────────────────┼─────────────────────────────────┤
  │ Rust 侧 output batch interval │ 8–16ms                          │
  ├───────────────────────────────┼─────────────────────────────────┤
  │ Replay > 200ms 时             │ 显示状态提示                    │
  └───────────────────────────────┴─────────────────────────────────┘

  内存

  - Hot session：xterm 实例常驻，scrollback 默认 10,000 行
  - Warm session：不保留 xterm 实例，ring buffer 上限 10,000 行 / 20MB
  - Archive：MVP 不保留日志，只保留元数据

  安全

  - 前端不直接访问 PTY、文件系统、shell
  - 所有 Tauri command payload 必须类型校验
  - Kill 进程操作需要用户二次确认（除非用户在设置中关闭确认）
  - 链接打开使用平台安全 opener（tauri::api::shell::open）
  - 配置写入原子化

  ---
  7. 错误处理要求

  每个错误提示必须包含：哪个 session 出错、出错原因、进程是否还活着、用户可采取的操作。

  ┌──────────────────────────┬─────────────────────────────────────────────────────┐
  │         错误场景         │                      处理方式                       │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ PTY 创建失败             │ 显示错误，session 进入 FailedToStart 状态           │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ shell 不存在             │ 显示 shell 路径，提示用户检查配置                   │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ cwd 不存在               │ 显示路径，提示用户修改                              │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ 配置文件损坏             │ 使用默认值启动，显示 warning banner                 │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ replay buffer 不可用     │ Recall 时显示"无法恢复历史输出"，但仍接入 live 输出 │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ session 在 recall 中退出 │ 停止 replay，显示退出状态                           │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ WebGL 渲染失败           │ xterm.js fallback 到 canvas renderer                │
  ├──────────────────────────┼─────────────────────────────────────────────────────┤
  │ Tauri command 超时       │ 显示错误，不重试                                    │
  └──────────────────────────┴─────────────────────────────────────────────────────┘

  ---
  8. 用户故事

  US-001：创建第一个 Shell Session

  描述： 作为开发者，我想通过 GUI 创建一个 shell session，以便开始在 Vibemux 中工作。

  验收标准：                             
  - 点击 New Session 按钮（或按 prefix + N）打开创建面板
  - 面板包含：名称输入框、工作目录选择、启动类型选择（Shell / Command）
  - 点击确认后，session 出现在 Deck 中并成为 focused                                                                          
  - 可以在 terminal 中输入命令并看到输出
  - session 有彩色边框，颜色自动分配                                                                                          
  - cwd 不存在时，显示具体错误信息，不创建 session                                            
  - Typecheck/lint passes                
  - 在浏览器中验证（dev-browser skill）

  US-002：在多个 Hot Session 间切换

  描述： 作为开发者，我想在 Deck 中的多个 session 间快速切换，不丢失任何 session 的状态。

  验收标准：
  - 创建 3 个 session，均显示在 Deck 中
  - 按 prefix + H/L 可切换 focused session
  - 切换后 terminal 内容、scrollback、光标位置完整保留
  - 切换感知延迟 < 50ms（无明显卡顿）
  - 切换动画流畅（150ms）
  - 鼠标点击 peripheral pane 可聚焦
  - Typecheck/lint passes
  - 在浏览器中验证（dev-browser skill）

  US-003：Park Session 到 Shelf

  描述： 作为开发者，我想把当前 session 放到后台，让进程继续运行，同时腾出 Deck 空间。

  验收标准：
  - 按 prefix + B，当前 focused session 从 Deck 移除
  - Shelf 底部出现该 session 的任务卡               
  - 任务卡显示：颜色、名称、状态、cwd、最近输出
  - Park 后进程继续运行（在 session 中运行 sleep 60，Park 后进程仍存在）
  - Deck 重新布局，剩余 session 重新分配宽度                            
  - Typecheck/lint passes                   
  - 在浏览器中验证（dev-browser skill）
                                         
  US-004：Recall Warm Session     

  描述： 作为开发者，我想把 Shelf 中的 session 拉回 Deck，并看到它在后台产生的输出。                                          
  
  验收标准：                                                                                                                  
  - 按 prefix + J/K 选中 Shelf card，按 Enter 触发 Recall                                     
  - session 出现在 Deck 中并成为 focused 
  - 显示"正在恢复..."状态（如果 replay > 200ms）
  - replay 完成后，后台产生的输出完整显示在 terminal 中
  - replay 完成后，新输出实时写入
  - Recall 不重启进程（PID 不变）
  - Typecheck/lint passes
  - 在浏览器中验证（dev-browser skill）

  US-005：Shelf 任务卡状态更新

  描述： 作为开发者，我想在 Shelf 中实时看到后台 session 的状态变化，不需要 Recall 就能判断任务是否完成或失败。

  验收标准：
  - warm session 有新输出时，任务卡最近输出实时更新
  - 进程正常退出（exit 0），任务卡显示"完成"badge
  - 进程异常退出（exit ≠ 0），任务卡显示"失败"badge（红色）
  - 输出包含 "y/n" 等交互提示，任务卡显示"需要输入"badge（黄色）
  - Typecheck/lint passes
  - 在浏览器中验证（dev-browser skill）

  US-006：配置字体和主题

  描述： 作为开发者，我想自定义终端字体和颜色主题，并在重启后保留设置。

  验收标准：
  - 设置面板可修改字体族、字号、行高
  - 设置面板可修改终端背景色、前景色、ANSI 16 色
  - 修改后立即生效（不需要重启）                
  - 重启应用后设置保留          
  - 配置文件损坏时，使用默认值启动，显示 warning
  - Typecheck/lint passes                       
  - 在浏览器中验证（dev-browser skill）
                                       
  US-007：Session 搜索
                                         
  描述： 作为开发者，我想快速搜索并跳转到任意 session，无论它在 Deck 还是 Shelf 中。

  验收标准：                                                                                                                  
  - 按 prefix + / 打开搜索框
  - 输入关键词，实时过滤 session 名称、cwd、最近输出                                                                          
  - 键盘上下选择结果                                                                          
  - 选中 hot session 按 Enter，聚焦该 session
  - 选中 warm session 按 Enter，触发 Recall
  - Typecheck/lint passes
  - 在浏览器中验证（dev-browser skill）

  ---
  9. 里程碑与验收

  Milestone 1：基础骨架

  交付：
  - Tauri 2 应用可启动
  - Rust mux-core crate（Session 模型、PTY 管理）
  - Rust pty-host crate（portable-pty 封装）
  - Tauri command bridge（session_create / session_write / session_resize）
  - 前端单 session：xterm.js 渲染、输入、输出、resize

  验收：
  - GUI 中可打开一个 shell
  - 可输入命令并看到输出
  - resize 窗口后 terminal 正常

  Milestone 2：Deck 体验

  交付：
  - 多 hot session 支持
  - 水平 Deck 布局（focused + peripheral）
  - 2.5D 彩色 pane 边框
  - Navigation Mode（prefix key + H/L 切换）
  - 鼠标点击聚焦
  - 拖拽重排

  验收：
  - 3 个 session 间切换不重建 terminal
  - 切换感知延迟 < 50ms
  - 动画流畅

  Milestone 3：Shelf 和 Thermal Model

  交付：
  - Hot/Warm/Cold 状态机
  - Park（session → Shelf）
  - Shelf card UI（颜色、名称、状态、最近输出）
  - Warm ring buffer（10,000 行 / 20MB）
  - Recall + replay（分帧写入）
  - Attention state 检测

  验收：
  - Park 不 kill 进程
  - Warm session 继续输出，任务卡实时更新
  - Recall 不重启进程
  - Replay 输出顺序正确
  - Attention badge 正确显示

  Milestone 4：配置和终端能力

  交付：
  - 字体 / 主题设置面板
  - 配置持久化（原子写入）               
  - copy / paste                  
  - 右键菜单
  - 链接识别与打开                                                                                                            
  - Session 搜索
                                                                                                                              
  验收：                                                                                      
  - 重启后设置保留                       
  - copy/paste/right-click 可用
  - 搜索可跳转到 hot/warm session

  Milestone 5：跨平台与发布准备

  交付：
  - macOS（Apple Silicon + Intel）打包
  - Linux（deb + AppImage）打包
  - Windows（ConPTY）打包
  - E2E 测试覆盖主流程
  - 性能测试（yes 高频输出、6 hot + 20 warm）
  - 已知限制文档

  验收：
  - 三大平台可运行
  - MVP 主流程（US-001 到 US-007）全部通过
  - 已知限制记录清楚
                                         
  ---                             
  10. 非目标（MVP 不做）
                                                                                                                              
  - 独立 mux daemon / 多窗口 attach
  - 多 workspace                                                                                                              
  - SSH / WSL / Docker session                                                                
  - Archive log 搜索                     
  - Agent 专用检测器（Claude Code / Codex detector）
  - 插件系统
  - 完美 alternate-screen snapshot（vim/top Park 后 Recall 不保证画面完整还原）
  - 云端同步
  - 代码编辑器 / IDE 功能


## 11. 项目结构

    ```text
    vibemux/
    apps/desktop/
      package.json
      src/
        main.ts
        App.svelte
        terminal/
          TerminalPane.svelte      # xterm.js 实例管理
          xtermLifecycle.ts        # 创建/销毁 xterm 实例
          xtermTheme.ts            # 主题映射
          terminalReplay.ts        # replay 分帧写入队列
          terminalResize.ts        # resize 通知
        deck/
          Deck.svelte              # 水平布局容器
          DeckPane.svelte          # 单个 pane（含彩色边框）
          deckLayout.ts            # focused/peripheral 宽度计算
    ```
  