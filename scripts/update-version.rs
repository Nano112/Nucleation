#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! toml = "0.8"
//! regex = "1.0"
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;
use regex::Regex;

#[derive(Debug)]
struct VersionConfig {
    version: String,
    description: String,
}

fn load_version_config() -> Result<VersionConfig, Box<dyn std::error::Error>> {
    let config_path = Path::new("version.toml");
    let content = fs::read_to_string(config_path)?;
    
    // Simple TOML parsing for our specific format
    let version_re = Regex::new(r#"version = "([^"]+)""#)?;
    let description_re = Regex::new(r#"description = "([^"]+)""#)?;
    
    let version = version_re.captures(&content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or("Version not found in version.toml")?;
    
    let description = description_re.captures(&content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or("Description not found in version.toml")?;
    
    Ok(VersionConfig { version, description })
}

fn update_cargo_toml(config: &VersionConfig) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_path = Path::new("Cargo.toml");
    let content = fs::read_to_string(cargo_path)?;
    
    let version_re = Regex::new(r#"^version = ".*""#)?;
    let description_re = Regex::new(r#"^description = ".*""#)?;
    
    let content = version_re.replace(&content, format!(r#"version = "{}""#, config.version));
    let content = description_re.replace(&content, format!(r#"description = "{}""#, config.description));
    
    fs::write(cargo_path, content.as_ref())?;
    println!("Updated Cargo.toml to version {}", config.version);
    Ok(())
}

fn update_pyproject_toml(config: &VersionConfig) -> Result<(), Box<dyn std::error::Error>> {
    let pyproject_path = Path::new("pyproject.toml");
    let content = fs::read_to_string(pyproject_path)?;
    
    let version_re = Regex::new(r#"^version = ".*""#)?;
    let description_re = Regex::new(r#"^description = ".*""#)?;
    
    let content = version_re.replace(&content, format!(r#"version = "{}""#, config.version));
    let content = description_re.replace(&content, format!(r#"description = "{}""#, config.description));
    
    fs::write(pyproject_path, content.as_ref())?;
    println!("Updated pyproject.toml to version {}", config.version);
    Ok(())
}

fn update_api_definition(config: &VersionConfig) -> Result<(), Box<dyn std::error::Error>> {
    let api_def_path = Path::new("src/api_definition.rs");
    let content = fs::read_to_string(api_def_path)?;
    
    let version_re = Regex::new(r#"version: ".*?".to_string\(\),"#)?;
    let content = version_re.replace(&content, format!(r#"version: "{}".to_string(),"#, config.version));
    
    fs::write(api_def_path, content.as_ref())?;
    println!("Updated api_definition.rs to version {}", config.version);
    Ok(())
}

fn update_cargo_lock() -> Result<(), Box<dyn std::error::Error>> {
    Command::new("cargo")
        .args(&["check"])
        .output()?;
    println!("Updated Cargo.lock");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_version_config()?;
    
    println!("Updating to version {}", config.version);
    println!("Description: {}", config.description);
    println!();
    
    update_cargo_toml(&config)?;
    update_pyproject_toml(&config)?;
    update_api_definition(&config)?;
    update_cargo_lock()?;
    
    println!();
    println!("✅ All files updated successfully!");
    println!("New version: {}", config.version);
    
    Ok(())
}
