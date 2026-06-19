use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const USER_RUNTIME_CONFIG_FILE_NAME: &str = "runtime-bundle.json";
fn app_runtime_dir_name() -> &'static str {
    if cfg!(debug_assertions) {
        "System-I2-Dev"
    } else {
        "System-I2"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRuntimeConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub bundle_dir: String,
}

pub(crate) fn load_optional_user_runtime_config() -> Result<Option<UserRuntimeConfig>, String> {
    let config_path = resolve_user_runtime_config_path()?;

    if !config_path.exists() {
        return Ok(None);
    }

    let file_contents = fs::read_to_string(&config_path).map_err(|error| {
        format!(
            "failed to read user runtime config at {}: {error}",
            config_path.display()
        )
    })?;

    let config = serde_json::from_str::<UserRuntimeConfig>(&file_contents).map_err(|error| {
        format!(
            "failed to parse user runtime config at {}: {error}",
            config_path.display()
        )
    })?;

    validate_user_runtime_config(&config, &config_path)?;

    Ok(Some(config))
}

pub(crate) fn save_user_runtime_config(bundle_dir: &Path) -> Result<PathBuf, String> {
    let config_path = resolve_user_runtime_config_path()?;
    let parent_dir = config_path.parent().ok_or_else(|| {
        format!(
            "failed to resolve parent directory for user runtime config path {}",
            config_path.display()
        )
    })?;

    fs::create_dir_all(parent_dir).map_err(|error| {
        format!(
            "failed to create user runtime config directory {}: {error}",
            parent_dir.display()
        )
    })?;

    let config = UserRuntimeConfig {
        schema_version: default_schema_version(),
        bundle_dir: bundle_dir.display().to_string(),
    };

    let serialized = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("failed to serialize user runtime config: {error}"))?;

    fs::write(&config_path, format!("{serialized}\n")).map_err(|error| {
        format!(
            "failed to write user runtime config at {}: {error}",
            config_path.display()
        )
    })?;

    Ok(config_path)
}

pub(crate) fn resolve_user_runtime_config_path() -> Result<PathBuf, String> {
    #[cfg(test)]
    {
        if let Some(override_path) = super::load_test_user_runtime_config_override_path() {
            return Ok(override_path);
        }
    }

    Ok(resolve_user_runtime_config_dir()?.join(USER_RUNTIME_CONFIG_FILE_NAME))
}

pub(crate) fn resolve_default_runtime_bundle_dir() -> Result<PathBuf, String> {
    #[cfg(test)]
    {
        if let Some(override_path) = super::load_test_user_runtime_config_override_path() {
            return override_path
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| {
                    format!(
                    "failed to resolve parent directory for overridden user runtime config path {}",
                    override_path.display()
                )
                });
        }
    }

    resolve_user_runtime_config_dir()
}

fn validate_user_runtime_config(
    config: &UserRuntimeConfig,
    config_path: &Path,
) -> Result<(), String> {
    if config.bundle_dir.trim().is_empty() {
        return Err(format!(
            "user runtime config field 'bundleDir' is empty in {}",
            config_path.display()
        ));
    }

    Ok(())
}

fn resolve_user_runtime_config_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|config_dir| config_dir.join(app_runtime_dir_name()))
        .ok_or_else(|| {
            "failed to resolve system config directory for user runtime config".to_string()
        })
}

fn default_schema_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::UserRuntimeConfig;
    use serde_json::json;

    #[test]
    fn deserializes_user_runtime_config_from_camel_case_json() {
        let config = serde_json::from_value::<UserRuntimeConfig>(json!({
            "schemaVersion": 1,
            "bundleDir": "/tmp/system-i2-data"
        }))
        .expect("expected user runtime config to deserialize");

        assert_eq!(config.schema_version, 1);
        assert_eq!(config.bundle_dir, "/tmp/system-i2-data");
    }
}
