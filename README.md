# Systemd Manager TUI (Plus)

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20(x86__64%20%7C%20aarch64)-green?style=flat-square&logo=linux)](https://github.com/LimeBlogs/systemd-manager-tui-plus)
[![Design](https://img.shields.io/badge/Design-Japandi%20Aesthetics-d4a373?style=flat-square)](https://github.com/LimeBlogs/systemd-manager-tui-plus)

**[English](#english) | [简体中文](#简体中文)**

</div>

---

<a name="english"></a>
## English

A modern, aesthetic, and feature-packed Terminal User Interface (TUI) for managing Linux systemd services with ease.

Built with Rust, **Ratatui**, and **zbus** (D-Bus API). Features bilingual support, a visual `.service` generator, user-level service management without sudo, and a calming **Japandi (Zen × Nordic)** design language.

### ✨ Highlights & New Features

- 🌐 **Native Bilingual Support**: Seamless English and Chinese localization, auto-detected from system locale (`LANG`/`LC_ALL`) or selectable via `--lang zh` / `--lang en`.
- 🛠️ **Visual `.service` Generator (Press `n`)**:
  - Interactively build and deploy systemd units in seconds directly inside the TUI.
  - **Dual Scope Support**:
    - **User Services (`--user`)**: Writes directly to `~/.config/systemd/user/`. **Completely free from sudo / root passwords!**
    - **System Services**: Writes to `/etc/systemd/system/` (with sudo/root privileges).
  - **Quick Lazy Presets**: One-key presets for **Python** (`python3 main.py`), **Node.js** (`node index.js`), **Docker Compose** (`docker compose up -d`), and **Generic Binary**.
  - **Smart Path Auto-Completion**: Full `[Tab]` path completion for working directories, exec paths, and `.env` files.
  - **Security & User Isolation**: Quick toggle between root and current `$USER` isolation.
  - **Seamless Combo Flow**: Automatic validation, unit file generation, `systemctl daemon-reload`, enable on boot, immediate start, and instant list jump.
- 🗑️ **Uninstall Custom Services (Press `Del`)**: Safely stop, disable, and clean up custom service unit files with an interactive confirmation dialog.
- 🎨 **Japandi Aesthetic Design System**:
  - Calming, low-fatigue natural earth-tone palette: **Warm Sand** (`#D4A373`), **Sage Moss** (`#7EA088`), **Stone Gray** (`#737A84`), **Terracotta** (`#C96A65`), and **Charcoal Slate** (`#282E36`).
  - Tranquil status glyphs (`● ` running, `◐ ` activating, `○ ` inactive, `✕ ` failed).
  - Elegant vertical wood-rail selection indicator (`▎ `) replacing harsh, eye-burning blue highlight blocks.
  - Exact East Asian Unicode (CJK) display width alignment and pixel-perfect text cursor tracking.
- ⚡ **Full Lifecycle & Monitoring**: Start, stop, restart, mask, unmask, enable, disable, inspect unit files, and view live journal logs (`journalctl`) with auto-scroll.

### 🚀 Quick Start & CLI Usage

```bash
# Launch with system locale auto-detection
systemd-manager-tui

# Specify UI language explicitly (zh: 简体中文, en: English)
systemd-manager-tui --lang zh
systemd-manager-tui --lang en

# Start with a search filter
systemd-manager-tui -f nginx

# Disable desktop notifications for failed units
systemd-manager-tui --no-notifications
```

### ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| `↑` / `k`, `↓` / `j` | Navigate service list |
| `←` / `h`, `→` / `l` | Switch between System and User Session units |
| `/` | Focus search filter input (`Esc` to exit filter) |
| `s` | Start selected service |
| `x` | Stop selected service |
| `r` | Restart selected service |
| `e` | Enable service on boot |
| `d` | Disable service on boot |
| `m` | Toggle Mask / Unmask service |
| **`n`** | **✨ Create new `.service` unit (Visual Generator)** |
| **`Del`** | **🗑️ Uninstall and delete selected custom service file** |
| `v` | View real-time service logs (`journalctl`) |
| `c` | Cat / View service configuration file |
| `f` | Toggle filter: All units vs Services only |
| `a` | Cycle active state filter (active / inactive / failed / all) |
| `u` | Refresh service list |
| `?` | Toggle keyboard shortcuts help popup |
| `Ctrl + c` | Quit application |

### 📦 Installation

#### Ubuntu / Debian (.deb)
Pre-built packages are available for both **AMD64 (x86_64)** and **ARM64 (aarch64)** in `dist/` and [Releases](https://github.com/LimeBlogs/systemd-manager-tui-plus/releases):

```bash
# ARM64 (Ubuntu / Debian / Raspberry Pi / ARM servers)
sudo dpkg -i systemd-manager-tui_1.3.0_arm64-gnu.deb

# AMD64 (x86_64)
sudo dpkg -i systemd-manager-tui_1.3.0_amd64.deb
```

#### Build from Source (Cargo)

```bash
git clone https://github.com/LimeBlogs/systemd-manager-tui-plus.git
cd systemd-manager-tui-plus
cargo build --release
./target/release/systemd-manager-tui
```

---

<a name="简体中文"></a>
## 简体中文

适用于 Linux 系统的现代化高颜值 Systemd 服务管理终端工具（TUI）。

基于 Rust、**Ratatui** 与 **zbus** (D-Bus API) 深度重构。提供开箱即用的原生中英双语支持、可视化 `.service` 服务创建向导、免 sudo 的用户层级服务管理，并融合了极具温润美感的 **Japandi（日式禅宗 × 北欧极简）** 设计语言。

### ✨ 核心亮点与增强特性

- 🌐 **原生中英双语支持**：
  - 界面所有标签、表格列名、快捷键面板、日志说明、错误反馈均已实现完整本地化。
  - 启动时自动读取系统区域设置（`LANG`/`LC_ALL`），也可通过命令行参数 `--lang zh` 或 `--lang en` 显式切换。
- 🛠️ **可视化 `.service` 创建向导（按下快捷键 `n`）**：
  - 在 TUI 内填写几个核心参数即可秒级生成标准 Systemd 服务。
  - **双层级服务隔离（Scope）**：
    - **用户服务（`--user`）**：直接写入 `~/.config/systemd/user/`，**完全免 sudo / root 密码**，普通用户自由掌控！
    - **系统服务（System）**：标准写入 `/etc/systemd/system/`（需要 sudo / root 权限）。
  - **懒人预设模板**：支持 **Python** (`python3 main.py`)、**Node.js** (`node index.js`)、**Docker Compose** (`docker compose up -d`) 及 **通用二进制** 一键套用。
  - **Tab 键路径智能补全**：工作目录、启动程序路径、环境变量文件（`.env`）均支持在输入框内按 `Tab` 智能补全。
  - **运行用户隔离**：自动读取当前 `$USER`，避免本地服务以 root 权限“裸奔”。
  - **开箱无缝连招**：表单确认提交后，后台自动执行配置文件写入、`systemctl daemon-reload`、开机自启配置、立即启动与列表高亮聚焦跳转。
- 🗑️ **一键卸载自定义服务（按下快捷键 `Del`）**：
  - 选中自定义服务后按 `Del`，提供优雅安全的确认弹窗，一键停止、取消自启并彻底移除其 `.service` 配置文件。
- 🎨 **Japandi 视觉设计系统（禅意 × 极简）**：
  - **大地温润色系**：告别瞎眼的纯蓝高亮底色和荧光青框，采用温润的**暖木沙色**（`#D4A373`）、**苔绿色**（`#7EA088`）、**浅灰石**（`#737A84`）、**陶土红**（`#C96A65`）与**深炭石灰底**（`#282E36`）。
  - **极简状态点阵徽标**：`● ` 运行中、`◐ ` 启动中、`○ ` 已停止、`✕ ` 运行异常，扫一眼即知系统健康状况。
  - **左侧垂直木轨指示条（`▎ `）**：取代传统突兀的双箭头 `>> `，视觉宁静低疲劳。
  - **东亚宽字符（CJK）光标完美对齐**：彻底修复中文字符在终端中因 Unicode 显示列宽差异导致的标签参差不齐与光标错位漂移问题。
- ⚡ **完备的生命周期管控**：支持启动、停止、重启、屏蔽（Mask）、解除屏蔽、开机自启切换、实时日志流跟踪（`journalctl`）与配置文件预览。

### 🚀 命令行启动参数

```bash
# 默认启动（自动识别系统语言）
systemd-manager-tui

# 显式指定语言为中文
systemd-manager-tui --lang zh

# 启动并预填过滤搜索词
systemd-manager-tui -f docker

# 关闭服务失败时的桌面系统通知
systemd-manager-tui --no-notifications
```

### ⌨️ 操作快捷键清单

| 按键 | 功能说明 |
| :--- | :--- |
| `↑` / `k`, `↓` / `j` | 向上 / 向下移动光标选择服务 |
| `←` / `h`, `→` / `l` | 切换系统服务 (System) 与用户会话服务 (Session) |
| `/` | 聚焦搜索过滤输入框（按 `Esc` 退出搜索模式） |
| `s` | 启动当前选中的服务 (Start) |
| `x` | 停止当前选中的服务 (Stop) |
| `r` | 重启当前选中的服务 (Restart) |
| `e` | 启用服务开机自启 (Enable) |
| `d` | 禁用服务开机自启 (Disable) |
| `m` | 屏蔽 / 解除屏蔽服务 (Mask/Unmask) |
| **`n`** | **✨ 新建 Systemd 服务单元（可视化表单创建向导）** |
| **`Del`** | **🗑️ 卸载并删除自定义服务配置文件（带确认弹窗）** |
| `v` | 查看服务的实时日志（`journalctl -u ...`） |
| `c` | 查看服务的配置文件内容（`systemctl cat ...`） |
| `f` | 切换显示范围（所有单元 / 仅服务服务单元） |
| `a` | 循环切换状态过滤（全部 / 运行中 / 已停止 / 启动失败） |
| `u` | 刷新读取最新的服务列表 |
| `?` | 弹出 / 关闭键盘快捷键帮助指南 |
| `Ctrl + c` | 安全退出程序 |

### 📦 安装方式

#### Ubuntu / Debian (.deb 安装包)
在项目的 `dist/` 目录或 [GitHub Releases](https://github.com/LimeBlogs/systemd-manager-tui-plus/releases) 中已提供现成的编译包（支持 **x86_64** 与 **ARM64**）：

```bash
# ARM64 架构（适用于 Ubuntu ARM64、Debian、树莓派等）
sudo dpkg -i systemd-manager-tui_1.3.0_arm64-gnu.deb

# x86_64 架构
sudo dpkg -i systemd-manager-tui_1.3.0_amd64.deb
```

#### 源码编译安装 (Cargo)

```bash
git clone https://github.com/LimeBlogs/systemd-manager-tui-plus.git
cd systemd-manager-tui-plus
cargo build --release
./target/release/systemd-manager-tui
```

---

## 📝 开源协议 (License)

本项目采用 [MIT 许可证](LICENSE) 进行开源。
