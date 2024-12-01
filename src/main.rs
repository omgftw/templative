use clap::Parser;
use std::path::{Path, PathBuf};
#[derive(serde::Deserialize, Debug)]
struct PathRewrite {
    from: String,
    to: String,
}

#[derive(serde::Deserialize, Debug)]
struct Config {
    path_rewrites: Vec<PathRewrite>,
}

fn read_config(path: &str) -> eyre::Result<Config> {
    let config = std::fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&config)?;
    Ok(config)
}

#[derive(clap::Parser, Debug)]
struct Args {
    /// Path to process
    path: String,
    
    /// Additional dynamic key-value pairs
    #[clap(trailing_var_arg = true)]
    #[clap(allow_hyphen_values = true)]
    #[clap(value_parser)]
    dynamic: Vec<String>,

    #[clap(long)]
    output: Option<String>,

    #[clap(long)]
    config: Option<String>,
}

fn process_path(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &Path, output_path: &str) -> eyre::Result<()> {
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "tmpl") {
            process_file(path.to_str().unwrap(), dynamic_pairs, config, root_path, output_path)?;
        }
        if path.is_file() && path.extension().map_or(false, |ext| ext == "tmpl_chunk") {
            process_chunk(path.to_str().unwrap(), dynamic_pairs, config, root_path)?;
        }
    }
    Ok(())
}

fn process_file(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &Path, output_path: &str) -> eyre::Result<()> {
    let template_content = std::fs::read_to_string(path)?;
    let handlebars = handlebars::Handlebars::new();
    // Create template data from dynamic pairs
    let mut data = serde_json::Map::new();
    for (key, value) in dynamic_pairs {
        data.insert(key.clone(), serde_json::Value::String(value.clone()));
    }
    
    // Render template
    let rendered = handlebars.render_template(&template_content, &data)?;
    
    // Get output path by removing .tmpl extension and applying path rewrites
    let mut file_path = path.strip_suffix(".tmpl").unwrap().to_string();
    for rewrite in &config.path_rewrites {
        file_path = file_path.replace(&rewrite.from, &rewrite.to);
    }
    
    // Convert the path to be relative to root_path
    let relative_path = Path::new(&file_path)
        .strip_prefix(root_path)
        .unwrap_or(Path::new(&file_path));
    
    // Combine output_path with the relative path
    let final_output_path = Path::new(output_path).join(relative_path);
    
    // Ensure output directory exists
    if let Some(parent) = final_output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Write rendered content
    std::fs::write(final_output_path, rendered)?;
    
    Ok(())
}

fn process_chunk(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &Path) -> eyre::Result<()> {
    // Read the chunk template content
    let template_content = std::fs::read_to_string(path)?;
    let handlebars = handlebars::Handlebars::new();

    // Create template data from dynamic pairs
    let mut data = serde_json::Map::new();
    for (key, value) in dynamic_pairs {
        data.insert(key.clone(), serde_json::Value::String(value.clone()));
    }
    
    // Render the chunk template
    let rendered_chunk = handlebars.render_template(&template_content, &data)?;
    
    // Get the target file path by removing .tmpl_* extension and applying rewrites
    let file_path = Path::new(path);
    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    let chunk_id = file_name.split("tmpl_").nth(1).unwrap();
    let mut target_path = path[..path.len() - chunk_id.len() - 5].to_string(); // Remove .tmpl_chunk_id
    
    // Apply path rewrites
    for rewrite in &config.path_rewrites {
        target_path = target_path.replace(&rewrite.from, &rewrite.to);
    }
    
    // Read the target file content
    let file_content = std::fs::read_to_string(&target_path)?;
    
    // Split the content into lines
    let lines: Vec<&str> = file_content.lines().collect();
    
    // Find the insertion point and build new content
    let mut new_content = String::new();
    for line in lines {
        if line.contains(chunk_id) {
            new_content.push_str(&rendered_chunk);
            new_content.push('\n');
        }
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    // Write the modified content back to the file
    std::fs::write(target_path, new_content)?;
    
    Ok(())
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    // Convert the raw args into key-value pairs
    let dynamic_pairs: Vec<(String, String)> = args.dynamic
        .chunks(2)
        .map(|chunk| {
            let key = chunk[0].trim_start_matches("--").to_string();
            let value = chunk[1].to_string();
            (key, value)
        })
        .collect();
        
    println!("Path: {}", args.path);
    println!("Dynamic pairs: {:?}", dynamic_pairs);
    
    // let current_dir = std::env::current_dir()?;
    let base_path = Path::new(&args.path);

    let config_path = base_path.join("tmpl.yaml");
    println!("Config path: {}", config_path.display());
    let config = read_config(config_path.to_str().unwrap())?;
    println!("{:?}", config);

    let output_dir = args.output.as_deref().unwrap_or_else(|| base_path.to_str().unwrap());
    process_path(&args.path, &dynamic_pairs, &config, base_path, output_dir)?;
    Ok(())
}
