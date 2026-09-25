# 死代码清理、窗口尺寸持久化、单实例保护与 Slint 侧 API 细节（批次 D）

## 背景 / 范围

批次 D 收尾：死代码清理、README 补齐 Slint 版、macOS ⌘N、窗口尺寸持久化、单实例保护。

## 1. 死代码清理（真删除，只删真死掉的）

语言下拉（`LangMenu`）在自绘顶栏那一轮已被"点击即切换"按钮取代，本轮把它留下的整条链清掉：

| 删除项 | 位置 |
| --- | --- |
| `ui/components/LangMenu.slint`（不再被任何文件引用、也不参与编译） | 整文件 |
| `Theme.lang-menu-w` | `ui/theme.slint` |
| `State.langs` / `State.lang-code` / `State.lang-label` / `struct LangOption` | `ui/state.slint` |
| `export { ..., LangOption }` 重导出 | `ui/main.slint` |
| `Lang::label()`（唯一调用方是已删掉的 state 写入） | `src/i18n.rs` |
| `strings.rs` 里写 `lang-code` / `lang-label` 的两行 + `State` 导入 | `src/strings.rs` |

需要找回时：`git show 55a9073:mnemo-slint/ui/components/LangMenu.slint > mnemo-slint/ui/components/LangMenu.slint`。

**没有删文案**：`viewer.loading`、`theme.toDark`、`theme.toLight`、`toast.copied` 继续留在 `locales/*.json`。
理由：保持与旧版 egui 的 locales 超集关系（迁移/回退零成本），且 `toast.copied` 是需求里明确要求补全的 key；
只清"确定不会再被引用"的代码，不删"以后可能回来"的文案。

## 2. 窗口尺寸持久化：只做尺寸 + 最大化，**不做位置**

- 存：`settings.json` 的 `window` 字段（与 `lang`/`theme` 共用同一份文件，读-改-写保留其它字段）。
- 记：在 winit 钩子的 `WindowEvent::Resized` 里，**仅在非最大化时**记录尺寸
  （最大化状态下 `Window::size()` 返回的是最大化尺寸，存下来会导致下次"还原"得到一个巨型窗口）；
  退出时（`run_event_loop_until_quit` 返回后）统一写盘。
- 恢复：`show()` 之前 `set_size`，`show()` 之后再 `set_maximized`
  （窗口尚未创建时设置最大化在部分平台不生效）。
- **不做位置持久化**：Slint 1.18 没有暴露显示器/工作区信息，无法在恢复前判断保存的位置是否仍在可见屏幕内；
  一旦恢复到屏幕外，无边框窗口 + 失焦隐藏会让应用彻底不可操作（托盘唤回也只是在屏幕外显示）。
  尺寸即使偏大也只会被窗口管理器收进可视区域，风险小得多。
- 尺寸做了范围夹取（`300..=10_000`），挡住损坏的 `settings.json`。

### 编译期踩到的两个 API 细节

| 现象 | 事实 |
| --- | --- |
| `PhysicalSize::new(f32, f32)` 报 `expected u32, found f32` | `slint::PhysicalSize` 的坐标是 **u32**（`i-slint-core/api.rs:184`），不是 f32；`Window::size()/set_size()` 用的都是它 |
| `Timer::start(Duration, cb)` 报 "takes 3 arguments but 2 supplied" | 1.18 的签名是 `Timer::start(TimerMode, Duration, callback)`（`i-slint-core/timers.rs:89`）；单次触发用静态方法 `Timer::single_shot(Duration, cb)`（Toast 用的就是它） |

## 3. 单实例保护（不加依赖）

**方案：回环 TCP + 双向握手**

```text
启动 → bind(127.0.0.1:47831)
 ├─ 成功 = 主实例：后台线程 accept，收到 "mnemo:show" 就置 AtomicBool，并回 "mnemo:ok"
 │            UI 线程用 500ms 的 Timer 轮询该标志 → 调用 show_window()
 └─ 失败 = 尝试连接 + 发送 "mnemo:show"；只有收到 "mnemo:ok" 才认为对方是本应用 → 退出本进程
           （没有 ack 一律放行，继续正常启动）
```

为什么不换别的做法：

| 备选 | 未采用原因 |
| --- | --- |
| 仅靠"端口被占用"判断 | 端口可能被无关进程占用，会误判成"已有实例"从而拒绝启动；因此必须双向握手（fail-open） |
| 锁文件 / 命名互斥体 | 锁文件无法把"请你唤回窗口"这个动作传给已有实例；命名互斥体要引入 `windows`/`winapi` 新依赖，违反"不擅自加依赖" |
| 后台线程直接调用 Slint API | Slint 的组件句柄不是 `Send`，跨线程只能传原子量；UI 线程轮询是唯一安全路径 |

## 4. macOS 的 ⌘N

新增 `KeyBinding { keys: @keys(Meta + N); ... }` 与原有 `@keys(Control + N)` 并存。
KeyBinding 按修饰键匹配，两条互不干扰；Windows 上 `Win+N` 被系统占用，实际不会触发。

## 5. README

两份 README（英文 / 简中）都补齐了 Slint 版信息：实现版本对照表（`mnemo-slint` 当前主用、
`mnemo-egui` 旧版）、运行与构建命令、项目结构、`settings.json` 字段表，
以及三处行为差异（无边框窗口、对笔记按 `Enter` 的语义、主题默认值）。此前两份文档完全没有提到 Slint 版。

## 遗留

- 位置持久化（见 §2）需要显示器诊断能力，Slint 1.18 不可行；
- 单实例保护的端口（47831）若被无关进程占用，则本次运行没有单实例保护（fail-open，符合预期）；
- `mnemo-slint/Cargo.toml` 的 bin 名仍是 `mnemo-slint`（等旧版 egui 下线后改回 `mnemo`）。

## 相关文件

- `mnemo-slint/ui/components/LangMenu.slint`（删除）
- `mnemo-slint/ui/{state,theme}.slint`、`ui/main.slint`（清理 + `Meta + N`）
- `mnemo-slint/src/{i18n,strings,settings,main}.rs`（清理、窗口尺寸持久化、单实例）
- `README.md`、`README.zh-CN.md`（补齐 Slint 版说明）
