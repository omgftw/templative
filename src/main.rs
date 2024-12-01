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

fn process_path(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &PathBuf, output_path: &str) -> eyre::Result<()> {
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

fn process_file(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &PathBuf, output_path: &str) -> eyre::Result<()> {
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
    let mut output_path = path.strip_suffix(".tmpl").unwrap().to_string();
    for rewrite in &config.path_rewrites {
        output_path = output_path.replace(&rewrite.from, &rewrite.to);
    }
    
    // Ensure output directory exists
    if let Some(parent) = Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Write rendered content
    std::fs::write(output_path, rendered)?;
    
    Ok(())
}

fn process_chunk(path: &str, dynamic_pairs: &Vec<(String, String)>, config: &Config, root_path: &PathBuf) -> eyre::Result<()> {
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
    let default_config_path = current_dir.join(args.path.clone()).join("tmpl.yaml");
    let config_path = args.config.as_deref().clone().unwrap_or(default_config_path.to_str().unwrap());
    println!("Config path: {}", config_path);
    let config = read_config(config_path)?;
    println!("{:?}", config);

    let output_dir = args.output.as_deref().unwrap_or_else(|| current_dir.to_str().unwrap());
    process_path(&args.path, &dynamic_pairs, &config, &current_dir, output_dir)?;
    Ok(())
}
