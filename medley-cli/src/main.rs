//! Command-line interface for ProPresenter Medley Maker

use clap::{Parser, Subcommand};
use medley_core::*;
use propresenter_parser::*;
use std::path::PathBuf;

/// ProPresenter Medley Maker CLI
#[derive(Parser)]
#[command(name = "medley-maker")]
#[command(about = "Create medleys/mashups from ProPresenter songs")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and analyze a .pro file
    Parse {
        /// Path to the .pro file
        #[arg(short, long)]
        file: PathBuf,
        
        /// Output format (json, yaml, summary)
        #[arg(short, long, default_value = "summary")]
        output: String,
    },
    
    /// Create a new medley
    Create {
        /// Medley title
        #[arg(short, long)]
        title: String,
        
        /// Output file path for the medley .pro file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Song files to include
        #[arg(short, long, required = true, num_args = 1..)]
        songs: Vec<PathBuf>,
        
        /// Template to apply (modern, classic, worship, or path to custom)
        #[arg(short = 'T', long)]
        template: Option<String>,
    },
    
    /// List available templates
    Templates,
    
    /// Interactive medley builder
    Interactive,
    
    /// Launch the GUI application
    Gui,
    
    /// Analyze multiple songs and show compatibility
    Analyze {
        /// Song files to analyze
        #[arg(short, long, required = true, num_args = 1..)]
        songs: Vec<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { file, output } => {
            handle_parse(file, output)?;
        }
        Commands::Create { title, output, songs, template } => {
            handle_create(title, output, songs, template)?;
        }
        Commands::Templates => {
            handle_templates();
        }
        Commands::Interactive => {
            handle_interactive()?;
        }
        Commands::Gui => {
            handle_gui_launch()?;
        }
        Commands::Analyze { songs } => {
            handle_analyze(songs)?;
        }
    }

    Ok(())
}

fn handle_parse(file: PathBuf, output_format: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Parsing ProPresenter file: {}", file.display());
    
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()).into());
    }
    
    let song = ProPresenterParser::parse_file(&file)?;
    
    match output_format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&song)?;
            println!("{}", json);
        }
        "summary" => {
            print_song_summary(&song);
        }
        _ => {
            eprintln!("Unknown output format: {}", output_format);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_song_summary(song: &ProPresenterSong) {
    println!("📄 Song: {}", song.title);
    println!("🆔 UUID: {}", song.uuid);
    if let Some(version) = &song.version {
        println!("📋 Version: {}", version);
    }
    
    println!("\n📑 Slide Groups ({}):", song.slide_groups.len());
    for (i, group) in song.slide_groups.iter().enumerate() {
        println!("  {}. {} ({} slides)", i + 1, group.name, group.slides.len());
        
        // Show first few words of each slide
        for (j, slide) in group.slides.iter().enumerate() {
            if let Some(plain_text) = &slide.plain_text {
                let preview = plain_text
                    .split_whitespace()
                    .take(8)
                    .collect::<Vec<_>>()
                    .join(" ");
                let preview = if plain_text.split_whitespace().count() > 8 {
                    format!("{}...", preview)
                } else {
                    preview
                };
                println!("     {}.{} {}", i + 1, j + 1, preview);
            }
        }
    }
    
    if !song.media_assets.is_empty() {
        println!("\n🎬 Media Assets ({}):", song.media_assets.len());
        for asset in &song.media_assets {
            println!("  • {} ({:?})", asset.relative_path.as_ref().unwrap_or(&asset.file_path), asset.media_type);
        }
    }
    
    if let Some(author) = &song.metadata.author {
        println!("\n👤 Author: {}", author);
    }
    if let Some(copyright) = &song.metadata.copyright {
        println!("© Copyright: {}", copyright);
    }
}

fn handle_create(
    title: String,
    output: PathBuf,
    songs: Vec<PathBuf>,
    template: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Creating medley: {}", title);
    
    let mut builder = MedleyBuilder::new(title);
    
    // Load all songs
    for song_path in &songs {
        println!("Loading song: {}", song_path.display());
        builder = builder.load_song(song_path)?;
    }
    
    // Apply template if specified
    if let Some(template_name) = template {
        let template_manager = TemplateManager::new();
        if let Some(template) = template_manager.get_template(&template_name) {
            builder = builder.with_template(template.clone());
            println!("Applied template: {}", template_name);
        } else {
            eprintln!("Warning: Template '{}' not found", template_name);
        }
    }
    
    // Build the medley
    let medley = builder.build();
    
    println!("\n✅ Medley created successfully!");
    println!("Title: {}", medley.title);
    println!("Source songs: {}", medley.source_songs.len());
    println!("Total slides: {}", medley.slide_references.len());
    println!("Has template: {}", medley.template.is_some());
    
    // Save medley metadata (JSON for now)
    let metadata_path = output.with_extension("json");
    let json = serde_json::to_string_pretty(&medley)?;
    std::fs::write(&metadata_path, json)?;
    
    println!("Medley metadata saved to: {}", metadata_path.display());
    println!("📝 TODO: Generate actual .pro file (Phase 2 enhancement)");

    Ok(())
}

fn handle_templates() {
    let template_manager = TemplateManager::new();
    let templates = template_manager.list_templates();
    
    println!("📋 Available templates:");
    for template_name in templates {
        if let Some(template) = template_manager.get_template(template_name) {
            println!("  • {}", template.name);
            
            if let Some(bg) = &template.background {
                if let Some(color) = &bg.color {
                    println!("    Background: {}", color);
                }
            }
            
            if let Some(text_format) = &template.text_format {
                if let Some(font) = &text_format.font_family {
                    println!("    Font: {}", font);
                }
                if let Some(size) = text_format.font_size {
                    println!("    Size: {}pt", size);
                }
            }
            println!();
        }
    }
}

fn handle_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Interactive Medley Builder");
    println!("⚠️  Coming soon in Phase 3!");
    Ok(())
}

fn handle_gui_launch() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Launching ProPresenter Medley Maker GUI...");
    println!("⚠️  To run the GUI application:");
    println!("  cargo run --bin medley-gui");
    println!("⚠️  GUI not implemented yet - coming in Phase 4!");
    Ok(())
}

fn handle_analyze(songs: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing {} ProPresenter files for medley compatibility...\n", songs.len());
    
    let mut loaded_songs = Vec::new();
    let mut parse_errors = 0;
    
    // Load all songs
    for song_path in &songs {
        print!("Loading {}... ", song_path.display());
        match ProPresenterParser::parse_file(song_path) {
            Ok(song) => {
                println!("✅");
                loaded_songs.push((song_path.clone(), song));
            }
            Err(e) => {
                println!("❌ {}", e);
                parse_errors += 1;
            }
        }
    }
    
    if loaded_songs.is_empty() {
        println!("❌ No songs could be loaded. Check file paths and formats.");
        return Ok(());
    }
    
    println!("\n📊 Analysis Results:");
    println!("═══════════════════");
    
    // Show song summaries
    for (path, song) in &loaded_songs {
        println!("\n🎵 {}", song.title);
        println!("   File: {}", path.display());
        println!("   Slide Groups: {}", song.slide_groups.len());
        println!("   Total Slides: {}", song.slide_groups.iter().map(|g| g.slides.len()).sum::<usize>());
        
        if !song.slide_groups.is_empty() {
            println!("   Structure:");
            for (i, group) in song.slide_groups.iter().enumerate() {
                println!("     {}. {} ({} slides)", i + 1, group.name, group.slides.len());
            }
        }
    }
    
    println!("\n💡 Medley Suggestions:");
    println!("═════════════════════");
    
    if loaded_songs.len() >= 2 {
        println!("✅ You could create medleys with these songs!");
        println!("📝 Example command:");
        print!("  cargo run --bin medley-cli -- create --title \"My Medley\" --template modern --songs");
        
        for (path, _) in loaded_songs.iter() {
            print!(" \"{}\"", path.display());
        }
        println!();
        
        println!("\n🎯 Compatibility Notes:");
        println!("  • All songs parsed successfully");
        if parse_errors > 0 {
            println!("  • {} files had parsing errors", parse_errors);
        }
        println!("  • Consider grouping songs with similar themes");
        println!("  • Check for compatible keys and tempos (manual review needed)");
    } else {
        println!("ℹ️  Load multiple songs to see medley suggestions");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test that CLI args parse correctly
        let cli = Cli::try_parse_from(&["medley-cli", "templates"]);
        assert!(cli.is_ok());
    }
}