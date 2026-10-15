# Systemd Manager TUI

A terminal user interface for managing systemd services on Linux.

[English](#english) | [简体中文](#简体中文)

---

<a name="english"></a>
## English

### Features

- Bilingual interface with automatic system locale detection and manual `--lang` flag
- Visual service generator for creating system and user unit files
- User-level service support in `~/.config/systemd/user/` without root or sudo permissions
- Path and environment file auto-completion with Tab key
- Presets for Python, Node.js, Docker Compose, and generic binaries
- Safe removal of custom service files with confirmation dialog
- Japandi theme with natural earth tones and low-contrast selection
- Real-time journal log viewing and unit file inspection
- Full service lifecycle control: start, stop, restart, enable, disable, mask, unmask

### Usage

```bash
systemd-manager-tui

# Specify interface language
systemd-manager-tui --lang zh
systemd-manager-tui --lang en

# Filter on startup
systemd-manager-tui -f nginx

# Disable failure notifications
systemd-manager-tui --no-notifications
```

### Keybindings

| Key | Action |
| --- | --- |
| `↑` / `k`, `↓` / `j` | Navigate service list |
| `←` / `h`, `→` / `l` | Switch between system and user units |
| `/` | Search filter |
| `s` | Start service |
| `x` | Stop service |
| `r` | Restart service |
| `e` | Enable service |
| `d` | Disable service |
| `m` | Toggle mask |
| `n` | Create service |
| `Del` | Delete custom service file |
| `v` | View journal logs |
| `c` | Inspect unit file |
| `f` | Toggle all units vs services |
| `a` | Cycle state filter |
| `u` | Refresh list |
| `?` | Help |
| `Ctrl + c` | Quit |

### Installation

#### Debian / Ubuntu

Download the `.deb` package:

```bash
# ARM64
sudo dpkg -i systemd-manager-tui_1.3.0_arm64-gnu.deb

# x86_64
sudo dpkg -i systemd-manager-tui_1.3.0_amd64.deb
```

#### Build from Source

```bash
cargo build --release
```

---

<a name="简体中文"></a>
## 简体中文

基于终端的 Linux Systemd 服务管理工具。

### 特性

- 原生双语界面，自动识别系统语言，支持 `--lang` 参数切换
- 可视化新建服务向导，直接生成标准单元文件
- 支持用户级服务，写入 `~/.config/systemd/user/`，无需 sudo 或 root 权限
- 路径与环境文件输入支持 Tab 键自动补全
- 提供 Python、Node.js、Docker Compose 与通用二进制快捷模板
- 安全卸载自定义服务文件，带有确认提示
- Japandi 风格配色，低疲劳大地色系与条形高亮指示
- 实时查看 journalctl 日志与单元配置
- 完整的服务生命周期控制：启动、停止、重启、启用、禁用、屏蔽、解除屏蔽

### 使用

```bash
systemd-manager-tui

# 指定界面语言
systemd-manager-tui --lang zh
systemd-manager-tui --lang en

# 启动时过滤服务
systemd-manager-tui -f nginx

# 关闭失败通知
systemd-manager-tui --no-notifications
```

### 快捷键

| 按键 | 功能 |
| --- | --- |
| `↑` / `k`, `↓` / `j` | 移动选择服务 |
| `←` / `h`, `→` / `l` | 切换系统与用户服务 |
| `/` | 搜索过滤 |
| `s` | 启动服务 |
| `x` | 停止服务 |
| `r` | 重启服务 |
| `e` | 启用服务 |
| `d` | 禁用服务 |
| `m` | 屏蔽或解除屏蔽 |
| `n` | 新建服务 |
| `Del` | 删除自定义服务文件 |
| `v` | 查看日志 |
| `c` | 查看配置文件 |
| `f` | 切换单元范围 |
| `a` | 切换状态过滤 |
| `u` | 刷新列表 |
| `?` | 帮助面板 |
| `Ctrl + c` | 退出 |

### 安装

#### Debian / Ubuntu

下载 `.deb` 安装包：

```bash
# ARM64
sudo dpkg -i systemd-manager-tui_1.3.0_arm64-gnu.deb

# x86_64
sudo dpkg -i systemd-manager-tui_1.3.0_amd64.deb
```

#### 源码编译

```bash
cargo build --release
```

---

### 许可证

MIT License
