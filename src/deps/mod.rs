use async_std::fs;
use async_std::io::Result;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;

mod packages;

#[derive(Deserialize)]
struct Pkg {
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Default)]
struct UserCustomDeps {
    dependencies: Option<HashSet<String>>,
    dev_dependencies: Option<HashSet<String>>,
    exclude: Option<HashSet<String>>,
}

pub async fn fetch_installed_packages() -> Result<String> {
    let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("package.json");

    let content = fs::read(path).await?;
    let pkg = serde_json::from_slice::<Pkg>(&content)?;

    let deps = pkg
        .dependencies
        .map(|deps| deps.keys().join("\n"))
        .unwrap_or_default();
    let dev_deps = pkg
        .dev_dependencies
        .map(|deps| deps.keys().join("\n"))
        .unwrap_or_default();

    Ok(format!("{}\n{}", deps, dev_deps))
}

pub async fn return_dependencies(path: Option<PathBuf>) -> Result<String> {
    let path = path.unwrap_or_else(default_custom_deps_file_path);

    let custom = fetch_custom_dependencies(&path).await?;
    let exclude = fetch_exclude_dependencies(&path).await?;

    let mut dependencies = packages::dependencies();
    dependencies.extend(custom.into_iter());
    let dependencies = dependencies.difference(&exclude).join("\n");
    Ok(dependencies)
}

pub async fn return_dev_dependencies(path: Option<PathBuf>) -> Result<String> {
    let path = path.unwrap_or_else(default_custom_deps_file_path);

    let custom = fetch_custom_dev_dependencies(&path).await?;
    let exclude = fetch_exclude_dependencies(&path).await?;

    let mut dependencies = packages::dev_dependencies();
    dependencies.extend(custom.into_iter());
    let dependencies = dependencies.difference(&exclude).join("\n");
    Ok(dependencies)
}

fn default_custom_deps_file_path() -> PathBuf {
    dirs::home_dir()
        .map(|path| path.join(".yarn-autocompletions.yml"))
        .unwrap_or_default()
}

async fn fetch_custom_dependencies(path: &PathBuf) -> Result<HashSet<String>> {
    let content = fs::read(path).await?;
    let custom = serde_yaml::from_slice::<UserCustomDeps>(&content).unwrap_or_default();

    Ok(custom.dependencies.unwrap_or_default())
}

async fn fetch_custom_dev_dependencies(path: &PathBuf) -> Result<HashSet<String>> {
    let content = fs::read(path).await?;
    let custom = serde_yaml::from_slice::<UserCustomDeps>(&content).unwrap_or_default();

    Ok(custom.dev_dependencies.unwrap_or_default())
}

async fn fetch_exclude_dependencies(path: &PathBuf) -> Result<HashSet<String>> {
    let content = fs::read(path).await?;
    let custom = serde_yaml::from_slice::<UserCustomDeps>(&content).unwrap_or_default();

    Ok(custom.exclude.unwrap_or_default())
}

// TODO: use async here.
pub fn list_node_modules() -> std::io::Result<String> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let directories = std::fs::read_dir(cwd.join("node_modules"));

    directories.map(|dirs| {
        dirs.map(|dir| {
            dir.map(|entry| entry.file_name().into_string().unwrap_or_default())
                .unwrap_or_default()
        })
        .join("\n")
    })
}

#[async_std::test]
async fn test_fetch_installed_packages() {
    let output = fetch_installed_packages().await.unwrap();
    let output = output.trim();
    let mut packages: Vec<&str> = output.split('\n').collect();
    packages.sort();
    assert_eq!(packages, ["a", "b", "c", "d"]);
}

#[async_std::test]
async fn test_return_dependencies() {
    let path = PathBuf::from("yarn-autocompletions.example.yml");
    let output = return_dependencies(Some(path)).await.unwrap();
    assert!(output.contains("vue"));
    assert!(!output.contains("axios"));
}

#[async_std::test]
async fn test_return_dev_dependencies() {
    let path = PathBuf::from("yarn-autocompletions.example.yml");
    let output = return_dev_dependencies(Some(path)).await.unwrap();
    assert!(output.contains("@babel/core"));
    assert!(!output.contains("gulp"));
}
