pub mod en;
pub mod zh;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn from_str(lang: &str) -> Option<Self> {
        match lang.to_lowercase().as_str() {
            "zh" | "zh_cn" | "zh-cn" | "chinese" => Some(Self::Chinese),
            "en" | "en_us" | "en-us" | "english" => Some(Self::English),
            _ => None,
        }
    }

    pub fn detect(cli_lang: Option<&str>) -> Self {
        if let Some(l) = cli_lang {
            if let Some(parsed) = Self::from_str(l) {
                return parsed;
            }
        }

        let env_lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .unwrap_or_default();

        if env_lang.to_lowercase().starts_with("zh") {
            Self::Chinese
        } else {
            Self::English
        }
    }

    pub fn is_chinese(&self) -> bool {
        matches!(self, Self::Chinese)
    }
}

#[allow(dead_code)]
pub struct I18n {
    pub system_units: &'static str,
    pub session_units: &'static str,
    pub filter_format: &'static str,
    pub header_unit: &'static str,
    pub header_active: &'static str,
    pub header_file: &'static str,
    pub header_load: &'static str,
    pub header_desc: &'static str,
    pub actions_header: &'static str,
    pub list_shortcuts: &'static str,
    pub log_shortcuts: &'static str,
    pub log_shortcuts_live: &'static str,
    pub details_shortcuts: &'static str,
    pub help_title: &'static str,
    pub help_navigation: &'static str,
    pub help_navigation_lines: &'static [&'static str],
    pub help_filter: &'static str,
    pub help_filter_lines: &'static [&'static str],
    pub help_service_control: &'static str,
    pub help_service_control_lines: &'static [&'static str],
    pub help_application: &'static str,
    pub help_application_lines: &'static [&'static str],
    pub help_close_hint: &'static str,
    pub error_auth_required: &'static str,
    pub error_service_unknown: &'static str,
    pub error_no_reply: &'static str,
    pub error_access_denied: &'static str,
    pub error_no_such_unit: &'static str,
}

#[allow(dead_code)]
pub fn get_i18n(lang: Language) -> &'static I18n {
    match lang {
        Language::English => &en::EN_STRINGS,
        Language::Chinese => &zh::ZH_STRINGS,
    }
}
