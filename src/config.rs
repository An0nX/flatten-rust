//! Gitignore templates management module
//!
//! This module provides functionality for managing gitignore templates
//! from the toptal.com API with caching and user overrides.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for gitignore templates
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitignoreConfig {
    /// Last update timestamp
    pub last_updated: u64,
    /// Cache duration in seconds (default: 24 hours)
    pub cache_duration: u64,
    /// Custom user overrides
    pub user_overrides: HashMap<String, Vec<String>>,
    /// Whether to check for internet connectivity
    pub check_internet: bool,
}

impl Default for GitignoreConfig {
    fn default() -> Self {
        Self {
            last_updated: 0,
            cache_duration: 86400, // 24 hours
            user_overrides: HashMap::new(),
            check_internet: true,
        }
    }
}

/// Gitignore template data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitignoreTemplate {
    pub key: String,
    pub name: String,
    pub contents: String,
    pub file_name: String,
}

/// Gitignore templates manager
pub struct GitignoreManager {
    config_path: PathBuf,
    templates_path: PathBuf,
    config: GitignoreConfig,
    templates: HashMap<String, GitignoreTemplate>,
}

impl GitignoreManager {
    /// Creates a new GitignoreManager instance
    ///
    /// # Errors
    /// Returns an error if the home directory cannot be determined or
    /// if the configuration files cannot be created/loaded
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Could not determine home directory")?;
        let flatten_dir = home_dir.join(".flatten");
        
        // Create .flatten directory if it doesn't exist
        if !flatten_dir.exists() {
            std::fs::create_dir_all(&flatten_dir)
                .context("Failed to create .flatten directory")?;
        }
        
        let config_path = flatten_dir.join("config.json");
        let templates_path = flatten_dir.join("templates.json");
        
        let mut manager = Self {
            config_path,
            templates_path,
            config: GitignoreConfig::default(),
            templates: HashMap::new(),
        };
        
        // Load existing config or create default
        manager.load_config()?;
        
        // Load templates
        manager.load_templates()?;
        
        Ok(manager)
    }
    
    /// Check if internet connectivity is available
    ///
    /// # Examples
    /// ```no_run
    /// use flatten_rust::config::GitignoreManager;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let manager = GitignoreManager::new()?;
    /// if manager.check_internet_connectivity().await {
    ///     println!("Internet is available");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_internet_connectivity(&self) -> bool {
        // Try to connect to a reliable, fast service
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();
            
        match client {
            Ok(client) => {
                match client.get("https://www.google.com/generate_204").send().await {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
    
    /// Load configuration from file
    fn load_config(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path)
                .context("Failed to read config file")?;
            self.config = serde_json::from_str(&content)
                .context("Failed to parse config file")?;
        } else {
            // Save default config
            self.save_config()?;
        }
        Ok(())
    }
    
    /// Save configuration to file
    fn save_config(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;
        std::fs::write(&self.config_path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
    
    /// Load templates from file
    fn load_templates(&mut self) -> Result<()> {
        if self.templates_path.exists() {
            let content = std::fs::read_to_string(&self.templates_path)
                .context("Failed to read templates file")?;
            self.templates = serde_json::from_str(&content)
                .context("Failed to parse templates file")?;
        }
        Ok(())
    }
    
    /// Save templates to file
    fn save_templates(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.templates)
            .context("Failed to serialize templates")?;
        std::fs::write(&self.templates_path, content)
            .context("Failed to write templates file")?;
        Ok(())
    }
    
    /// Check if templates need update
    fn needs_update(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        current_time.saturating_sub(self.config.last_updated) > self.config.cache_duration
    }
    
    /// Fetch templates from API
    async fn fetch_templates(&mut self) -> Result<()> {
        let client = reqwest::Client::new();
        
        // Get list of available templates
        let list_url = "https://www.toptal.com/developers/gitignore/api/list?format=json";
        let list_response = client.get(list_url)
            .send()
            .await
            .context("Failed to fetch template list")?;
            
        let template_list: HashMap<String, serde_json::Value> = list_response
            .json()
            .await
            .context("Failed to parse template list")?;
        
        // Fetch each template
        for (key, _) in template_list {
            let template_url = format!("https://www.toptal.com/developers/gitignore/api/{}", key);
            
            match client.get(&template_url).send().await {
                Ok(response) => {
                    if let Ok(content) = response.text().await {
                        let template = GitignoreTemplate {
                            key: key.clone(),
                            name: key.clone(), // Simple name for now
                            contents: content,
                            file_name: format!("{}.gitignore", key),
                        };
                        self.templates.insert(key, template);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to fetch template {}: {}", key, e);
                }
            }
            
            // Small delay to be respectful to the API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // Update timestamp
        self.config.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Save updated templates and config
        self.save_templates()?;
        self.save_config()?;
        
        Ok(())
    }
    
    /// Update templates if needed
    ///
    /// # Examples
    /// ```no_run
    /// use flatten_rust::config::GitignoreManager;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut manager = GitignoreManager::new()?;
    /// manager.update_if_needed().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_if_needed(&mut self) -> Result<()> {
        if self.needs_update() {
            println!("🔄 Updating gitignore templates...");
            
            // Check internet connectivity if configured
            if self.config.check_internet && !self.check_internet_connectivity().await {
                println!("⚠️  No internet connection available, using cached templates");
                return Ok(());
            }
            
            if let Err(e) = self.fetch_templates().await {
                eprintln!("Warning: Failed to update templates: {}", e);
                // Continue with cached templates if available
                if self.templates.is_empty() {
                    return Err(e);
                }
            } else {
                println!("✅ Templates updated successfully");
            }
        }
        Ok(())
    }
    
    /// Get all available template keys
    pub fn get_available_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|k| k.as_str()).collect()
    }
    
    /// Get patterns for specific templates
    pub fn get_patterns_for_templates(&self, template_keys: &[String]) -> Vec<String> {
        let mut patterns = Vec::new();
        
        for key in template_keys {
            if let Some(template) = self.templates.get(key) {
                // Parse gitignore patterns from template content
                let template_patterns = self.parse_gitignore_patterns(&template.contents);
                patterns.extend(template_patterns);
            }
        }
        
        // Add user overrides
        for override_patterns in self.config.user_overrides.values() {
            patterns.extend(override_patterns.clone());
        }
        
        patterns
    }
    
    /// Parse gitignore patterns from template content
    fn parse_gitignore_patterns(&self, content: &str) -> Vec<String> {
        content
            .lines()
            .filter(|line| {
                // Skip empty lines and comments
                !line.trim().is_empty() && !line.trim().starts_with('#') && !line.trim().starts_with("###")
            })
            .map(|line| line.trim().to_string())
            .collect()
    }
    
    
    
    /// Set internet connectivity checking
    ///
    /// # Arguments
    /// * `check` - Whether to check for internet connectivity
    ///
    /// # Examples
    /// ```no_run
    /// use flatten_rust::config::GitignoreManager;
    /// 
    /// # fn main() -> anyhow::Result<()> {
    /// let mut manager = GitignoreManager::new()?;
    /// manager.set_check_internet(false)?; // Disable internet checks
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_check_internet(&mut self, check: bool) -> Result<()> {
        self.config.check_internet = check;
        self.save_config()?;
        Ok(())
    }
    
    /// Force update templates
    ///
    /// # Examples
    /// ```no_run
    /// use flatten_rust::config::GitignoreManager;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut manager = GitignoreManager::new()?;
    /// manager.force_update().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn force_update(&mut self) -> Result<()> {
        self.config.last_updated = 0; // Force update
        self.update_if_needed().await
    }
}