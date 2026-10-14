use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

use crate::domain::service_unit::{RestartPolicy, ServicePresetTemplate, ServiceScope, ServiceUnitConfig};
use crate::terminal::components::form_fields::{PathField, SelectorField, TextField, ToggleField};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateServiceResult {
    Editing,
    Cancelled,
    Submitted(ServiceUnitConfig),
}

pub struct CreateServiceForm {
    pub current_field: usize,
    pub scope_selector: SelectorField<ServiceScope>,
    pub template_selector: SelectorField<ServicePresetTemplate>,
    pub service_name: TextField,
    pub description: TextField,
    pub working_dir: PathField,
    pub exec_start: PathField,
    pub env_file: PathField,
    pub run_user: SelectorField<String>,
    pub restart_policy: SelectorField<RestartPolicy>,
    pub enable_on_boot: ToggleField,
    pub start_immediately: ToggleField,
    pub error: Option<String>,
    pub is_chinese: bool,
}

impl Default for CreateServiceForm {
    fn default() -> Self {
        Self::new(true, ServiceScope::User)
    }
}

impl CreateServiceForm {
    pub fn new(is_chinese: bool, default_scope: ServiceScope) -> Self {
        let scope_options = vec![
            (ServiceScope::User, ServiceScope::User.label(is_chinese).to_string()),
            (ServiceScope::System, ServiceScope::System.label(is_chinese).to_string()),
        ];

        let mut scope_selector = SelectorField::new(
            if is_chinese { "服务层级" } else { "Service Scope" },
            scope_options,
        );
        scope_selector.set_selected_value(&default_scope);

        let templates = vec![
            (ServicePresetTemplate::Custom, if is_chinese { "自定义 (Custom)" } else { "Custom" }.to_string()),
            (ServicePresetTemplate::Python, if is_chinese { "Python 脚本 (Python Script)" } else { "Python Script" }.to_string()),
            (ServicePresetTemplate::Node, if is_chinese { "Node.js 服务 (Node.js Service)" } else { "Node.js Service" }.to_string()),
            (ServicePresetTemplate::DockerCompose, if is_chinese { "Docker Compose" } else { "Docker Compose" }.to_string()),
            (ServicePresetTemplate::Binary, if is_chinese { "通用二进制 (Generic Binary)" } else { "Generic Binary" }.to_string()),
        ];

        let mut user_options = vec![("root".to_string(), "root (Superuser)".to_string())];
        let detected_user = std::env::var("SUDO_USER")
            .ok()
            .filter(|u| u != "root" && !u.is_empty())
            .or_else(|| {
                std::env::var("USER")
                    .ok()
                    .filter(|u| u != "root" && !u.is_empty())
            })
            .or_else(|| {
                std::env::var("LOGNAME")
                    .ok()
                    .filter(|u| u != "root" && !u.is_empty())
            });

        if let Some(user) = detected_user {
            user_options.push((user.clone(), format!("{} (Regular User / 普通用户)", user)));
        }

        let restart_options = vec![
            (RestartPolicy::OnFailure, "on-failure (Recommended / 推荐)".to_string()),
            (RestartPolicy::Always, "always (总是重启)".to_string()),
            (RestartPolicy::No, "no (不重启)".to_string()),
            (RestartPolicy::OnSuccess, "on-success (成功后重启)".to_string()),
            (RestartPolicy::OnAbnormal, "on-abnormal (异常终止重启)".to_string()),
        ];

        Self {
            current_field: 0,
            scope_selector,
            template_selector: SelectorField::new(
                if is_chinese { "预设模板" } else { "Preset Template" },
                templates,
            ),
            service_name: TextField::new(
                if is_chinese { "服务名称" } else { "Service Name" },
                "e.g. my-app / my-service",
            ),
            description: TextField::new(
                if is_chinese { "服务描述" } else { "Description" },
                "e.g. My Application Service",
            ),
            working_dir: PathField::new(
                if is_chinese { "工作目录" } else { "Working Dir" },
                "e.g. /opt/my-app",
            ),
            exec_start: PathField::new(
                if is_chinese { "启动命令" } else { "ExecStart" },
                "e.g. /usr/bin/python3 main.py",
            ),
            env_file: PathField::new(
                if is_chinese { "环境变量文件 (.env)" } else { "Environment (.env)" },
                "e.g. /opt/my-app/.env (optional)",
            ),
            run_user: SelectorField::new(
                if is_chinese { "运行用户" } else { "Run User" },
                user_options,
            ),
            restart_policy: SelectorField::new(
                if is_chinese { "重启策略" } else { "Restart Policy" },
                restart_options,
            ),
            enable_on_boot: ToggleField::new(
                if is_chinese { "开机自启" } else { "Enable on Boot" },
                true,
            ),
            start_immediately: ToggleField::new(
                if is_chinese { "立即启动" } else { "Start Immediately" },
                true,
            ),
            error: None,
            is_chinese,
        }
    }

    pub fn set_chinese(&mut self, is_chinese: bool) {
        self.is_chinese = is_chinese;
    }

    pub fn apply_template(&mut self, template: ServicePresetTemplate) {
        match template {
            ServicePresetTemplate::Python => {
                let py = ServiceUnitConfig::detect_binary("python3")
                    .or_else(|| ServiceUnitConfig::detect_binary("python"))
                    .unwrap_or_else(|| "/usr/bin/python3".into());
                self.exec_start.set_value(format!("{py} main.py"));
                self.restart_policy.set_selected_value(&RestartPolicy::OnFailure);
                if self.description.value().is_empty() {
                    self.description.set_value("Python Service");
                }
            }
            ServicePresetTemplate::Node => {
                let node = ServiceUnitConfig::detect_binary("node")
                    .unwrap_or_else(|| "/usr/bin/node".into());
                self.exec_start.set_value(format!("{node} index.js"));
                self.restart_policy.set_selected_value(&RestartPolicy::OnFailure);
                if self.description.value().is_empty() {
                    self.description.set_value("Node.js Service");
                }
            }
            ServicePresetTemplate::DockerCompose => {
                let docker = ServiceUnitConfig::detect_binary("docker")
                    .unwrap_or_else(|| "/usr/bin/docker".into());
                self.exec_start.set_value(format!("{docker} compose up -d"));
                self.restart_policy.set_selected_value(&RestartPolicy::Always);
                if self.description.value().is_empty() {
                    self.description.set_value("Docker Compose Service");
                }
            }
            ServicePresetTemplate::Binary => {
                self.exec_start.set_value("/usr/local/bin/my-app");
                self.restart_policy.set_selected_value(&RestartPolicy::OnFailure);
                if self.description.value().is_empty() {
                    self.description.set_value("Binary Application");
                }
            }
            ServicePresetTemplate::Custom => {}
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> CreateServiceResult {
        if key.kind == KeyEventKind::Release {
            return CreateServiceResult::Editing;
        }

        // Global shortcuts
        if key.code == KeyCode::Esc {
            return CreateServiceResult::Cancelled;
        }

        if key.code == KeyCode::F(2)
            || (key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return self.validate_and_submit();
        }

        // Field navigation: Up / Down
        match key.code {
            KeyCode::Up => {
                self.current_field = if self.current_field == 0 {
                    10
                } else {
                    self.current_field - 1
                };
                self.error = None;
                return CreateServiceResult::Editing;
            }
            KeyCode::Down => {
                self.current_field = (self.current_field + 1) % 11;
                self.error = None;
                return CreateServiceResult::Editing;
            }
            KeyCode::Enter if self.current_field == 10 => {
                return self.validate_and_submit();
            }
            KeyCode::Enter => {
                // Enter moves to next field
                self.current_field = (self.current_field + 1) % 11;
                self.error = None;
                return CreateServiceResult::Editing;
            }
            _ => {}
        }

        // Delegate to focused field
        match self.current_field {
            0 => {
                self.scope_selector.handle_key(key);
            }
            1 => {
                let prev_tpl = *self.template_selector.selected_value();
                if self.template_selector.handle_key(key) {
                    let new_tpl = *self.template_selector.selected_value();
                    if new_tpl != prev_tpl {
                        self.apply_template(new_tpl);
                    }
                }
            }
            2 => {
                self.service_name.handle_key(key);
            }
            3 => {
                self.description.handle_key(key);
            }
            4 => {
                self.working_dir.handle_key(key);
            }
            5 => {
                self.exec_start.handle_key(key);
            }
            6 => {
                self.env_file.handle_key(key);
            }
            7 => {
                if *self.scope_selector.selected_value() != ServiceScope::User {
                    self.run_user.handle_key(key);
                }
            }
            8 => {
                self.restart_policy.handle_key(key);
            }
            9 => {
                self.enable_on_boot.handle_key(key);
            }
            10 => {
                self.start_immediately.handle_key(key);
            }
            _ => {}
        }

        CreateServiceResult::Editing
    }

    fn validate_and_submit(&mut self) -> CreateServiceResult {
        let name = self.service_name.value().trim();
        if name.is_empty() {
            self.error = Some(if self.is_chinese {
                "服务名称不能为空！请输入服务名。".to_string()
            } else {
                "Service name cannot be empty!".to_string()
            });
            self.current_field = 2;
            return CreateServiceResult::Editing;
        }

        let exec = self.exec_start.value().trim();
        if exec.is_empty() {
            self.error = Some(if self.is_chinese {
                "启动命令 ExecStart 不能为空！".to_string()
            } else {
                "ExecStart command cannot be empty!".to_string()
            });
            self.current_field = 5;
            return CreateServiceResult::Editing;
        }

        let scope = *self.scope_selector.selected_value();
        let config = ServiceUnitConfig {
            service_name: name.to_string(),
            description: self.description.value().trim().to_string(),
            working_dir: self.working_dir.value().trim().to_string(),
            exec_start: exec.to_string(),
            env_file: self.env_file.value().trim().to_string(),
            run_user: if scope == ServiceScope::User {
                String::new()
            } else {
                self.run_user.selected_value().clone()
            },
            restart: *self.restart_policy.selected_value(),
            enable_on_boot: self.enable_on_boot.value(),
            start_immediately: self.start_immediately.value(),
            scope,
        };

        CreateServiceResult::Submitted(config)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let width = std::cmp::min(84, area.width.saturating_sub(4));
        let height = std::cmp::min(32, area.height.saturating_sub(2));

        let popup_x = (area.width.saturating_sub(width)) / 2;
        let popup_y = (area.height.saturating_sub(height)) / 2;

        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            width,
            height,
        );

        frame.render_widget(Clear, popup_area);

        let title = if self.is_chinese {
            " ✨ 新建 Systemd 服务单元 (Create Service) ✨ "
        } else {
            " ✨ Create Systemd Service Unit ✨ "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(2, 2, 1, 1))
            .title(Line::from(Span::styled(
                title,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Layout rows
        let rows = Layout::vertical([
            Constraint::Length(2), // 0: Scope
            Constraint::Length(2), // 1: Template
            Constraint::Length(2), // 2: Service Name
            Constraint::Length(2), // 3: Description
            Constraint::Length(2), // 4: Working Dir
            Constraint::Length(2), // 5: ExecStart
            Constraint::Length(2), // 6: Env File
            Constraint::Length(2), // 7: User
            Constraint::Length(2), // 8: Restart
            Constraint::Length(2), // 9: Enable on boot
            Constraint::Length(2), // 10: Start immediately
            Constraint::Length(2), // 11: Scope Notice / Error message
            Constraint::Length(2), // 12: Shortcuts footer
        ])
        .split(inner);

        let is_user_mode = *self.scope_selector.selected_value() == ServiceScope::User;

        // Render each field
        self.render_selector(frame, rows[0], 0, &self.scope_selector.label, self.scope_selector.selected_label(), " [←/→/Space]");
        self.render_selector(frame, rows[1], 1, &self.template_selector.label, self.template_selector.selected_label(), " [←/→/Space]");
        self.render_text_field(frame, rows[2], 2, &self.service_name.label, self.service_name.value(), &self.service_name.placeholder, self.service_name.cursor);
        self.render_text_field(frame, rows[3], 3, &self.description.label, self.description.value(), &self.description.placeholder, self.description.cursor);
        self.render_path_field(frame, rows[4], 4, &self.working_dir.text_field.label, self.working_dir.value(), &self.working_dir.text_field.placeholder, self.working_dir.text_field.cursor);
        self.render_path_field(frame, rows[5], 5, &self.exec_start.text_field.label, self.exec_start.value(), &self.exec_start.text_field.placeholder, self.exec_start.text_field.cursor);
        self.render_path_field(frame, rows[6], 6, &self.env_file.text_field.label, self.env_file.value(), &self.env_file.text_field.placeholder, self.env_file.text_field.cursor);

        let user_label = if is_user_mode {
            if self.is_chinese {
                "当前用户 (免 sudo 自动隔离 / --user)"
            } else {
                "Current user (Auto-isolated / --user)"
            }
        } else {
            self.run_user.selected_label()
        };
        let user_hint = if is_user_mode {
            if self.is_chinese { " [已锁定为当前用户]" } else { " [Fixed to current user]" }
        } else {
            " [←/→/Space]"
        };
        self.render_selector(frame, rows[7], 7, &self.run_user.label, user_label, user_hint);

        self.render_selector(frame, rows[8], 8, &self.restart_policy.label, self.restart_policy.selected_label(), " [←/→/Space]");
        self.render_toggle(frame, rows[9], 9, &self.enable_on_boot.label, self.enable_on_boot.value());
        self.render_toggle(frame, rows[10], 10, &self.start_immediately.label, self.start_immediately.value());

        // Error message / Scope notice row
        if let Some(err) = &self.error {
            let p = Paragraph::new(Line::from(vec![
                Span::styled("❌ 错误: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(err, Style::default().fg(Color::LightRed)),
            ]));
            frame.render_widget(p, rows[11]);
        } else if is_user_mode {
            let tip = if self.is_chinese {
                "💡 提示: 用户服务写入 ~/.config/systemd/user/，免 sudo 密码。注销常驻: loginctl enable-linger"
            } else {
                "💡 Tip: Saved to ~/.config/systemd/user/, no sudo needed. Keep after logout: loginctl enable-linger"
            };
            let p = Paragraph::new(Line::from(Span::styled(tip, Style::default().fg(Color::Cyan))));
            frame.render_widget(p, rows[11]);
        } else {
            let tip = if self.is_chinese {
                "💡 提示: 系统服务写入 /etc/systemd/system/，需要 sudo / root 权限"
            } else {
                "💡 Tip: System service is written to /etc/systemd/system/, requires sudo / root privileges"
            };
            let p = Paragraph::new(Line::from(Span::styled(tip, Style::default().fg(Color::DarkGray))));
            frame.render_widget(p, rows[11]);
        }

        // Footer shortcuts
        let footer_text = if self.is_chinese {
            "[↑/↓] 切换输入框  [Tab] 路径补全  [←/→] 切换选项  [F2/Ctrl+S] 创建服务  [Esc] 取消"
        } else {
            "[↑/↓] Switch Field  [Tab] Auto-complete  [←/→] Select  [F2/Ctrl+S] Create  [Esc] Cancel"
        };
        let footer = Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(Color::Gray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(footer, rows[12]);
    }

    fn render_text_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        field_idx: usize,
        label: &str,
        value: &str,
        placeholder: &str,
        cursor: usize,
    ) {
        let is_focused = self.current_field == field_idx;
        let label_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let pointer = if is_focused { "▶ " } else { "  " };

        let display_value = if value.is_empty() {
            Span::styled(placeholder, Style::default().fg(Color::DarkGray))
        } else {
            Span::styled(value, Style::default().fg(Color::White))
        };

        let line = Line::from(vec![
            Span::styled(pointer, Style::default().fg(Color::Yellow)),
            Span::styled(format!("{label:<18}: "), label_style),
            Span::styled("[ ", Style::default().fg(Color::DarkGray)),
            display_value,
            Span::styled(" ]", Style::default().fg(Color::DarkGray)),
        ]);

        let p = Paragraph::new(line);
        frame.render_widget(p, area);

        // Show cursor if focused
        if is_focused {
            let label_len = 2 + 18 + 2 + 2; // pointer + label + colon-space + "[ "
            let cursor_x = area.x + label_len as u16 + cursor as u16;
            if cursor_x < area.x + area.width {
                frame.set_cursor_position((cursor_x, area.y));
            }
        }
    }

    fn render_path_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        field_idx: usize,
        label: &str,
        value: &str,
        placeholder: &str,
        cursor: usize,
    ) {
        let is_focused = self.current_field == field_idx;
        let label_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let pointer = if is_focused { "▶ " } else { "  " };

        let display_value = if value.is_empty() {
            Span::styled(placeholder, Style::default().fg(Color::DarkGray))
        } else {
            Span::styled(value, Style::default().fg(Color::White))
        };

        let tab_hint = if is_focused {
            Span::styled(" (Tab ⇥ 补全)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        };

        let line = Line::from(vec![
            Span::styled(pointer, Style::default().fg(Color::Yellow)),
            Span::styled(format!("{label:<18}: "), label_style),
            Span::styled("[ ", Style::default().fg(Color::DarkGray)),
            display_value,
            Span::styled(" ]", Style::default().fg(Color::DarkGray)),
            tab_hint,
        ]);

        let p = Paragraph::new(line);
        frame.render_widget(p, area);

        if is_focused {
            let label_len = 2 + 18 + 2 + 2;
            let cursor_x = area.x + label_len as u16 + cursor as u16;
            if cursor_x < area.x + area.width {
                frame.set_cursor_position((cursor_x, area.y));
            }
        }
    }

    fn render_selector(
        &self,
        frame: &mut Frame,
        area: Rect,
        field_idx: usize,
        label: &str,
        selected_label: &str,
        hint: &str,
    ) {
        let is_focused = self.current_field == field_idx;
        let label_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let pointer = if is_focused { "▶ " } else { "  " };

        let line = Line::from(vec![
            Span::styled(pointer, Style::default().fg(Color::Yellow)),
            Span::styled(format!("{label:<18}: "), label_style),
            Span::styled("◀ ", Style::default().fg(if is_focused { Color::Cyan } else { Color::DarkGray })),
            Span::styled(selected_label, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" ▶", Style::default().fg(if is_focused { Color::Cyan } else { Color::DarkGray })),
            Span::styled(if is_focused { hint } else { "" }, Style::default().fg(Color::DarkGray)),
        ]);

        let p = Paragraph::new(line);
        frame.render_widget(p, area);
    }

    fn render_toggle(
        &self,
        frame: &mut Frame,
        area: Rect,
        field_idx: usize,
        label: &str,
        value: bool,
    ) {
        let is_focused = self.current_field == field_idx;
        let label_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let pointer = if is_focused { "▶ " } else { "  " };

        let status_span = if value {
            Span::styled("[ ✔ 是 (Yes) ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("[ ✖ 否 (No)  ]", Style::default().fg(Color::Red))
        };

        let line = Line::from(vec![
            Span::styled(pointer, Style::default().fg(Color::Yellow)),
            Span::styled(format!("{label:<18}: "), label_style),
            status_span,
            Span::styled(if is_focused { " [Space 切换]" } else { "" }, Style::default().fg(Color::DarkGray)),
        ]);

        let p = Paragraph::new(line);
        frame.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn applies_python_template() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        form.apply_template(ServicePresetTemplate::Python);
        assert!(form.exec_start.value().contains("python"));
        assert_eq!(*form.restart_policy.selected_value(), RestartPolicy::OnFailure);
    }

    #[test]
    fn applies_node_template() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        form.apply_template(ServicePresetTemplate::Node);
        assert!(form.exec_start.value().contains("node"));
        assert_eq!(*form.restart_policy.selected_value(), RestartPolicy::OnFailure);
    }

    #[test]
    fn applies_docker_template() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        form.apply_template(ServicePresetTemplate::DockerCompose);
        assert!(form.exec_start.value().contains("docker"));
        assert_eq!(*form.restart_policy.selected_value(), RestartPolicy::Always);
    }

    #[test]
    fn validation_fails_on_empty_name() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        form.exec_start.set_value("/usr/bin/my-app");
        let result = form.on_key(make_key(KeyCode::F(2)));
        assert_eq!(result, CreateServiceResult::Editing);
        assert!(form.error.is_some());
    }

    #[test]
    fn validation_fails_on_empty_exec_start() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        form.service_name.set_value("my-service");
        let result = form.on_key(make_key(KeyCode::F(2)));
        assert_eq!(result, CreateServiceResult::Editing);
        assert!(form.error.is_some());
    }

    #[test]
    fn validation_succeeds_for_user_service() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        form.service_name.set_value("my-service");
        form.exec_start.set_value("/usr/bin/my-app");
        form.description.set_value("Test Service");
        form.working_dir.set_value("/opt/test");
        form.env_file.set_value("/opt/test/.env");

        let result = form.on_key(make_key(KeyCode::F(2)));
        match result {
            CreateServiceResult::Submitted(config) => {
                assert_eq!(config.service_name, "my-service");
                assert_eq!(config.unit_file_name(), "my-service.service");
                assert_eq!(config.exec_start, "/usr/bin/my-app");
                assert_eq!(config.working_dir, "/opt/test");
                assert_eq!(config.env_file, "/opt/test/.env");
                assert!(config.enable_on_boot);
                assert!(config.start_immediately);
                assert_eq!(config.scope, ServiceScope::User);
                // User mode should not specify run_user
                assert!(config.run_user.is_empty());
            }
            _ => panic!("Expected Submitted result"),
        }
    }

    #[test]
    fn validation_succeeds_for_system_service() {
        let mut form = CreateServiceForm::new(true, ServiceScope::System);
        form.service_name.set_value("sys-service");
        form.exec_start.set_value("/usr/bin/sys-daemon");

        let result = form.on_key(make_key(KeyCode::F(2)));
        match result {
            CreateServiceResult::Submitted(config) => {
                assert_eq!(config.service_name, "sys-service");
                assert_eq!(config.scope, ServiceScope::System);
                assert_eq!(config.run_user, "root");
            }
            _ => panic!("Expected Submitted result"),
        }
    }

    #[test]
    fn key_navigation_cycles_fields() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        assert_eq!(form.current_field, 0);

        form.on_key(make_key(KeyCode::Down));
        assert_eq!(form.current_field, 1);

        form.on_key(make_key(KeyCode::Up));
        assert_eq!(form.current_field, 0);

        form.on_key(make_key(KeyCode::Up));
        assert_eq!(form.current_field, 10);
    }

    #[test]
    fn esc_cancels_form() {
        let mut form = CreateServiceForm::new(true, ServiceScope::User);
        let result = form.on_key(make_key(KeyCode::Esc));
        assert_eq!(result, CreateServiceResult::Cancelled);
    }
}

