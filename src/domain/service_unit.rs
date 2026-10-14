use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum RestartPolicy {
    No,
    #[default]
    OnFailure,
    Always,
    OnSuccess,
    OnAbnormal,
    OnAbort,
    OnWatchdog,
}

#[allow(dead_code)]
impl RestartPolicy {
    pub const ALL: [RestartPolicy; 7] = [
        RestartPolicy::OnFailure,
        RestartPolicy::Always,
        RestartPolicy::No,
        RestartPolicy::OnSuccess,
        RestartPolicy::OnAbnormal,
        RestartPolicy::OnAbort,
        RestartPolicy::OnWatchdog,
    ];

    pub fn as_systemd_str(&self) -> &'static str {
        match self {
            Self::No => "no",
            Self::OnFailure => "on-failure",
            Self::Always => "always",
            Self::OnSuccess => "on-success",
            Self::OnAbnormal => "on-abnormal",
            Self::OnAbort => "on-abort",
            Self::OnWatchdog => "on-watchdog",
        }
    }
}

impl fmt::Display for RestartPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_systemd_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ServicePresetTemplate {
    #[default]
    Custom,
    Python,
    Node,
    DockerCompose,
    Binary,
}

#[allow(dead_code)]
impl ServicePresetTemplate {
    pub const ALL: [ServicePresetTemplate; 5] = [
        ServicePresetTemplate::Custom,
        ServicePresetTemplate::Python,
        ServicePresetTemplate::Node,
        ServicePresetTemplate::DockerCompose,
        ServicePresetTemplate::Binary,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Custom => "Custom / 自定义",
            Self::Python => "Python Script / 脚本",
            Self::Node => "Node.js Service",
            Self::DockerCompose => "Docker Compose",
            Self::Binary => "Generic Binary / 通用程序",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceScope {
    #[default]
    User,
    System,
}

#[allow(dead_code)]
impl ServiceScope {
    pub const ALL: [ServiceScope; 2] = [ServiceScope::User, ServiceScope::System];

    pub fn label(&self, is_chinese: bool) -> &'static str {
        match self {
            Self::User => {
                if is_chinese {
                    "用户服务 (--user, 无需 root 权限)"
                } else {
                    "User Service (--user, No root required)"
                }
            }
            Self::System => {
                if is_chinese {
                    "系统服务 (系统全局, 需 root 权限)"
                } else {
                    "System Service (System-wide, requires root)"
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceUnitConfig {
    pub service_name: String,
    pub description: String,
    pub working_dir: String,
    pub exec_start: String,
    pub env_file: String,
    pub run_user: String,
    pub restart: RestartPolicy,
    pub enable_on_boot: bool,
    pub start_immediately: bool,
    pub scope: ServiceScope,
}

impl Default for ServiceUnitConfig {
    fn default() -> Self {
        Self {
            service_name: String::new(),
            description: String::new(),
            working_dir: String::new(),
            exec_start: String::new(),
            env_file: String::new(),
            run_user: "root".to_string(),
            restart: RestartPolicy::OnFailure,
            enable_on_boot: true,
            start_immediately: true,
            scope: ServiceScope::User,
        }
    }
}

impl ServiceUnitConfig {
    /// Returns the unit file name, ensuring it ends with `.service`.
    pub fn unit_file_name(&self) -> String {
        let trimmed = self.service_name.trim();
        if trimmed.ends_with(".service") {
            trimmed.to_string()
        } else {
            format!("{trimmed}.service")
        }
    }

    /// Generates standard systemd unit file content
    pub fn generate_unit_content(&self) -> String {
        let mut content = String::new();

        // [Unit] section
        content.push_str("[Unit]\n");
        let desc = if self.description.trim().is_empty() {
            &self.service_name
        } else {
            self.description.trim()
        };
        content.push_str(&format!("Description={desc}\n"));
        content.push_str("After=network.target\n\n");

        // [Service] section
        content.push_str("[Service]\n");
        content.push_str("Type=simple\n");

        // In user mode (--user), systemd already executes under the user's UID/GID.
        // Specifying User= in a user unit will cause errors. Only write User= in System mode!
        if self.scope == ServiceScope::System {
            let run_user = self.run_user.trim();
            if !run_user.is_empty() && run_user != "root" {
                content.push_str(&format!("User={run_user}\n"));
            }
        }

        let working_dir = self.working_dir.trim();
        if !working_dir.is_empty() {
            content.push_str(&format!("WorkingDirectory={working_dir}\n"));
        }

        let env_file = self.env_file.trim();
        if !env_file.is_empty() {
            content.push_str(&format!("EnvironmentFile={env_file}\n"));
        }

        content.push_str(&format!("ExecStart={}\n", self.exec_start.trim()));
        content.push_str(&format!("Restart={}\n", self.restart.as_systemd_str()));

        // [Install] section
        content.push_str("\n[Install]\n");
        if self.scope == ServiceScope::User {
            content.push_str("WantedBy=default.target\n");
        } else {
            content.push_str("WantedBy=multi-user.target\n");
        }

        content
    }

    /// Try to locate binary path on system (checking common locations or PATH)
    pub fn detect_binary(binary_name: &str) -> Option<String> {
        let common_paths = [
            format!("/usr/bin/{binary_name}"),
            format!("/usr/local/bin/{binary_name}"),
            format!("/bin/{binary_name}"),
        ];

        for p in &common_paths {
            if Path::new(p).exists() {
                return Some(p.clone());
            }
        }

        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(binary_name);
                if candidate.exists() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_file_name_appends_service() {
        let config = ServiceUnitConfig {
            service_name: "my-app".into(),
            ..Default::default()
        };
        assert_eq!(config.unit_file_name(), "my-app.service");

        let config2 = ServiceUnitConfig {
            service_name: "my-app.service".into(),
            ..Default::default()
        };
        assert_eq!(config2.unit_file_name(), "my-app.service");
    }

    #[test]
    fn generates_expected_content() {
        let config = ServiceUnitConfig {
            service_name: "web".into(),
            description: "My Web App".into(),
            working_dir: "/opt/web".into(),
            exec_start: "/usr/bin/python3 main.py".into(),
            env_file: "/opt/web/.env".into(),
            run_user: "ubuntu".into(),
            restart: RestartPolicy::Always,
            enable_on_boot: true,
            start_immediately: true,
            scope: ServiceScope::System,
        };

        let content = config.generate_unit_content();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("Description=My Web App"));
        assert!(content.contains("User=ubuntu"));
        assert!(content.contains("WorkingDirectory=/opt/web"));
        assert!(content.contains("EnvironmentFile=/opt/web/.env"));
        assert!(content.contains("ExecStart=/usr/bin/python3 main.py"));
        assert!(content.contains("Restart=always"));
        assert!(content.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn generates_user_service_content() {
        let config = ServiceUnitConfig {
            service_name: "my-user-app".into(),
            description: "User Application".into(),
            exec_start: "/usr/bin/node index.js".into(),
            run_user: "ubuntu".into(), // In user mode, this should be omitted
            restart: RestartPolicy::OnFailure,
            scope: ServiceScope::User,
            ..Default::default()
        };

        let content = config.generate_unit_content();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("Description=User Application"));
        // Must NOT contain User= in user mode!
        assert!(!content.contains("User="));
        assert!(content.contains("ExecStart=/usr/bin/node index.js"));
        assert!(content.contains("Restart=on-failure"));
        // Must use default.target for user services!
        assert!(content.contains("WantedBy=default.target"));
        assert!(!content.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn root_user_is_omitted() {
        let config = ServiceUnitConfig {
            service_name: "test".into(),
            exec_start: "/bin/true".into(),
            run_user: "root".into(),
            ..Default::default()
        };
        let content = config.generate_unit_content();
        assert!(!content.contains("User="));
    }

    #[test]
    fn empty_env_file_is_omitted() {
        let config = ServiceUnitConfig {
            service_name: "test".into(),
            exec_start: "/bin/true".into(),
            env_file: "".into(),
            ..Default::default()
        };
        let content = config.generate_unit_content();
        assert!(!content.contains("EnvironmentFile="));
    }
}
