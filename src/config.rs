//! Модуль для управления шаблонами исключений.
//!
//! Предоставляет функциональность для загрузки, кэширования и обновления
//! шаблонов в формате gitignore из внешнего API (toptal.com).
//! Управление конфигурацией и кэшем происходит в директории `~/.flatten/`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const API_LIST_URL: &str = "https://www.toptal.com/developers/gitignore/api/list?format=json";
const API_TEMPLATE_URL_BASE: &str = "https://www.toptal.com/developers/gitignore/api/";

/// Конфигурация менеджера шаблонов.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ManagerConfig {
    /// Временная метка последнего обновления в секундах (Unix time).
    pub last_updated: u64,
    /// Продолжительность хранения кэша в секундах.
    pub cache_duration: u64,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            last_updated: 0,
            cache_duration: 86_400, // 24 часа
        }
    }
}

/// Представление шаблона исключений.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    /// Уникальный ключ шаблона (например, "rust").
    pub key: String,
    /// Имя шаблона.
    pub name: String,
    /// Содержимое шаблона (в формате gitignore).
    pub contents: String,
}

/// Управляет получением, кэшированием и доступом к шаблонам исключений.
#[derive(Debug)]
pub struct TemplateManager {
    config_path: PathBuf,
    templates_path: PathBuf,
    config: ManagerConfig,
    templates: HashMap<String, Template>,
}

impl TemplateManager {
    /// Создает новый экземпляр `TemplateManager`.
    ///
    /// Инициализирует пути, загружает конфигурацию и кэшированные шаблоны.
    ///
    /// # Ошибки
    /// Возвращает ошибку, если не удается определить домашнюю директорию
    /// или создать/прочитать файлы конфигурации.
    ///
    /// # Examples
    /// ```no_run
    /// # use flatten_rust::config::TemplateManager;
    /// # use anyhow::Result;
    /// # async fn example() -> Result<()> {
    /// let mut manager = TemplateManager::new()?;
    /// manager.update_if_needed().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;
        let flatten_dir = home_dir.join(".flatten");

        std::fs::create_dir_all(&flatten_dir).context("Failed to create .flatten directory")?;

        let config_path = flatten_dir.join("manager_config.json");
        let templates_path = flatten_dir.join("templates_cache.json");

        let mut manager = Self {
            config_path,
            templates_path,
            config: ManagerConfig::default(),
            templates: HashMap::new(),
        };

        manager.load_config()?;
        manager.load_templates()?;

        Ok(manager)
    }

    /// Загружает конфигурацию из файла или создает новую, если файл отсутствует.
    fn load_config(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path)
                .context("Failed to read config file")?;
            self.config =
                serde_json::from_str(&content).context("Failed to parse config file")?;
        } else {
            self.save_config()?;
        }
        Ok(())
    }

    /// Сохраняет текущую конфигурацию в файл.
    fn save_config(&self) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self.config).context("Failed to serialize config")?;
        std::fs::write(&self.config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    /// Загружает кэшированные шаблоны из файла.
    fn load_templates(&mut self) -> Result<()> {
        if self.templates_path.exists() {
            let content = std::fs::read_to_string(&self.templates_path)
                .context("Failed to read templates file")?;
            self.templates =
                serde_json::from_str(&content).context("Failed to parse templates file")?;
        }
        Ok(())
    }

    /// Сохраняет текущий набор шаблонов в кэш-файл.
    fn save_templates(&self) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self.templates).context("Failed to serialize templates")?;
        std::fs::write(&self.templates_path, content).context("Failed to write templates file")?;
        Ok(())
    }

    /// Проверяет, истек ли срок действия кэша шаблонов.
    fn needs_update(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        current_time.saturating_sub(self.config.last_updated) > self.config.cache_duration
    }

    /// Загружает шаблоны из API, если кэш устарел.
    pub async fn update_if_needed(&mut self) -> Result<()> {
        if self.needs_update() || self.templates.is_empty() {
            println!("🔄 Updating exclusion templates...");
            if let Err(e) = self.fetch_templates().await {
                eprintln!("Warning: Failed to update templates: {}. Using cached version if available.", e);
            } else {
                println!("✅ Templates updated successfully");
            }
        }
        Ok(())
    }
    
    /// Принудительно обновляет шаблоны из API.
    pub async fn force_update(&mut self) -> Result<()> {
        self.config.last_updated = 0; // Сброс времени для принудительного обновления
        println!("🔄 Force updating exclusion templates...");
        match self.fetch_templates().await {
             Ok(()) => {
                println!("✅ Templates updated successfully");
                Ok(())
             },
             Err(e) => {
                eprintln!("Error: Failed to update templates: {}", e);
                Err(e)
             }
        }
    }

    /// Получает шаблоны из API toptal.com.
    async fn fetch_templates(&mut self) -> Result<()> {
        let client = reqwest::Client::new();
        let list_response = client.get(API_LIST_URL).send().await?.text().await?;
        let template_keys: Vec<&str> = list_response.lines().collect();

        for key in template_keys {
            let template_url = format!("{}{}", API_TEMPLATE_URL_BASE, key);
            match client.get(&template_url).send().await {
                Ok(response) => {
                    if let Ok(content) = response.text().await {
                        let template = Template {
                            key: key.to_string(),
                            name: key.to_string(),
                            contents: content,
                        };
                        self.templates.insert(key.to_string(), template);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to fetch template '{}': {}", key, e);
                }
            }
        }

        self.config.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        self.save_templates()?;
        self.save_config()?;
        Ok(())
    }

    /// Возвращает список ключей всех доступных шаблонов.
    pub fn get_available_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Возвращает содержимое шаблона по его ключу.
    pub fn get_template_contents(&self, key: &str) -> Option<&str> {
        self.templates.get(key).map(|t| t.contents.as_str())
    }
}
