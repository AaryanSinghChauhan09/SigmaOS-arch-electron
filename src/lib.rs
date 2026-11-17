//! Electron-like desktop application framework for SigmaOS.
//!
//! Manages application windows, processes, and lifecycle. Simulates Electron
//! main/renderer architecture without actual WebView/Chromium dependencies.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Errors for Electron operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectronError {
    AppNotFound(String),
    WindowNotFound(u32),
    InvalidConfig(String),
    AlreadyRunning(String),
    NotRunning(String),
}

impl fmt::Display for ElectronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppNotFound(n) => write!(f, "application not found: {n}"),
            Self::WindowNotFound(id) => write!(f, "window not found: {id}"),
            Self::InvalidConfig(m) => write!(f, "invalid config: {m}"),
            Self::AlreadyRunning(n) => write!(f, "application already running: {n}"),
            Self::NotRunning(n) => write!(f, "application not running: {n}"),
        }
    }
}

impl std::error::Error for ElectronError {}

/// Window configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowConfig {
    pub id: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub url: Option<String>,
}

impl WindowConfig {
    pub fn new(id: u32, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            width: 800,
            height: 600,
            visible: true,
            url: None,
        }
    }

    pub fn with_size(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

/// Application configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub name: String,
    pub main_path: PathBuf,
    pub preload_path: Option<PathBuf>,
    pub windows: Vec<WindowConfig>,
    pub single_instance: bool,
}

impl AppConfig {
    pub fn new(name: impl Into<String>, main_path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            main_path: main_path.into(),
            preload_path: None,
            windows: Vec::new(),
            single_instance: false,
        }
    }

    pub fn with_window(mut self, window: WindowConfig) -> Self {
        self.windows.push(window);
        self
    }

    pub fn with_preload(mut self, path: impl Into<PathBuf>) -> Self {
        self.preload_path = Some(path.into());
        self
    }

    pub fn validate(&self) -> Result<(), ElectronError> {
        if self.name.trim().is_empty() {
            return Err(ElectronError::InvalidConfig("name must not be empty".into()));
        }
        if self.windows.is_empty() {
            return Err(ElectronError::InvalidConfig("at least one window required".into()));
        }
        Ok(())
    }
}

/// Window instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub config: WindowConfig,
    pub focused: bool,
    pub destroyed: bool,
}

/// Application instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Application {
    pub config: AppConfig,
    pub running: bool,
    pub windows: HashMap<u32, Window>,
    pub pid: Option<u32>,
}

/// Electron-like application manager.
#[derive(Debug, Clone, Default)]
pub struct ElectronManager {
    apps: HashMap<String, Application>,
    next_pid: u32,
}

impl ElectronManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, config: AppConfig) -> Result<(), ElectronError> {
        config.validate()?;
        if self.apps.contains_key(&config.name) {
            return Err(ElectronError::AlreadyRunning(config.name.clone()));
        }

        let mut windows = HashMap::new();
        for wc in &config.windows {
            windows.insert(wc.id, Window {
                config: wc.clone(),
                focused: false,
                destroyed: false,
            });
        }

        let app = Application {
            config: config.clone(),
            running: false,
            windows,
            pid: None,
        };
        self.apps.insert(config.name.clone(), app);
        Ok(())
    }

    pub fn launch(&mut self, name: &str) -> Result<u32, ElectronError> {
        let app = self.apps.get_mut(name)
            .ok_or_else(|| ElectronError::AppNotFound(name.to_string()))?;
        if app.running {
            return Err(ElectronError::AlreadyRunning(name.to_string()));
        }
        app.running = true;
        self.next_pid += 1;
        app.pid = Some(self.next_pid);
        Ok(self.next_pid)
    }

    pub fn quit(&mut self, name: &str) -> Result<(), ElectronError> {
        let app = self.apps.get_mut(name)
            .ok_or_else(|| ElectronError::AppNotFound(name.to_string()))?;
        if !app.running {
            return Err(ElectronError::NotRunning(name.to_string()));
        }
        app.running = false;
        app.pid = None;
        for window in app.windows.values_mut() {
            window.destroyed = true;
        }
        Ok(())
    }

    pub fn focus_window(&mut self, app_name: &str, window_id: u32) -> Result<(), ElectronError> {
        let app = self.apps.get_mut(app_name)
            .ok_or_else(|| ElectronError::AppNotFound(app_name.to_string()))?;
        if !app.running {
            return Err(ElectronError::NotRunning(app_name.to_string()));
        }

        for (id, window) in app.windows.iter_mut() {
            window.focused = *id == window_id;
        }

        if !app.windows.contains_key(&window_id) {
            return Err(ElectronError::WindowNotFound(window_id));
        }
        Ok(())
    }

    pub fn get_app(&self, name: &str) -> Option<&Application> {
        self.apps.get(name)
    }

    pub fn list_apps(&self) -> Vec<&Application> {
        self.apps.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_app_aaa() {
        // Arrange
        let mut mgr = ElectronManager::new();
        let config = AppConfig::new("test-app", "/app/main.js")
            .with_window(WindowConfig::new(1, "Main"));

        // Act
        let res = mgr.register(config);

        // Assert
        assert!(res.is_ok());
        assert!(mgr.get_app("test-app").is_some());
    }

    #[test]
    fn test_launch_and_quit_aaa() {
        // Arrange
        let mut mgr = ElectronManager::new();
        let config = AppConfig::new("test-app", "/app/main.js")
            .with_window(WindowConfig::new(1, "Main"));
        mgr.register(config).unwrap();

        // Act
        let pid = mgr.launch("test-app").unwrap();
        let running = mgr.get_app("test-app").unwrap().running;
        mgr.quit("test-app").unwrap();

        // Assert
        assert!(pid > 0);
        assert!(running);
        assert!(!mgr.get_app("test-app").unwrap().running);
    }

    #[test]
    fn test_focus_window_aaa() {
        // Arrange
        let mut mgr = ElectronManager::new();
        let config = AppConfig::new("test-app", "/app/main.js")
            .with_window(WindowConfig::new(1, "Main"))
            .with_window(WindowConfig::new(2, "Settings"));
        mgr.register(config).unwrap();
        mgr.launch("test-app").unwrap();

        // Act
        mgr.focus_window("test-app", 2).unwrap();
        let window2 = mgr.get_app("test-app").unwrap().windows.get(&2).unwrap();
        let window1 = mgr.get_app("test-app").unwrap().windows.get(&1).unwrap();

        // Assert
        assert!(window2.focused);
        assert!(!window1.focused);
    }
}
