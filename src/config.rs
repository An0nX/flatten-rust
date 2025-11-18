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

/// Вспомогательная структура для парсинга ответа от Toptal API.
#[derive(Debug, Deserialize)]
struct ToptalEntry {
    name: String,
    #[serde(default)]
    contents: String,
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
            // Если конфиг поврежден, используем дефолтный, не падаем
            self.config = serde_json::from_str(&content).unwrap_or_default();
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
            // Если кэш поврежден, инициализируем пустой картой
            self.templates = serde_json::from_str(&content).unwrap_or_default();
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
        
        // Если шаблонов нет совсем, обновление обязательно нужно
        if self.templates.is_empty() {
            return true;
        }

        // Иначе проверяем TTL
        current_time.saturating_sub(self.config.last_updated) > self.config.cache_duration
    }

    /// Загружает шаблоны из API, если кэш устарел или отсутствует.
    ///
    /// # Логика обновления
    /// 1. Если кэш актуален -> ничего не делаем.
    /// 2. Если кэш устарел, пробуем скачать.
    /// 3. Если скачивание не удалось, но есть старый кэш -> используем его (soft fail).
    /// 4. Если шаблонов нет вообще и скачивание не удалось -> возвращаем ошибку.
    pub async fn update_if_needed(&mut self) -> Result<()> {
        if !self.needs_update() {
            return Ok(());
        }

        // Пытаемся обновить
        match self.fetch_templates().await {
            Ok(()) => {
                // Успех, ничего не пишем в консоль, чтобы не спамить
            },
            Err(e) => {
                if self.templates.is_empty() {
                    // Критическая ошибка: нет ни кэша, ни сети
                    return Err(e.context("Failed to fetch initial templates and cache is empty"));
                } else {
                    // Не критично, используем кэш. Ошибку сети можно было бы залогировать в debug.
                    // Для пользователя это должно быть прозрачно.
                }
            }
        }
        Ok(())
    }
    
    /// Принудительно обновляет шаблоны из API.
    pub async fn force_update(&mut self) -> Result<()> {
        println!("🔄 Force updating exclusion templates from API...");
        match self.fetch_templates().await {
            Ok(_) => {
                println!("✅ Templates updated successfully");
                Ok(())
            }
            Err(e) => {
                // При явном обновлении ошибку нужно показать
                Err(e)
            }
        }
    }

    /// Получает шаблоны из API toptal.com.
    ///
    /// Использует endpoint `list?format=json`, который возвращает полный список
    /// шаблонов с их содержимым, что позволяет избежать N+1 запросов.
    async fn fetch_templates(&mut self) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let response = client.get(API_LIST_URL)
            .send()
            .await
            .context("Failed to connect to templates API")?;

        // Парсим ответ как HashMap, где ключ - ID шаблона, значение - структура с содержимым
        // Это решает проблему неправильного парсинга строк
        let api_data: HashMap<String, ToptalEntry> = response
            .json()
            .await
            .context("Failed to parse templates JSON")?;

        if api_data.is_empty() {
            return Err(anyhow::anyhow!("Received empty templates list from API"));
        }

        // Конвертируем во внутренний формат
        self.templates = api_data
            .into_iter()
            .map(|(key, entry)| {
                (
                    key.clone(),
                    Template {
                        key,
                        name: entry.name,
                        contents: entry.contents,
                    },
                )
            })
            .collect();

        // Обновляем метку времени только при успехе
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
