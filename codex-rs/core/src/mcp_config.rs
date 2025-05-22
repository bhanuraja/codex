//! MCP Server Preset Management
use crate::config::{find_codex_home, ConfigToml};
use crate::config_types::McpServerPreset;
use std::fs;
use std::io;
use std::path::PathBuf;

fn get_config_path() -> io::Result<PathBuf> {
    let mut codex_home = find_codex_home()?;
    codex_home.push("config.toml");
    Ok(codex_home)
}

fn load_config_toml() -> io::Result<ConfigToml> {
    let config_path = get_config_path()?;
    match fs::read_to_string(&config_path) {
        Ok(contents) => toml::from_str::<ConfigToml>(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ConfigToml::default()),
        Err(e) => Err(e),
    }
}

fn save_config_toml(config_toml: &ConfigToml) -> io::Result<()> {
    let config_path = get_config_path()?;
    let toml_string =
        toml::to_string_pretty(&config_toml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(config_path, toml_string)
}

pub fn add_mcp_server_preset(label: String, url: String) -> Result<(), String> {
    let mut config_toml = load_config_toml().map_err(|e| e.to_string())?;

    let presets = config_toml.mcp_server_presets.get_or_insert_with(Vec::new);

    if let Some(preset) = presets.iter_mut().find(|p| p.label == label) {
        preset.url = url;
        preset.is_enabled = true;
    } else {
        presets.push(McpServerPreset {
            label,
            url,
            is_enabled: true,
        });
    }
    save_config_toml(&config_toml).map_err(|e| e.to_string())
}

pub fn remove_mcp_server_preset(label: String) -> Result<(), String> {
    let mut config_toml = load_config_toml().map_err(|e| e.to_string())?;

    let presets = config_toml.mcp_server_presets.get_or_insert_with(Vec::new);

    let initial_len = presets.len();
    presets.retain(|p| p.label != label);

    if presets.len() == initial_len {
        return Err(format!("Preset with label '{}' not found.", label));
    }

    save_config_toml(&config_toml).map_err(|e| e.to_string())
}

pub fn enable_mcp_server_preset(label: String, enabled_status: bool) -> Result<(), String> {
    let mut config_toml = load_config_toml().map_err(|e| e.to_string())?;

    let presets = config_toml.mcp_server_presets.get_or_insert_with(Vec::new);

    if let Some(preset) = presets.iter_mut().find(|p| p.label == label) {
        preset.is_enabled = enabled_status;
    } else {
        return Err(format!("Preset with label '{}' not found.", label));
    }

    save_config_toml(&config_toml).map_err(|e| e.to_string())
}

pub fn list_mcp_server_presets() -> Result<Vec<McpServerPreset>, String> {
    let config_toml = load_config_toml().map_err(|e| e.to_string())?;
    Ok(config_toml.mcp_server_presets.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    // Helper to setup a temporary config for tests
    fn setup_test_config_file(temp_dir: &tempfile::TempDir, initial_toml_content: Option<&str>) -> PathBuf {
        let config_path = temp_dir.path().join("config.toml");
        if let Some(content) = initial_toml_content {
            fs::write(&config_path, content).expect("Failed to write initial test config");
        }
        config_path
    }
    
    // Temporarily override CODEX_HOME for tests
    struct CodexHomeOverride {
        original_value: Option<String>,
        temp_dir: tempfile::TempDir,
    }

    impl CodexHomeOverride {
        fn new() -> Self {
            let temp_dir = tempdir().expect("Failed to create temp dir for CODEX_HOME");
            let original_value = std::env::var("CODEX_HOME").ok();
            std::env::set_var("CODEX_HOME", temp_dir.path());
            CodexHomeOverride { original_value, temp_dir }
        }
    }

    impl Drop for CodexHomeOverride {
        fn drop(&mut self) {
            if let Some(val) = &self.original_value {
                std::env::set_var("CODEX_HOME", val);
            } else {
                std::env::remove_var("CODEX_HOME");
            }
        }
    }


    #[test]
    fn test_add_new_mcp_server_preset() {
        let _home_override = CodexHomeOverride::new();
        setup_test_config_file(&_home_override.temp_dir, None);

        let label = "test_label".to_string();
        let url = "http://test.url".to_string();
        add_mcp_server_preset(label.clone(), url.clone()).expect("Failed to add preset");

        let presets = list_mcp_server_presets().expect("Failed to list presets");
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].label, label);
        assert_eq!(presets[0].url, url);
        assert!(presets[0].is_enabled);
    }

    #[test]
    fn test_add_existing_mcp_server_preset_updates_it() {
        let _home_override = CodexHomeOverride::new();
        let initial_toml = r#"
mcp_server_presets = [
    { label = "test_label", url = "http://old.url", is_enabled = false }
]
"#;
        setup_test_config_file(&_home_override.temp_dir, Some(initial_toml));
        
        let label = "test_label".to_string();
        let new_url = "http://new.url".to_string();
        add_mcp_server_preset(label.clone(), new_url.clone()).expect("Failed to add/update preset");

        let presets = list_mcp_server_presets().expect("Failed to list presets");
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].label, label);
        assert_eq!(presets[0].url, new_url);
        assert!(presets[0].is_enabled); // Should be re-enabled
    }

    #[test]
    fn test_remove_mcp_server_preset() {
        let _home_override = CodexHomeOverride::new();
        let initial_toml = r#"
mcp_server_presets = [
    { label = "label1", url = "url1", is_enabled = true },
    { label = "label2", url = "url2", is_enabled = true }
]
"#;
        setup_test_config_file(&_home_override.temp_dir, Some(initial_toml));

        remove_mcp_server_preset("label1".to_string()).expect("Failed to remove preset");

        let presets = list_mcp_server_presets().expect("Failed to list presets");
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].label, "label2");
    }

    #[test]
    fn test_remove_non_existent_mcp_server_preset() {
        let _home_override = CodexHomeOverride::new();
         let initial_toml = r#"
mcp_server_presets = [
    { label = "label1", url = "url1", is_enabled = true }
]
"#;
        setup_test_config_file(&_home_override.temp_dir, Some(initial_toml));

        let result = remove_mcp_server_preset("non_existent_label".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Preset with label 'non_existent_label' not found.");
    }

    #[test]
    fn test_enable_disable_mcp_server_preset() {
        let _home_override = CodexHomeOverride::new();
        let initial_toml = r#"
mcp_server_presets = [
    { label = "test_label", url = "http://test.url", is_enabled = true }
]
"#;
        setup_test_config_file(&_home_override.temp_dir, Some(initial_toml));

        // Disable
        enable_mcp_server_preset("test_label".to_string(), false).expect("Failed to disable preset");
        let presets_disabled = list_mcp_server_presets().expect("Failed to list presets");
        assert_eq!(presets_disabled.len(), 1);
        assert!(!presets_disabled[0].is_enabled);

        // Enable
        enable_mcp_server_preset("test_label".to_string(), true).expect("Failed to enable preset");
        let presets_enabled = list_mcp_server_presets().expect("Failed to list presets");
        assert_eq!(presets_enabled.len(), 1);
        assert!(presets_enabled[0].is_enabled);
    }

    #[test]
    fn test_enable_non_existent_mcp_server_preset() {
        let _home_override = CodexHomeOverride::new();
        setup_test_config_file(&_home_override.temp_dir, None);

        let result = enable_mcp_server_preset("non_existent_label".to_string(), true);
        assert!(result.is_err());
         assert_eq!(result.unwrap_err(), "Preset with label 'non_existent_label' not found.");
    }

    #[test]
    fn test_list_mcp_server_presets_empty() {
        let _home_override = CodexHomeOverride::new();
        setup_test_config_file(&_home_override.temp_dir, None);
        
        let presets = list_mcp_server_presets().expect("Failed to list presets");
        assert!(presets.is_empty());
    }

    #[test]
    fn test_list_mcp_server_presets_multiple() {
         let _home_override = CodexHomeOverride::new();
        let initial_toml = r#"
mcp_server_presets = [
    { label = "label1", url = "url1", is_enabled = true },
    { label = "label2", url = "url2", is_enabled = false }
]
"#;
        setup_test_config_file(&_home_override.temp_dir, Some(initial_toml));

        let presets = list_mcp_server_presets().expect("Failed to list presets");
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].label, "label1");
        assert!(presets[0].is_enabled);
        assert_eq!(presets[1].label, "label2");
        assert!(!presets[1].is_enabled);
    }

    #[test]
    fn test_config_file_is_pretty_printed() {
        let _home_override = CodexHomeOverride::new();
        setup_test_config_file(&_home_override.temp_dir, None);

        add_mcp_server_preset("label1".to_string(), "url1".to_string()).unwrap();
        add_mcp_server_preset("label2".to_string(), "url2".to_string()).unwrap();

        let config_path = get_config_path().unwrap();
        let content = fs::read_to_string(config_path).unwrap();
        
        let expected_content = r#"mcp_server_presets = [
    { label = "label1", url = "url1", is_enabled = true },
    { label = "label2", url = "url2", is_enabled = true },
]
"#;
        // Normalize line endings for comparison
        assert_eq!(content.replace("\r\n", "\n"), expected_content.replace("\r\n", "\n"));
    }
}
