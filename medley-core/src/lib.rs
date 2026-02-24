//! Medley Core Library
//! 
//! This library provides the core functionality for creating medleys
//! from ProPresenter songs, including templates and arrangement logic.

use serde::{Deserialize, Serialize};

pub use propresenter_parser::ProPresenterSong;

/// Builder for creating medleys from ProPresenter songs
pub struct MedleyBuilder {
    title: String,
    source_songs: Vec<ProPresenterSong>,
    slide_references: Vec<SlideReference>,
    template: Option<MedleyTemplate>,
}

/// Reference to a specific slide in a source song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideReference {
    /// Index of the source song
    pub song_index: usize,
    
    /// Group name in the source song
    pub group_name: String,
    
    /// Slide index within the group
    pub slide_index: usize,
    
    /// Optional custom transition
    pub transition: Option<String>,
}

/// Template for medley styling and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedleyTemplate {
    /// Template name
    pub name: String,
    
    /// Background settings
    pub background: Option<BackgroundSettings>,
    
    /// Text formatting settings
    pub text_format: Option<TextFormatSettings>,
    
    /// Transition settings
    pub transitions: Option<TransitionSettings>,
}

/// Background settings for the medley
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSettings {
    pub color: Option<String>,
    pub image_path: Option<String>,
    pub opacity: Option<f64>,
}

/// Text formatting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormatSettings {
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub color: Option<String>,
    pub shadow: Option<bool>,
    pub outline: Option<bool>,
}

/// Transition settings between slides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionSettings {
    pub default_transition: Option<String>,
    pub transition_duration: Option<f64>,
    pub auto_advance: Option<bool>,
}

/// Complete medley with all its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medley {
    /// Medley title
    pub title: String,
    
    /// Source songs used in the medley
    pub source_songs: Vec<ProPresenterSong>,
    
    /// Ordered list of slide references
    pub slide_references: Vec<SlideReference>,
    
    /// Applied template (if any)
    pub template: Option<MedleyTemplate>,
    
    /// Creation timestamp
    pub created_at: Option<String>,
}

impl MedleyBuilder {
    /// Create a new medley builder with the given title
    pub fn new(title: String) -> Self {
        Self {
            title,
            source_songs: Vec::new(),
            slide_references: Vec::new(),
            template: None,
        }
    }
    
    /// Load a song from a file path
    pub fn load_song<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let song = propresenter_parser::ProPresenterParser::parse_file(path)?;
        self.source_songs.push(song);
        Ok(self)
    }
    
    /// Add a song directly
    pub fn add_song(mut self, song: ProPresenterSong) -> Self {
        self.source_songs.push(song);
        self
    }
    
    /// Apply a template to the medley
    pub fn with_template(mut self, template: MedleyTemplate) -> Self {
        self.template = Some(template);
        self
    }
    
    /// Add a slide reference to the medley
    pub fn add_slide(mut self, song_index: usize, group_name: String, slide_index: usize) -> Self {
        self.slide_references.push(SlideReference {
            song_index,
            group_name,
            slide_index,
            transition: None,
        });
        self
    }
    
    /// Build the final medley
    pub fn build(self) -> Medley {
        Medley {
            title: self.title,
            source_songs: self.source_songs,
            slide_references: self.slide_references,
            template: self.template,
            created_at: Some(
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
            ),
        }
    }
}

/// Template manager for predefined medley templates
pub struct TemplateManager;

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        Self
    }
    
    /// List all available template names
    pub fn list_templates(&self) -> Vec<&'static str> {
        vec!["modern", "classic", "worship"]
    }
    
    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<MedleyTemplate> {
        match name {
            "modern" => Some(self.create_modern_template()),
            "classic" => Some(self.create_classic_template()),
            "worship" => Some(self.create_worship_template()),
            _ => None,
        }
    }
    
    fn create_modern_template(&self) -> MedleyTemplate {
        MedleyTemplate {
            name: "Modern".to_string(),
            background: Some(BackgroundSettings {
                color: Some("#1a1a1a".to_string()),
                image_path: None,
                opacity: Some(1.0),
            }),
            text_format: Some(TextFormatSettings {
                font_family: Some("Helvetica Neue".to_string()),
                font_size: Some(72.0),
                color: Some("#ffffff".to_string()),
                shadow: Some(true),
                outline: Some(false),
            }),
            transitions: Some(TransitionSettings {
                default_transition: Some("fade".to_string()),
                transition_duration: Some(0.5),
                auto_advance: Some(false),
            }),
        }
    }
    
    fn create_classic_template(&self) -> MedleyTemplate {
        MedleyTemplate {
            name: "Classic".to_string(),
            background: Some(BackgroundSettings {
                color: Some("#000080".to_string()),
                image_path: None,
                opacity: Some(1.0),
            }),
            text_format: Some(TextFormatSettings {
                font_family: Some("Times New Roman".to_string()),
                font_size: Some(64.0),
                color: Some("#ffff00".to_string()),
                shadow: Some(false),
                outline: Some(true),
            }),
            transitions: Some(TransitionSettings {
                default_transition: Some("slide".to_string()),
                transition_duration: Some(1.0),
                auto_advance: Some(false),
            }),
        }
    }
    
    fn create_worship_template(&self) -> MedleyTemplate {
        MedleyTemplate {
            name: "Worship".to_string(),
            background: Some(BackgroundSettings {
                color: Some("#2c1810".to_string()),
                image_path: None,
                opacity: Some(0.8),
            }),
            text_format: Some(TextFormatSettings {
                font_family: Some("Open Sans".to_string()),
                font_size: Some(68.0),
                color: Some("#f5f5dc".to_string()),
                shadow: Some(true),
                outline: Some(false),
            }),
            transitions: Some(TransitionSettings {
                default_transition: Some("fade".to_string()),
                transition_duration: Some(0.3),
                auto_advance: Some(false),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_medley_builder() {
        let builder = MedleyBuilder::new("Test Medley".to_string());
        let medley = builder.build();
        assert_eq!(medley.title, "Test Medley");
        assert_eq!(medley.source_songs.len(), 0);
    }
    
    #[test]
    fn test_template_manager() {
        let manager = TemplateManager::new();
        let templates = manager.list_templates();
        assert!(templates.contains(&"modern"));
        assert!(templates.contains(&"classic"));
        assert!(templates.contains(&"worship"));
        
        let modern = manager.get_template("modern").unwrap();
        assert_eq!(modern.name, "Modern");
    }
}