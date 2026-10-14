use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    Pending,
    Confirmed,
    Cancelled,
}

pub struct ConfirmDialog {
    pub title: String,
    pub service_name: String,
    pub file_path: String,
    pub is_chinese: bool,
}

impl ConfirmDialog {
    pub fn new(service_name: impl Into<String>, file_path: impl Into<String>, is_chinese: bool) -> Self {
        Self {
            title: if is_chinese {
                "确认卸载服务 (Uninstall Service)".to_string()
            } else {
                "Confirm Uninstall Service".to_string()
            },
            service_name: service_name.into(),
            file_path: file_path.into(),
            is_chinese,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> ConfirmResult {
        if key.kind == KeyEventKind::Release {
            return ConfirmResult::Pending;
        }

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => ConfirmResult::Confirmed,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => ConfirmResult::Cancelled,
            _ => ConfirmResult::Pending,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let width = std::cmp::min(70, area.width.saturating_sub(4));
        let height = std::cmp::min(13, area.height.saturating_sub(4));

        let popup_x = (area.width.saturating_sub(width)) / 2;
        let popup_y = (area.height.saturating_sub(height)) / 2;

        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            width,
            height,
        );

        frame.render_widget(Clear, popup_area);

        let warning_text = if self.is_chinese {
            "此操作将停止服务、取消开机自启并彻底删除该配置文件！"
        } else {
            "This will stop the service, disable it, and permanently delete the file!"
        };

        let confirm_hint = if self.is_chinese {
            "[ y / Enter ] 确认卸载      [ n / Esc ] 取消"
        } else {
            "[ y / Enter ] Confirm      [ n / Esc ] Cancel"
        };

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if self.is_chinese { "目标服务: " } else { "Service: " },
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &self.service_name,
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    if self.is_chinese { "文件路径: " } else { "File: " },
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    &self.file_path,
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                warning_text,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                confirm_hint,
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
            )]),
        ];

        let block = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Red))
                    .padding(Padding::new(2, 2, 0, 0))
                    .title(Line::from(Span::styled(
                        format!(" {} ", self.title),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))),
            )
            .alignment(Alignment::Center);

        frame.render_widget(block, popup_area);
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
    fn confirms_on_y_or_enter() {
        let mut dialog = ConfirmDialog::new("my-service", "/etc/systemd/system/my-service.service", true);
        assert_eq!(dialog.on_key(make_key(KeyCode::Char('y'))), ConfirmResult::Confirmed);

        let mut dialog2 = ConfirmDialog::new("my-service", "/etc/systemd/system/my-service.service", true);
        assert_eq!(dialog2.on_key(make_key(KeyCode::Enter)), ConfirmResult::Confirmed);
    }

    #[test]
    fn cancels_on_n_or_esc() {
        let mut dialog = ConfirmDialog::new("my-service", "/etc/systemd/system/my-service.service", true);
        assert_eq!(dialog.on_key(make_key(KeyCode::Char('n'))), ConfirmResult::Cancelled);

        let mut dialog2 = ConfirmDialog::new("my-service", "/etc/systemd/system/my-service.service", true);
        assert_eq!(dialog2.on_key(make_key(KeyCode::Esc)), ConfirmResult::Cancelled);
    }

    #[test]
    fn ignores_other_keys() {
        let mut dialog = ConfirmDialog::new("my-service", "/etc/systemd/system/my-service.service", true);
        assert_eq!(dialog.on_key(make_key(KeyCode::Char('x'))), ConfirmResult::Pending);
    }
}

