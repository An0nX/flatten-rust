//! Модуль для управления логикой исключений.
//!
//! `ExclusionManager` является центральным компонентом, который использует
//! `TemplateManager` для получения шаблонов и применяет их для
//! определения, какие файлы и папки следует исключить из обработки.

use crate::config::TemplateManager;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

/// Управляет логикой исключения файлов и папок.
///
/// Содержит в себе `TemplateManager` для доступа к шаблонам,
/// а также списки включенных шаблонов и пользовательских паттернов.
#[derive(Debug)]
pub struct ExclusionManager {
    template_manager: TemplateManager,
    enabled_templates: HashSet<String>,
}

impl ExclusionManager {
    /// Создает новый `ExclusionManager` и инициализирует `TemplateManager`.
    ///
    /// # Ошибки
    /// Возвращает ошибку, если не удается инициализировать `TemplateManager`.
    ///
    /// # Examples
    /// ```no_run
    /// # use flatten_rust::exclusions::ExclusionManager;
    /// # use anyhow::Result;
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let manager = ExclusionManager::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        let mut template_manager = TemplateManager::new()?;
        template_manager.update_if_needed().await?;

        Ok(Self {
            template_manager,
            enabled_templates: HashSet::new(),
        })
    }
    
    /// Автоматически включает шаблоны, релевантные для указанного проекта.
    ///
    /// Определяет тип проекта по наличию характерных файлов (например, `Cargo.toml`).
    pub async fn enable_templates_for_project(&mut self, project_path: &Path) -> Result<()> {
        let detection_map = Self::get_detection_map();
        for (template_key, file_indicators) in detection_map {
            for indicator in file_indicators {
                if project_path.join(indicator).exists() {
                    self.enabled_templates.insert(template_key.to_string());
                    break;
                }
            }
        }
        Ok(())
    }

    /// Возвращает карту для определения типов проектов.
    fn get_detection_map() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("rust", vec!["Cargo.toml"]),
            ("node", vec!["package.json"]),
            ("python", vec!["requirements.txt", "pyproject.toml"]),
            ("java", vec!["pom.xml", "build.gradle"]),
            ("go", vec!["go.mod"]),
            ("ruby", vec!["Gemfile"]),
            ("php", vec!["composer.json"]),
        ]
    }
    
    /// Возвращает все паттерны из включенных шаблонов.
    pub fn get_all_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        for key in &self.enabled_templates {
            if let Some(contents) = self.template_manager.get_template_contents(key) {
                patterns.extend(Self::parse_ignore_patterns(contents));
            }
        }
        patterns
    }

    /// Парсит содержимое шаблона, возвращая список паттернов.
    fn parse_ignore_patterns(content: &str) -> Vec<String> {
        content
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Возвращает набор паттернов для исключения папок.
    pub async fn get_folder_patterns(&self) -> HashSet<String> {
        self.get_all_patterns()
            .iter()
            .filter_map(|p| Self::extract_folder_name(p))
            .collect()
    }

    /// Возвращает набор паттернов для исключения файлов по расширению.
    pub async fn get_extension_patterns(&self) -> HashSet<String> {
        self.get_all_patterns()
            .iter()
            .filter_map(|p| Self::extract_extension(p))
            .collect()
    }

    /// Извлекает имя папки из паттерна.
    fn extract_folder_name(pattern: &str) -> Option<String> {
        let p = pattern.trim_end_matches('/');
        if !p.contains('*') && !p.contains('.') {
            return Some(p.to_string());
        }
        None
    }

    /// Извлекает расширение файла из паттерна.
    fn extract_extension(pattern: &str) -> Option<String> {
        if pattern.starts_with("*.") {
            return Some(pattern.trim_start_matches("*.").to_string());
        }
        None
    }

    /// Возвращает список включенных шаблонов.
    pub fn get_enabled_templates(&self) -> Vec<&str> {
        self.enabled_templates.iter().map(|s| s.as_str()).collect()
    }

    /// Включает шаблон по ключу.
    pub fn enable_template(&mut self, template_key: String) {
        self.enabled_templates.insert(template_key);
    }

    /// Отключает шаблон по ключу.
    pub fn disable_template(&mut self, template_key: &str) {
        self.enabled_templates.remove(template_key);
    }

    /// Принудительно обновляет шаблоны через `TemplateManager`.
    pub async fn force_update_templates(&mut self) -> Result<()> {
        self.template_manager.force_update().await
    }

    /// Возвращает список всех доступных шаблонов.
    pub async fn get_available_templates(&self) -> Vec<String> {
        self.template_manager.get_available_templates()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ignore_patterns() {
        let content = "
# Comment
target/
*.log
file.txt
        ";
        let patterns = ExclusionManager::parse_ignore_patterns(content);
        assert_eq!(patterns, vec!["target/", "*.log", "file.txt"]);
    }

    #[test]
    fn test_extract_folder_name() {
        assert_eq!(ExclusionManager::extract_folder_name("target/"), Some("target".to_string()));
        assert_eq!(ExclusionManager::extract_folder_name("node_modules"), Some("node_modules".to_string()));
        assert_eq!(ExclusionManager::extract_folder_name("*.log"), None);
        assert_eq!(ExclusionManager::extract_folder_name("file.txt"), None);
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(ExclusionManager::extract_extension("*.log"), Some("log".to_string()));
        assert_eq!(ExclusionManager::extract_extension("*.pyc"), Some("pyc".to_string()));
        assert_eq!(ExclusionManager::extract_extension("target/"), None);
        assert_eq!(ExclusionManager::extract_extension("file.txt"), None);
    }
}
