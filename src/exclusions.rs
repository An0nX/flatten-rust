//! Exclusion patterns management module
//!
//! This module provides intelligent exclusion patterns based on
//! gitignore templates and project auto-detection.

use crate::config::GitignoreManager;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Exclusion patterns manager
///
/// Manages intelligent exclusion patterns based on gitignore templates
/// and project auto-detection capabilities
pub struct ExclusionManager {
    gitignore_manager: Arc<RwLock<GitignoreManager>>,
    custom_patterns: HashSet<String>,
    enabled_templates: HashSet<String>,
}

impl ExclusionManager {
    /// Creates a new ExclusionManager instance
    ///
    /// # Errors
    /// Returns an error if the gitignore manager cannot be initialized
    /// or if template updates fail
    ///
    /// # Examples
    /// ```no_run
    /// use flatten_rust::exclusions::ExclusionManager;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let manager = ExclusionManager::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        let mut gitignore_manager = GitignoreManager::new()?;
        
        // Update templates if needed
        gitignore_manager.update_if_needed().await?;
        
        Ok(Self {
            gitignore_manager: Arc::new(RwLock::new(gitignore_manager)),
            custom_patterns: HashSet::new(),
            enabled_templates: HashSet::new(),
        })
    }
    
    /// Enable templates based on project detection
    pub async fn enable_templates_for_project(&mut self, project_path: &Path) -> Result<()> {
        let manager = self.gitignore_manager.read().await;
        let available_templates = manager.get_available_templates();
        
        // Reset enabled templates
        self.enabled_templates.clear();
        
        // Check for each template if its patterns exist in the project
        for template_key in available_templates {
            if self.detect_template_in_project(project_path, template_key).await? {
                self.enabled_templates.insert(template_key.to_string());
            }
        }
        
        Ok(())
    }
    
    /// Detect if a template is relevant for the project
    async fn detect_template_in_project(&self, project_path: &Path, template_key: &str) -> Result<bool> {
        let manager = self.gitignore_manager.read().await;
        
        // Get patterns for this template
        let patterns = manager.get_patterns_for_templates(&[template_key.to_string()]);
        
        // Check if any pattern files/directories exist in the project
        for pattern in patterns {
            if self.pattern_exists_in_project(project_path, &pattern) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Check if a pattern exists in the project
    fn pattern_exists_in_project(&self, project_path: &Path, pattern: &str) -> bool {
        // Simple pattern matching - check for common project files
        match pattern.trim() {
            // File patterns
            p if p.ends_with("Cargo.toml") => project_path.join("Cargo.toml").exists(),
            p if p.ends_with("package.json") => project_path.join("package.json").exists(),
            p if p.ends_with("requirements.txt") => project_path.join("requirements.txt").exists(),
            p if p.ends_with("pom.xml") => project_path.join("pom.xml").exists(),
            p if p.ends_with("build.gradle") => project_path.join("build.gradle").exists(),
            p if p.ends_with("*.csproj") => {
                // Check for any .csproj file
                std::fs::read_dir(project_path)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok())
                            .any(|e| e.file_name().to_string_lossy().ends_with(".csproj"))
                    })
                    .unwrap_or(false)
            }
            p if p.ends_with("go.mod") => project_path.join("go.mod").exists(),
            p if p.ends_with("Gemfile") => project_path.join("Gemfile").exists(),
            p if p.ends_with("composer.json") => project_path.join("composer.json").exists(),
            p if p.ends_with("pubspec.yaml") => project_path.join("pubspec.yaml").exists(),
            
            // Directory patterns
            p if p.contains("node_modules") => project_path.join("node_modules").exists(),
            p if p.contains("target") => project_path.join("target").exists(),
            p if p.contains("__pycache__") => project_path.join("__pycache__").exists(),
            p if p.contains(".gradle") => project_path.join(".gradle").exists(),
            p if p.contains(".vs") => project_path.join(".vs").exists(),
            p if p.contains(".idea") => project_path.join(".idea").exists(),
            
            _ => false,
        }
    }
    
    /// Get all exclusion patterns (folders and files)
    pub async fn get_exclusion_patterns(&self) -> Vec<String> {
        let manager = self.gitignore_manager.read().await;
        let mut patterns = Vec::new();
        
        // Add patterns from enabled templates
        let enabled_templates: Vec<String> = self.enabled_templates.iter().cloned().collect();
        patterns.extend(manager.get_patterns_for_templates(&enabled_templates));
        
        // Add custom patterns
        patterns.extend(self.custom_patterns.iter().cloned());
        
        patterns
    }
    
    /// Get folder exclusion patterns
    pub async fn get_folder_patterns(&self) -> HashSet<String> {
        let all_patterns = self.get_exclusion_patterns().await;
        let mut folder_patterns = HashSet::new();
        
        for pattern in all_patterns {
            // Extract folder names from patterns
            if let Some(folder_name) = self.extract_folder_name(&pattern) {
                folder_patterns.insert(folder_name);
            }
        }
        
        folder_patterns
    }
    
    /// Get file extension exclusion patterns
    pub async fn get_extension_patterns(&self) -> HashSet<String> {
        let all_patterns = self.get_exclusion_patterns().await;
        let mut extension_patterns = HashSet::new();
        
        for pattern in all_patterns {
            // Extract file extensions from patterns
            if let Some(extension) = self.extract_extension(&pattern) {
                extension_patterns.insert(extension);
            }
        }
        
        extension_patterns
    }
    
    /// Extract folder name from pattern
    fn extract_folder_name(&self, pattern: &str) -> Option<String> {
        let pattern = pattern.trim();
        
        // Skip comments and special patterns
        if pattern.starts_with('#') || pattern.starts_with('!') {
            return None;
        }
        
        // Handle directory patterns (ending with /)
        if pattern.ends_with('/') {
            return Some(pattern.trim_end_matches('/').to_string());
        }
        
        // Handle patterns that look like directories
        if !pattern.contains('.') && !pattern.contains('*') {
            return Some(pattern.to_string());
        }
        
        // Extract directory from path patterns
        if let Some(last_slash) = pattern.rfind('/') {
            let dir_part = &pattern[..last_slash];
            if !dir_part.contains('*') && !dir_part.contains('.') {
                return Some(dir_part.to_string());
            }
        }
        
        None
    }
    
    /// Extract file extension from pattern
    fn extract_extension(&self, pattern: &str) -> Option<String> {
        let pattern = pattern.trim();
        
        // Skip comments and special patterns
        if pattern.starts_with('#') || pattern.starts_with('!') {
            return None;
        }
        
        // Handle *.ext patterns
        if pattern.starts_with("*.") && !pattern.contains('/') {
            return Some(pattern.trim_start_matches("*.").to_string());
        }
        
        // Handle patterns ending with specific extensions
        if let Some(dot_pos) = pattern.rfind('.') {
            let after_dot = &pattern[dot_pos + 1..];
            if !after_dot.contains('/') && !after_dot.contains('*') {
                return Some(after_dot.to_string());
            }
        }
        
        None
    }
    
    
    
    /// Get list of enabled templates
    pub fn get_enabled_templates(&self) -> Vec<&str> {
        self.enabled_templates.iter().map(|s| s.as_str()).collect()
    }
    
    /// Enable specific template
    pub fn enable_template(&mut self, template_key: String) {
        self.enabled_templates.insert(template_key);
    }
    
    /// Disable specific template
    pub fn disable_template(&mut self, template_key: &str) -> bool {
        self.enabled_templates.remove(template_key)
    }
    
    /// Force update templates
    pub async fn force_update_templates(&self) -> Result<()> {
        let manager = Arc::clone(&self.gitignore_manager);
        let mut manager = manager.write().await;
        manager.force_update().await
    }
    
    /// Get available templates
    pub async fn get_available_templates(&self) -> Vec<String> {
        let manager = self.gitignore_manager.read().await;
        manager.get_available_templates().into_iter().map(|s| s.to_string()).collect()
    }
    
    
    
    /// Set internet connectivity checking
    pub async fn set_check_internet(&self, check: bool) -> Result<()> {
        let manager = Arc::clone(&self.gitignore_manager);
        let mut manager = manager.write().await;
        manager.set_check_internet(check)
    }
}