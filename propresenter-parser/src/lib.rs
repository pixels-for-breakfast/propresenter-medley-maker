//! ProPresenter File Parser Library
//! 
//! This library provides functionality to parse ProPresenter (.pro) files
//! and extract song data including slides, groups, and metadata.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Represents a complete ProPresenter song/presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProPresenterSong {
    /// Song title
    pub title: String,
    
    /// Unique identifier for the song
    pub uuid: String,
    
    /// Version information (optional)
    pub version: Option<String>,
    
    /// Collection of slide groups (verses, chorus, etc.)
    pub slide_groups: Vec<SlideGroup>,
    
    /// Media assets referenced in the presentation
    pub media_assets: Vec<MediaAsset>,
    
    /// Song metadata (author, copyright, etc.)
    pub metadata: SongMetadata,
}

/// A group of slides (e.g., "Verse 1", "Chorus", "Bridge")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideGroup {
    /// Group name (e.g., "Verse 1", "Chorus")
    pub name: String,
    
    /// Unique identifier for this group
    pub uuid: String,
    
    /// Slides within this group
    pub slides: Vec<Slide>,
    
    /// Color/styling information
    pub color: Option<String>,
}

/// Individual slide within a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    /// Unique identifier for this slide
    pub uuid: String,
    
    /// Text elements on this slide
    pub text_elements: Vec<TextElement>,
    
    /// Plain text content (for search/analysis)
    pub plain_text: Option<String>,
    
    /// Background information
    pub background: Option<SlideBackground>,
}

/// Text element on a slide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElement {
    /// Unique identifier
    pub uuid: String,
    
    /// Text content (may include RTF formatting)
    pub text: String,
    
    /// Plain text version
    pub plain_text: String,
    
    /// Position and sizing
    pub bounds: ElementBounds,
    
    /// Text formatting
    pub formatting: TextFormatting,
}

/// Position and size information for elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Text formatting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormatting {
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub color: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

/// Background information for slides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideBackground {
    pub background_type: BackgroundType,
    pub color: Option<String>,
    pub image_path: Option<String>,
}

/// Type of slide background
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundType {
    Color,
    Image,
    Video,
    None,
}

/// Media asset (images, videos, audio)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    /// Asset UUID
    pub uuid: String,
    
    /// File path
    pub file_path: String,
    
    /// Relative path (if different from file_path)
    pub relative_path: Option<String>,
    
    /// Type of media
    pub media_type: MediaType,
    
    /// File size in bytes
    pub file_size: Option<u64>,
}

/// Type of media asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Unknown,
}

/// Song metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongMetadata {
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub ccli_number: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

/// Main parser for ProPresenter files
pub struct ProPresenterParser;

impl ProPresenterParser {
    /// Parse a ProPresenter file from the given path
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<ProPresenterSong, Box<dyn std::error::Error>> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        
        // Read the entire file into memory for parsing
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        
        Self::parse_binary(&buffer, &path)
    }
    
    /// Parse binary ProPresenter data
    fn parse_binary<P: AsRef<Path>>(data: &[u8], path: P) -> Result<ProPresenterSong, Box<dyn std::error::Error>> {
        let path_ref = path.as_ref();
        let filename = path_ref.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        // For now, create a basic structure with extracted data
        // TODO: Implement proper binary format parsing
        
        let (title, uuid, groups) = Self::extract_basic_info(data, &filename)?;
        
        Ok(ProPresenterSong {
            title,
            uuid,
            version: Some("7.0".to_string()), // Default assumption
            slide_groups: groups,
            media_assets: Vec::new(), // TODO: Extract media assets
            metadata: SongMetadata {
                author: None,
                copyright: None,
                ccli_number: None,
                key: None,
                tempo: None,
                notes: None,
                tags: Vec::new(),
            },
        })
    }
    
    /// Extract basic information from binary data
    fn extract_basic_info(data: &[u8], fallback_title: &str) -> Result<(String, String, Vec<SlideGroup>), Box<dyn std::error::Error>> {
        let data_str = String::from_utf8_lossy(data);
        
        // For now, use filename as title since binary parsing is unreliable
        let title = Self::clean_filename_title(fallback_title);
        
        // Extract main UUID (usually first one)
        let uuid = Self::extract_first_uuid(&data_str)
            .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());
        
        // Extract slide groups
        let groups = Self::extract_slide_groups(&data_str)?;
        
        Ok((title, uuid, groups))
    }
    
    /// Clean up filename to make a proper title
    fn clean_filename_title(filename: &str) -> String {
        filename
            .replace(".pro", "")           // Remove extension
            .replace("_", " ")             // Underscores to spaces
            .replace("-", " ")             // Hyphens to spaces
            .split_whitespace()            // Normalize whitespace
            .collect::<Vec<_>>()
            .join(" ")
    }
    

    
    /// Extract first UUID from data
    fn extract_first_uuid(data_str: &str) -> Option<String> {
        use regex::Regex;
        let uuid_regex = Regex::new(r"[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}").ok()?;
        
        uuid_regex.find(data_str).map(|m| m.as_str().to_string())
    }
    
    /// Extract slide groups from binary data
    fn extract_slide_groups(data_str: &str) -> Result<Vec<SlideGroup>, Box<dyn std::error::Error>> {
        let mut groups = Vec::new();
        
        // Look for common slide group names
        let group_patterns = [
            "Verse", "Chorus", "Bridge", "Intro", "Outro", "Tag", 
            "Pre-Chorus", "Refrain", "Instrumental", "Solo"
        ];
        
        for pattern in &group_patterns {
            if data_str.contains(pattern) {
                // Found a group, try to extract it
                let group = SlideGroup {
                    name: format!("{} 1", pattern), // Default numbering
                    uuid: format!("extracted-{}-{}", pattern.to_lowercase(), groups.len()),
                    slides: vec![
                        Slide {
                            uuid: format!("slide-{}-1", pattern.to_lowercase()),
                            text_elements: Vec::new(),
                            plain_text: Some(format!("[{} content]", pattern)),
                            background: None,
                        }
                    ],
                    color: None,
                };
                groups.push(group);
            }
        }
        
        // If no groups found, create a default one
        if groups.is_empty() {
            groups.push(SlideGroup {
                name: "Content".to_string(),
                uuid: "default-content-group".to_string(),
                slides: vec![
                    Slide {
                        uuid: "default-slide-1".to_string(),
                        text_elements: Vec::new(),
                        plain_text: Some("[Song content - parsing in progress]".to_string()),
                        background: None,
                    }
                ],
                color: None,
            });
        }
        
        Ok(groups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uuid_extraction() {
        let test_data = "some data 6F889546-A201-4B99-928D-F0D45251162E more data";
        let uuid = ProPresenterParser::extract_first_uuid(test_data);
        assert_eq!(uuid, Some("6F889546-A201-4B99-928D-F0D45251162E".to_string()));
    }
    
    #[test]
    fn test_slide_groups() {
        let test_data = "some data Verse 1 content Chorus content";
        let groups = ProPresenterParser::extract_slide_groups(test_data).unwrap();
        assert!(!groups.is_empty());
        assert!(groups.iter().any(|g| g.name.contains("Verse")));
        assert!(groups.iter().any(|g| g.name.contains("Chorus")));
    }
    
    #[test]
    fn test_filename_title_cleaning() {
        assert_eq!(ProPresenterParser::clean_filename_title("Way_Maker.pro"), "Way Maker");
        assert_eq!(ProPresenterParser::clean_filename_title("Another-In-The-Fire.pro"), "Another In The Fire");
    }
}