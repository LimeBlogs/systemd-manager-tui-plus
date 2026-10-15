use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Tabs};
use rayon::prelude::*;
use crate::terminal::theme::{Theme, glyphs};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{
    io::{self},
    process::Command,
};

use crate::Config;
use crate::domain::connection_type::ConnectionType;
use crate::domain::service_unit::ServiceScope;
use crate::infrastructure::systemd_service_adapter::CompletedOperation;
use crate::terminal::components::list::ActiveFilterState;
use crate::usecases::services_manager::ServicesManager;

use super::components::confirm_dialog::{ConfirmDialog, ConfirmResult};
use super::components::create_service::{CreateServiceForm, CreateServiceResult};
use super::components::details::ServiceDetails;
use super::components::filter::{Filter, InputMode};
use super::components::list::{ServiceAction, TableServices};
use super::components::log::ServiceLog;

const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug, PartialEq)]
enum Status {
    List,
    Log,
    Details,
    Create,
}

pub enum Actions {
    RefreshLog,
    RefreshDetails,
    GoList,
    ResetList,
    GoLog,
    GoDetails,
    GoCreate,
    PromptUninstall,
    UpdateLog(ServiceRequestContext, String),
    #[allow(dead_code)]
    UpdateDetails,
    Filter(String),
    UpdateIgnoreListKeys(bool),
    EditCurrentService,
    ServiceAction(ServiceAction),
    ShowHelp,
    Redraw,
    UpdateTimestamp(ServiceRequestContext, Option<u64>),
    OperationCompleted(CompletedOperation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceRequestContext {
    pub connection: ConnectionType,
    pub service_name: String,
}

pub enum AppEvent {
    Key(KeyEvent),
    Action(Actions),
    Error(String),
}

#[allow(dead_code)]
pub fn get_user_friendly_error(error: &str) -> &str {
    get_user_friendly_error_i18n(error, false)
}

pub fn get_user_friendly_error_i18n(error: &str, is_chinese: bool) -> &str {
    if is_chinese {
        if error.contains("org.freedesktop.DBus.Error.InteractiveAuthorizationRequired") {
            "操作权限不足，请尝试使用 sudo 重新运行本程序！"
        } else if error.contains("org.freedesktop.DBus.Error.ServiceUnknown") {
            "请求的服务不存在或未运行。"
        } else if error.contains("org.freedesktop.DBus.Error.NoReply") {
            "服务未及时响应，可能正忙或运行异常。"
        } else if error.contains("org.freedesktop.DBus.Error.AccessDenied") {
            "访问被拒绝，当前账户权限不足。"
        } else if error.contains("org.freedesktop.systemd1.NoSuchUnit") {
            "请求的系统服务单元不存在。"
        } else {
            error
        }
    } else if error.contains("org.freedesktop.DBus.Error.InteractiveAuthorizationRequired") {
        "You do not have the permission to do that. Try running the program with sudo."
    } else if error.contains("org.freedesktop.DBus.Error.ServiceUnknown") {
        "The requested service is not available or not running."
    } else if error.contains("org.freedesktop.DBus.Error.NoReply") {
        "The service did not respond in time. It might be busy or not functioning properly."
    } else if error.contains("org.freedesktop.DBus.Error.AccessDenied") {
        "Access denied. You don't have sufficient permissions for this operation."
    } else if error.contains("org.freedesktop.systemd1.NoSuchUnit") {
        "The requested service unit doesn't exist."
    } else {
        error
    }
}

pub struct App {
    running: bool,
    status: Status,
    event_listener_enabled: Arc<AtomicBool>,
    event_listener_shutdown: Arc<AtomicBool>,
    event_listener_handle: Option<JoinHandle<()>>,
    table_service: TableServices,
    filter: Filter,
    service_log: ServiceLog,
    details: ServiceDetails,
    usecases: Rc<RefCell<ServicesManager>>,
    event_rx: Receiver<AppEvent>,
    event_tx: Sender<AppEvent>,
    selected_tab_index: usize,
    active_connection: ConnectionType,
    show_help: bool,
    errors: VecDeque<String>,
    create_form: Option<CreateServiceForm>,
    confirm_dialog: Option<ConfirmDialog>,
    pub is_chinese: bool,
}

impl App {
    pub fn new(
        event_tx: Sender<AppEvent>,
        event_rx: Receiver<AppEvent>,
        table_service: TableServices,
        filter: Filter,
        service_log: ServiceLog,
        details: ServiceDetails,
        usecases: Rc<RefCell<ServicesManager>>,
    ) -> Self {
        Self {
            running: true,
            status: Status::List,
            event_listener_enabled: Arc::new(AtomicBool::new(true)),
            event_listener_shutdown: Arc::new(AtomicBool::new(false)),
            event_listener_handle: None,
            table_service,
            filter,
            service_log,
            details,
            usecases,
            event_rx,
            event_tx,
            selected_tab_index: 0,
            active_connection: ConnectionType::System,
            show_help: false,
            errors: VecDeque::new(),
            create_form: None,
            confirm_dialog: None,
            is_chinese: false,
        }
    }

    pub fn init(&mut self, config: Config) {
        self.is_chinese = config.language.is_chinese();
        self.service_log.set_chinese(self.is_chinese);
        self.details.set_chinese(self.is_chinese);
        self.table_service.init(&config);
        let _ = self
            .event_tx
            .send(AppEvent::Action(Actions::Filter(config.filter)));
        self.spawn_key_event_listener();
    }

    fn spawn_key_event_listener(&mut self) {
        if self.event_listener_handle.is_some() {
            return;
        }
        let event_tx = self.event_tx.clone();
        let event_listener_enabled = self.event_listener_enabled.clone();
        let shutdown = self.event_listener_shutdown.clone();

        self.event_listener_handle = Some(thread::spawn(move || {
            loop {
                if shutdown.load(Ordering::Acquire) {
                    break;
                }
                if !event_listener_enabled.load(Ordering::Relaxed) {
                    thread::sleep(INPUT_POLL_INTERVAL);
                    continue;
                }

                match event::poll(INPUT_POLL_INTERVAL) {
                    Ok(true) => match event::read() {
                        Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                            if event_tx.send(AppEvent::Key(key_event)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(error) => {
                            let _ = event_tx.send(AppEvent::Error(format!(
                                "Failed to read terminal event: {error}"
                            )));
                            break;
                        }
                    },
                    Ok(false) => {}
                    Err(error) => {
                        let _ = event_tx.send(AppEvent::Error(format!(
                            "Failed to poll terminal events: {error}"
                        )));
                        break;
                    }
                }
            }
        }));
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.running = true;

        while self.running {
            if let Some(error) = self.errors.front() {
                terminal.draw(|frame| self.draw_error_popup(frame, frame.area(), error))?;
            } else {
                match self.status {
                    Status::Log => self.draw_log_status(terminal)?,
                    Status::List => self.draw_list_status(terminal)?,
                    Status::Details => self.draw_details_status(terminal)?,
                    Status::Create => self.draw_create_status(terminal)?,
                }
            }

            let use_timeout =
                self.status == Status::List && self.table_service.has_active_runtime();

            let event = if use_timeout {
                match self.event_rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(ev) => ev,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            } else {
                self.event_rx.recv()?
            };

            match event {
                AppEvent::Key(key)
                    if !self.errors.is_empty()
                        || self.confirm_dialog.is_some()
                        || self.table_service.has_instance_prompt() =>
                {
                    self.handle_modal_key(key, terminal)?;
                }
                AppEvent::Key(key) => match self.status {
                    Status::Create => {
                        self.handle_create_key(key, terminal)?;
                    }
                    Status::Log => {
                        if self.show_help {
                            self.show_help = false;
                        } else {
                            self.on_key_event(key, terminal)?;
                            self.service_log.on_key_event(key);
                        }
                    }
                    Status::List => {
                        if self.show_help {
                            self.show_help = false;
                            // Ensure the table is active and can receive key events
                            self.table_service.set_ignore_key_events(false);
                            // If no item is selected and list is not empty, select first item
                            if self.table_service.table_state.selected().is_none()
                                && !self.table_service.is_filtered_list_empty()
                            {
                                self.table_service.set_selected_index(0);
                            }
                        } else {
                            self.on_key_event(key, terminal)?;
                            self.on_key_horizontal_event(
                                key,
                                self.filter.input_mode == InputMode::Editing,
                            );
                            self.table_service.on_key_event(key);
                            self.filter.on_key_event(key);
                        }
                    }
                    Status::Details => {
                        if self.show_help {
                            self.show_help = false;
                        } else {
                            self.on_key_event(key, terminal)?;
                            self.details.on_key_event(key);
                        }
                    }
                },
                AppEvent::Action(Actions::GoCreate) => {
                    let default_scope = match self.active_connection {
                        ConnectionType::Session => ServiceScope::User,
                        ConnectionType::System => ServiceScope::System,
                    };
                    let mut form = CreateServiceForm::new(self.is_chinese, default_scope);
                    form.set_chinese(self.is_chinese);
                    self.create_form = Some(form);
                    self.status = Status::Create;
                }
                AppEvent::Action(Actions::PromptUninstall) => {
                    if let Some(service) = self.table_service.get_selected_service() {
                        let name = service.name();
                        if let Some(path) = self.usecases.borrow().get_unit_file_path(name) {
                            self.confirm_dialog = Some(ConfirmDialog::new(
                                name,
                                path.to_string_lossy(),
                                self.is_chinese,
                            ));
                        } else {
                            self.report_error(if self.is_chinese {
                                "无法卸载：仅支持删除位于 /etc/systemd/system/ (或会话目录) 下的自定义服务，系统内置服务受保护不可删除。"
                            } else {
                                "Cannot uninstall: only custom unit files in /etc/systemd/system/ (or session path) can be removed."
                            })?;
                        }
                    }
                }
                AppEvent::Action(Actions::ServiceAction(action)) => {
                    self.table_service.act_on_selected_service(&action);
                    self.table_service.invalidate_timestamp();
                }
                AppEvent::Action(Actions::UpdateIgnoreListKeys(bool)) => {
                    self.table_service.set_ignore_key_events(bool);
                }
                AppEvent::Action(Actions::Filter(input)) => {
                    self.table_service.set_selected_index(0);
                    self.table_service.refresh(&input);
                }
                AppEvent::Action(Actions::UpdateLog(context, log)) => {
                    if self.status == Status::Log && self.is_current_service_context(&context) {
                        self.service_log.update(context.service_name, log);
                    }
                }
                AppEvent::Action(Actions::RefreshLog) => {
                    if self.status == Status::Log
                        && let Some(service) = self.table_service.get_selected_service()
                    {
                        self.service_log
                            .fetch_log_and_dispatch(self.service_context(&service));
                    }
                }
                AppEvent::Action(Actions::GoLog) => {
                    self.status = Status::Log;
                    self.event_tx.send(AppEvent::Action(Actions::RefreshLog))?;
                }
                AppEvent::Action(Actions::GoList) => self.status = Status::List,
                AppEvent::Action(Actions::ResetList) => {
                    self.table_service.set_usecase(self.usecases.clone());
                }
                AppEvent::Action(Actions::UpdateTimestamp(context, ts)) => {
                    self.table_service.update_timestamp(context, ts);
                }
                AppEvent::Action(Actions::OperationCompleted(completion)) => {
                    if completion.connection == self.active_connection {
                        self.table_service.refresh_current();
                        if completion.result != "done" {
                            self.event_tx.send(AppEvent::Error(format!(
                                "systemd finished {} for '{}' with result '{}' after the earlier timeout",
                                completion.action, completion.service, completion.result
                            )))?;
                        }
                    }
                }
                AppEvent::Action(Actions::UpdateDetails | Actions::Redraw) => {}
                AppEvent::Action(Actions::RefreshDetails) => {
                    if self.status == Status::Details {
                        self.details.fetch_unit_file();
                    }
                }
                AppEvent::Action(Actions::GoDetails) => {
                    if let Some(service) = self.table_service.get_selected_service() {
                        self.details.update(service.clone());
                    }
                    self.event_tx
                        .send(AppEvent::Action(Actions::RefreshDetails))?;
                    self.status = Status::Details;
                }
                AppEvent::Action(Actions::EditCurrentService) => {
                    if let Some(service) = &self.table_service.get_selected_service() {
                        self.edit_unit(terminal, service.name())?;
                        self.event_tx
                            .send(AppEvent::Action(Actions::RefreshDetails))?;
                    }
                }
                AppEvent::Error(error_msg) => {
                    self.errors.push_back(error_msg);
                }
                AppEvent::Action(Actions::ShowHelp) => {
                    self.show_help = !self.show_help;
                }
            }
        }

        // Restore the user's screen before worker Drop implementations wait for
        // their threads. Shutdown remains orderly, but the TUI disappears as
        // soon as Ctrl+C is handled.
        ratatui::restore();

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn resume_tui(&self, terminal: &mut DefaultTerminal) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(Clear, area);
        })?;
        Ok(())
    }

    fn edit_unit(&self, terminal: &mut DefaultTerminal, unit_name: &str) -> Result<()> {
        self.event_listener_enabled.store(false, Ordering::Relaxed);

        if let Err(e) = disable_raw_mode() {
            self.resume_tui(terminal)?;
            self.report_error(&format!("Failed to disable raw mode: {e}"))?;
            return Ok(());
        }

        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, LeaveAlternateScreen) {
            self.resume_tui(terminal)?;
            self.report_error(&format!("Failed to leave alternate screen: {e}"))?;
            return Ok(());
        }

        if let Err(e) = terminal.show_cursor() {
            self.resume_tui(terminal)?;
            self.report_error(&format!("Failed to show cursor: {e}"))?;
            return Ok(());
        }

        let mut cmd = Command::new("systemctl");

        cmd.arg("edit").arg("--full");

        if self.selected_tab_index == 1 {
            cmd.arg("--user");
        }

        let status = cmd.arg(unit_name).status();

        match status {
            Ok(s) if s.success() => {}
            Ok(_s) => {
                self.resume_tui(terminal)?;
                self.report_error("'systemctl edit' failed. Try running the program with sudo!")?;
            }
            Err(e) => {
                self.resume_tui(terminal)?;
                self.report_error(&format!("Error executing systemctl: {e}"))?;
            }
        }

        if let Err(e) = self.resume_tui(terminal) {
            self.report_error(&format!("Failed to return to TUI: {e}"))?;
            return Ok(());
        }

        self.event_listener_enabled.store(true, Ordering::Relaxed);

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn draw_help_popup(&self, frame: &mut Frame, area: Rect) {
        let popup_width = std::cmp::min(80, area.width.saturating_sub(4));
        let popup_height = std::cmp::min(38, area.height.saturating_sub(4));

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            popup_width,
            popup_height,
        );

        frame.render_widget(Clear, popup_area);

        let title = if self.is_chinese { "SYSTEMD 管理终端 (TUI) - 快捷键帮助" } else { "SYSTEMD MANAGER TUI - HELP" };
        let nav_title = if self.is_chinese { "导航操作:" } else { "Navigation:" };
        let filter_title = if self.is_chinese { "搜索过滤:" } else { "Filter:" };
        let control_title = if self.is_chinese { "服务控制:" } else { "Service Control:" };
        let view_title = if self.is_chinese { "列表视图与过滤:" } else { "View & Filter list:" };
        let info_title = if self.is_chinese { "详细信息:" } else { "Information:" };
        let app_title = if self.is_chinese { "程序全局:" } else { "Application:" };
        let close_hint = if self.is_chinese { "按 ? 或任意键关闭此窗口" } else { "Press ? or any key to close" };

        let theme = Theme::current();
        let text = vec![
            Line::from(vec![Span::styled(
                title,
                theme.accent_style(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                nav_title,
                theme.accent_style(),
            )]),
            Line::from(if self.is_chinese { "↑/k - 向上移动    ↓/j - 向下移动" } else { "↑/k - Move up    ↓/j - Move down" }),
            Line::from(if self.is_chinese { "←/h - 上一个标签页    →/l - 下一个标签页" } else { "←/h - Previous tab    →/l - Next tab" }),
            Line::from(if self.is_chinese { "PageUp/PageDown - 快速翻页 (10项)" } else { "PageUp/PageDown - Jump 10 items" }),
            Line::from(""),
            Line::from(vec![Span::styled(
                filter_title,
                theme.accent_style(),
            )]),
            Line::from(if self.is_chinese { "Ctrl+u - 删除至开头" } else { "Ctrl+u - Delete until the start of the input" }),
            Line::from(if self.is_chinese { "Ctrl+k - 删除至末尾" } else { "Ctrl+k - Delete until the end of the input" }),
            Line::from(if self.is_chinese { "Alt+b / Ctrl+← - 向左跳转词    Alt+f / Ctrl+→ - 向右跳转词" } else { "Alt+b or Ctrl+← - Go backwards a word    Alt+f or Ctrl+→ - Go forwards a word" }),
            Line::from(""),
            Line::from(vec![Span::styled(
                control_title,
                theme.accent_style(),
            )]),
            Line::from(if self.is_chinese { "s - 启动服务    x - 停止服务    r - 重启服务" } else { "s - Start service    x - Stop service    r - Restart service" }),
            Line::from(if self.is_chinese { "e - 启用开机自启    d - 禁用开机自启    m - 屏蔽/解除屏蔽" } else { "e - Enable service    d - Disable service    m - Mask/Unmask" }),
            Line::from(if self.is_chinese { "n - 新建系统服务 (.service)" } else { "n - Create new systemd service (.service)" }),
            Line::from(if self.is_chinese { "Del - 卸载并删除自定义服务配置文件" } else { "Del - Uninstall selected custom service file" }),
            Line::from(if self.is_chinese { "模板 (@.): s/x/r/e/d 提示输入实例名称" } else { "Templates (@.): s/x/r/e/d ask for an instance name" }),
            Line::from(""),
            Line::from(vec![Span::styled(
                view_title,
                theme.accent_style(),
            )]),
            Line::from(if self.is_chinese { "f - 切换所有单元/仅服务    a - 循环切换状态过滤" } else { "f - Toggle all/services filter    a - Cycle filter state" }),
            Line::from(if self.is_chinese { "u - 重新读取服务列表" } else { "u - Refresh service list" }),
            Line::from(""),
            Line::from(vec![Span::styled(
                info_title,
                theme.accent_style(),
            )]),
            Line::from(if self.is_chinese { "v - 查看实时日志 (journalctl)    c - 查看单元配置 (cat)" } else { "v - View service logs    c - View unit file details" }),
            Line::from(""),
            Line::from(vec![Span::styled(
                app_title,
                theme.accent_style(),
            )]),
            Line::from("Ctrl+z - Suspend    Ctrl+c - Quit"),
            Line::from(""),
            Line::from(vec![Span::styled(
                close_hint,
                theme.muted_style(),
            )]),
        ];

        let help_block = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border_focused))
                    .padding(Padding::new(1, 1, 0, 0))
                    .title(Line::from(Span::styled(
                        if self.is_chinese { " 帮助指南 " } else { " Help " },
                        theme.accent_style(),
                    ))),
            )
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(
            help_block,
            popup_area.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
        );
    }

    fn report_error(&self, error: &str) -> Result<()> {
        self.event_tx.send(AppEvent::Error(error.to_string()))?;
        Ok(())
    }

    fn handle_modal_key<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        if !self.errors.is_empty() {
            self.handle_error_key(key, terminal)
        } else if self.confirm_dialog.is_some() {
            self.handle_confirm_key(key, terminal)
        } else {
            self.handle_instance_key(key, terminal)
        }
    }

    fn handle_confirm_key<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        if self.handle_quit_key(key, terminal)? {
            return Ok(());
        }
        if let Some(mut dialog) = self.confirm_dialog.take() {
            match dialog.on_key(key) {
                ConfirmResult::Confirmed => {
                    let name = dialog.service_name.clone();
                    match self.usecases.borrow().uninstall_service(&name) {
                        Ok(()) => {
                            self.table_service.refresh_current();
                        }
                        Err(e) => {
                            self.report_error(&format!("Failed to uninstall: {e}"))?;
                        }
                    }
                }
                ConfirmResult::Cancelled => {
                    // Modal dismissed
                }
                ConfirmResult::Pending => {
                    self.confirm_dialog = Some(dialog);
                }
            }
        }
        Ok(())
    }

    fn handle_create_key<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        if self.handle_quit_key(key, terminal)? {
            return Ok(());
        }
        if let Some(mut form) = self.create_form.take() {
            match form.on_key(key) {
                CreateServiceResult::Editing => {
                    self.create_form = Some(form);
                }
                CreateServiceResult::Cancelled => {
                    self.status = Status::List;
                }
                CreateServiceResult::Submitted(config) => {
                    let target_conn = match config.scope {
                        ServiceScope::User => ConnectionType::Session,
                        ServiceScope::System => ConnectionType::System,
                    };

                    if self.active_connection != target_conn {
                        self.active_connection = target_conn;
                        self.selected_tab_index = match target_conn {
                            ConnectionType::System => 0,
                            ConnectionType::Session => 1,
                        };
                        self.table_service.set_active_connection(target_conn);
                        let _ = self
                            .usecases
                            .borrow_mut()
                            .change_repository_connection(target_conn);
                    }

                    match self.usecases.borrow().create_service(&config) {
                        Ok(unit_name) => {
                            self.table_service.refresh_current();
                            self.table_service.select_service_by_name(&unit_name);
                            self.status = Status::Log;
                            self.event_tx.send(AppEvent::Action(Actions::RefreshLog))?;
                        }
                        Err(err) => {
                            self.report_error(&format!("Failed to create service: {err}"))?;
                            self.status = Status::List;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_error_key<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        if !self.handle_quit_key(key, terminal)? {
            self.errors.pop_front();
        }
        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn draw_error_popup(&self, frame: &mut Frame, area: Rect, error_msg: &str) {
        let user_friendly_message = get_user_friendly_error_i18n(error_msg, self.is_chinese);
        let popup_width = std::cmp::min(70, area.width.saturating_sub(4));
        let popup_height = std::cmp::min(10, area.height.saturating_sub(4));

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            popup_width,
            popup_height,
        );

        frame.render_widget(Clear, popup_area);

        let error_title = if self.is_chinese { "错误提示 (ERROR)" } else { "ERROR" };
        let dismiss_text = if self.is_chinese { "按任意键关闭此窗口" } else { "Press any key to dismiss" };

        let text = vec![
            Line::from(vec![Span::styled(
                error_title,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(user_friendly_message),
            Line::from(""),
            Line::from(vec![Span::styled(
                dismiss_text,
                Style::default().fg(Color::Gray),
            )]),
        ];

        let error_block = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title(if self.is_chinese { "错误" } else { "Error" }),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(error_block, popup_area);
    }

    fn draw_create_status(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();
            if let Some(form) = &self.create_form {
                form.render(frame, area);
            }
        })?;
        Ok(())
    }

    fn draw_details_status(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();

            let [list_box, help_area_box] =
                Layout::vertical([Constraint::Min(0), Constraint::Max(7)]).areas(area);

            self.details.render(frame, list_box);
            self.draw_shortcuts(frame, help_area_box, &self.details.shortcuts());
        })?;

        Ok(())
    }

    fn draw_log_status(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();

            let [list_box, help_area_box] =
                Layout::vertical([Constraint::Min(0), Constraint::Max(7)]).areas(area);

            self.service_log.render(frame, list_box);
            self.draw_shortcuts(frame, help_area_box, &self.service_log.shortcuts());
        })?;

        Ok(())
    }

    fn draw_list_status(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();

            let [filter_box, tabs_box, list_box, help_area_box] = Layout::vertical([
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Min(10),
                Constraint::Max(7),
            ])
            .areas(area);

            let filter_state = &self.table_service.get_active_filter_state();

            let system_label = if self.is_chinese { "系统服务单元 (System units)" } else { "System units" };
            let session_label = if self.is_chinese { "用户会话单元 (Session units)" } else { "Session units" };
            let filter_word = if self.is_chinese { "状态" } else { "Filter" };

            let theme = Theme::current();
            let system_tab =
                if self.selected_tab_index == 0 && *filter_state != ActiveFilterState::All {
                    Line::from(vec![
                        Span::raw(system_label),
                        Span::styled(
                            format!(" ({filter_word}: {})", filter_state.as_str()),
                            theme.dim_style(),
                        ),
                    ])
                } else {
                    Line::from(system_label)
                };

            let session_tab =
                if self.selected_tab_index == 1 && *filter_state != ActiveFilterState::All {
                    Line::from(vec![
                        Span::raw(session_label),
                        Span::styled(
                            format!(" ({filter_word}: {})", filter_state.as_str()),
                            theme.dim_style(),
                        ),
                    ])
                } else {
                    Line::from(session_label)
                };

            let tabs = Tabs::new(vec![system_tab, session_tab])
                .select(self.selected_tab_index)
                .divider(glyphs::SEPARATOR)
                .style(theme.muted_style())
                .highlight_style(theme.accent_style());

            frame.render_widget(tabs, tabs_box);

            let shortcuts = self.table_service.shortcuts();
            self.draw_shortcuts(frame, help_area_box, &shortcuts);
            let table_service = &mut self.table_service;
            self.filter.draw(frame, filter_box);
            table_service.render(frame, list_box);

            // Show help popup if needed
            if self.show_help {
                self.draw_help_popup(frame, area);
            }
            if let Some(dialog) = &self.confirm_dialog {
                dialog.render(frame, area);
            }
            self.table_service.render_instance_prompt(frame, area);
        })?;

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn draw_shortcuts(&self, frame: &mut Frame, help_area: Rect, shortcuts: &[Line<'_>]) {
        let mut help_text: Vec<Line<'_>> = Vec::new();
        let shortcuts_lens = shortcuts.len();

        help_text.extend(shortcuts.to_owned());

        if shortcuts_lens > 0 {
            help_text.push(Line::raw(""));
            let shortcuts_width = shortcuts
                .to_owned()
                .par_iter()
                .map(|line| line.spans.iter().map(ratatui::prelude::Span::width).sum())
                .max()
                .unwrap_or(0);
            let shortcuts_width = u16::try_from(shortcuts_width).unwrap_or(u16::MAX);
            if help_area.width > shortcuts_width {
                help_text.push(Line::raw(""));
            }
        }

        let theme = Theme::current();
        help_text.push(Line::from(vec![
            Span::styled(
                "Exit",
                theme.failed_style(),
            ),
            Span::raw(": Ctrl + c"),
        ]));

        let help_block = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(Line::from(Span::styled(" Shortcuts ", theme.muted_style())))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(help_block, help_area);
    }

    fn handle_instance_key<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        if !self.handle_quit_key(key, terminal)? {
            self.table_service.on_instance_key(key);
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent, terminal: &mut DefaultTerminal) -> Result<()> {
        if self.handle_quit_key(key, terminal)? {
            return Ok(());
        }
        if let KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('z'),
            ..
        } = key
        {
            self.suspend_tui(terminal)?;
        }
        Ok(())
    }

    fn handle_quit_key<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<bool> {
        if matches!(key.code, KeyCode::Char('c' | 'C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            // The filter may have made the cursor visible in the last rendered frame.
            // Hide it before shutdown work starts so it cannot remain visible on the
            // alternate screen while workers are being joined.
            terminal.hide_cursor()?;
            self.quit();
            return Ok(true);
        }
        Ok(false)
    }

    fn suspend_tui(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        #[cfg(unix)]
        unsafe {
            libc::raise(libc::SIGTSTP);
        }
        #[cfg(not(unix))]
        {
            // Suspend is not supported on non-unix systems
        }

        self.resume_tui(terminal)?;

        Ok(())
    }

    fn on_key_horizontal_event(&mut self, key: KeyEvent, is_filtering: bool) {
        let left_keys = [KeyCode::Left, KeyCode::Char('h')];
        let right_keys = [KeyCode::Right, KeyCode::Char('l')];
        match key {
            KeyEvent { code, .. } if left_keys.contains(&code) => {
                if !is_filtering && self.status == Status::List {
                    let requested_tab_index = if self.selected_tab_index == 0 {
                        1
                    } else {
                        self.selected_tab_index - 1
                    };
                    self.update_connection_and_reset(requested_tab_index);
                }
            }

            KeyEvent { code, .. }
                if right_keys.contains(&code) && !is_filtering && self.status == Status::List =>
            {
                let requested_tab_index = (self.selected_tab_index + 1) % 2;
                self.update_connection_and_reset(requested_tab_index);
            }

            _ => {}
        }
    }
    fn update_connection_and_reset(&mut self, requested_tab_index: usize) {
        let requested_connection = match requested_tab_index {
            0 => ConnectionType::System,
            _ => ConnectionType::Session,
        };

        if let Err(err) = self
            .usecases
            .borrow_mut()
            .change_repository_connection(requested_connection)
        {
            let _ = self.event_tx.send(AppEvent::Error(format!(
                        "Failed to switch from {} to {requested_connection}: {err}. The {} connection remains active.",
                        self.active_connection, self.active_connection
                    )));
            return;
        }

        self.active_connection = requested_connection;
        self.selected_tab_index = requested_tab_index;
        self.table_service
            .set_active_connection(requested_connection);
        self.table_service.invalidate_timestamp();
        let _ = self.event_tx.send(AppEvent::Action(Actions::ResetList));
    }

    fn service_context(&self, service: &crate::domain::service::Service) -> ServiceRequestContext {
        ServiceRequestContext {
            connection: self.active_connection,
            service_name: service.name().to_string(),
        }
    }

    fn is_current_service_context(&self, context: &ServiceRequestContext) -> bool {
        context.connection == self.active_connection
            && self
                .table_service
                .get_selected_service()
                .is_some_and(|service| service.name() == context.service_name)
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.event_listener_shutdown.store(true, Ordering::Release);
        if let Some(handle) = self.event_listener_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests;
