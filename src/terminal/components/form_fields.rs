use super::path_completer::PathCompleter;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone)]
pub struct TextField {
    pub label: String,
    pub placeholder: String,
    pub value: String,
    pub cursor: usize,
}

impl TextField {
    pub fn new(label: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            placeholder: placeholder.into(),
            value: String::new(),
            cursor: 0,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map_or(self.value.len(), |(index, _)| index)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }

        match key.code {
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.value.chars().count());
                true
            }
            KeyCode::Home => {
                self.cursor = 0;
                true
            }
            KeyCode::End => {
                self.cursor = self.value.chars().count();
                true
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.value.remove(self.byte_index());
                true
            }
            KeyCode::Delete if self.cursor < self.value.chars().count() => {
                self.value.remove(self.byte_index());
                true
            }
            KeyCode::Char(c)
                if !c.is_control()
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.value.insert(self.byte_index(), c);
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathField {
    pub text_field: TextField,
    pub completer: PathCompleter,
}

impl PathField {
    pub fn new(label: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            text_field: TextField::new(label, placeholder),
            completer: PathCompleter::new(),
        }
    }

    pub fn value(&self) -> &str {
        self.text_field.value()
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.text_field.set_value(value);
        self.completer.reset();
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.text_field.clear();
        self.completer.reset();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }

        if key.code == KeyCode::Tab {
            if let Some(completed) = self.completer.complete(self.text_field.value()) {
                self.text_field.set_value(completed);
                return true;
            }
            return true;
        }

        // On any non-tab key, reset completion history
        self.completer.reset();
        self.text_field.handle_key(key)
    }
}

#[derive(Debug, Clone)]
pub struct SelectorField<T: Clone> {
    pub label: String,
    pub options: Vec<(T, String)>,
    pub selected_index: usize,
}

impl<T: Clone + PartialEq> SelectorField<T> {
    pub fn new(label: impl Into<String>, options: Vec<(T, String)>) -> Self {
        Self {
            label: label.into(),
            options,
            selected_index: 0,
        }
    }

    pub fn selected_value(&self) -> &T {
        &self.options[self.selected_index].0
    }

    pub fn selected_label(&self) -> &str {
        &self.options[self.selected_index].1
    }

    #[allow(dead_code)]
    pub fn set_selected_by_index(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = index;
        }
    }

    pub fn set_selected_value(&mut self, value: &T) {
        if let Some(idx) = self.options.iter().position(|(opt, _)| opt == value) {
            self.selected_index = idx;
        }
    }

    pub fn prev(&mut self) {
        if !self.options.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.options.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn next(&mut self) {
        if !self.options.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.options.len();
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.prev();
                true
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => {
                self.next();
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToggleField {
    pub label: String,
    pub value: bool,
}

impl ToggleField {
    pub fn new(label: impl Into<String>, initial: bool) -> Self {
        Self {
            label: label.into(),
            value: initial,
        }
    }

    pub fn value(&self) -> bool {
        self.value
    }

    #[allow(dead_code)]
    pub fn set_value(&mut self, value: bool) {
        self.value = value;
    }

    pub fn toggle(&mut self) {
        self.value = !self.value;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }

        match key.code {
            KeyCode::Char(' ') | KeyCode::Enter | KeyCode::Left | KeyCode::Right => {
                self.toggle();
                true
            }
            _ => false,
        }
    }
}
