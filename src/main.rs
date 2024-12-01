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
        if path.is_file() && {
            let file_name = path.file_name().map_or("", |f| f.to_str().unwrap_or(""));
            file_name.contains("tmpl_") || (file_name.contains(".tmpl."))
        } {
            process_chunk(path.to_str().unwrap(), dynamic_pairs, config, root_path, output_path)?;
        }
    }
    Ok(())
}

fn apply_path_rewrites(path: &str, rewrites: &[PathRewrite], data: &serde_json::Map<String, serde_json::Value>) -> eyre::Result<String> {
    let handlebars = handlebars::Handlebars::new();
    let mut result = path.to_string();
    
    for rewrite in rewrites {
        // Render both 'from' and 'to' patterns using handlebars
        let from_pattern = handlebars.render_template(&rewrite.from, data)?;
        let to_pattern = handlebars.render_template(&rewrite.to, data)?;
        result = result.replace(&from_pattern, &to_pattern);
    }
    
    Ok(result)
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
    file_path = apply_path_rewrites(&file_path, &config.path_rewrites, &data)?;
    
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

fn process_chunk(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &Path, output_path: &str) -> eyre::Result<()> {
    // Read and render the chunk template
    let template_content = std::fs::read_to_string(path)?;
    let handlebars = handlebars::Handlebars::new();
    let mut data = serde_json::Map::new();
    for (key, value) in dynamic_pairs {
        data.insert(key.clone(), serde_json::Value::String(value.clone()));
    }
    let rendered_chunk = handlebars.render_template(&template_content, &data)?;
    
    // Get the target file path and chunk ID using either separator
    let file_path = Path::new(path);
    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    
    // Try to get chunk_id from either underscore or dot notation
    let chunk_id = if file_name.contains("tmpl_") {
        // Handle underscore separator (e.g., file.tmpl_chunk_id)
        file_name.split("tmpl_").nth(1).unwrap()
    } else if file_name.contains("tmpl.") {
        // Handle dot separator (e.g., file.txt.tmpl.chunk_id)
        file_name.split('.').rev().next().unwrap()
    } else {
        return Err(eyre::eyre!("No chunk ID found in file name"));
    };

    // Get base path by removing the chunk extension
    let target_path = if file_name.contains("tmpl_") {
        path[..path.rfind("tmpl_").unwrap()].trim_end_matches('.').to_string()
    } else {
        // For dot notation, remove both .tmpl and the chunk_id
        path[..path.rfind(".tmpl.").unwrap()].to_string()
    };

    // Apply path rewrites
    let target_path = apply_path_rewrites(&target_path, &config.path_rewrites, &data)?;
    
    // Convert to relative path and combine with output_path
    let relative_path = Path::new(&target_path)
        .strip_prefix(root_path)
        .unwrap_or(Path::new(&target_path));
    let final_target_path = Path::new(output_path).join(relative_path);
    
    println!("Final target path: {}", final_target_path.display());
    // Read and modify the target file
    let file_content = std::fs::read_to_string(&final_target_path)?;
    let lines: Vec<&str> = file_content.lines().collect();
    
    // Create the exact marker to search for
    let chunk_marker = format!("tmpl:{}", chunk_id);
    
    let mut new_content = String::new();
    for line in lines {
        if line.contains(&chunk_marker) {
            new_content.push_str(&rendered_chunk);
            new_content.push('\n');
        }
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    // Write the modified content back
    std::fs::write(final_target_path, new_content)?;
    
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
    
    let current_dir = std::env::current_dir()?;
    let base_path = Path::new(&args.path);

    let config_path = base_path.join("tmpl.yaml");
    println!("Config path: {}", config_path.display());
    let config = read_config(config_path.to_str().unwrap())?;
    println!("{:?}", config);

    let output_dir = args.output.as_deref().unwrap_or_else(|| current_dir.to_str().unwrap());
    process_path(&args.path, &dynamic_pairs, &config, base_path, output_dir)?;
    Ok(())
}
