//! Public mod-declared configuration schema contracts.
//!
//! These types describe the configuration settings a mod declares and their
//! defaults. They do not represent the active runtime configuration; resolved
//! runtime values are supplied by the platform through guest start input.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Supported schema version for `ModConfigSchema`.
pub const MOD_CONFIG_SCHEMA_V1: u32 = 1;

/// A mod-declared configuration schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModConfigSchema {
    /// Schema version for this document.
    pub schema: u32,
    /// Settings declared by the mod.
    #[serde(default)]
    pub settings: Vec<ModConfigSetting>,
}

impl Default for ModConfigSchema {
    fn default() -> Self {
        Self {
            schema: MOD_CONFIG_SCHEMA_V1,
            settings: Vec::new(),
        }
    }
}

impl ModConfigSchema {
    /// Validates that all settings are internally consistent and uniquely keyed.
    pub fn validate(&self) -> Result<(), ModConfigSchemaError> {
        if self.schema != MOD_CONFIG_SCHEMA_V1 {
            return Err(ModConfigSchemaError::UnsupportedSchema {
                found: self.schema,
                supported: MOD_CONFIG_SCHEMA_V1,
            });
        }

        let mut keys = HashSet::with_capacity(self.settings.len());
        for setting in &self.settings {
            setting.validate()?;
            if !keys.insert(setting.key.as_str()) {
                return Err(ModConfigSchemaError::DuplicateSettingKey {
                    key: setting.key.clone(),
                });
            }
        }

        Ok(())
    }
}

/// A single mod-declared configuration setting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModConfigSetting {
    /// Stable setting key used in config files and resolved runtime config maps.
    pub key: String,
    /// Declared value type for this setting.
    #[serde(rename = "type")]
    pub value_type: ConfigValueType,
    /// Mod-declared fallback value.
    pub default: ConfigValue,
    /// Optional inclusive lower bound for numeric settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<ConfigValue>,
    /// Optional inclusive upper bound for numeric settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<ConfigValue>,
    /// Optional explicit allowed values. Enum settings require string values here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<ConfigValue>,
    /// Optional human-readable description for tooling and UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Scope at which the platform resolves this setting.
    pub scope: ConfigScope,
    /// Required action when the effective value changes.
    pub reload: ConfigReloadPolicy,
    /// Authority responsible for choosing the effective value.
    pub authority: ConfigAuthority,
    /// Hints that tooling should hide this setting from ordinary UI.
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// Hints that tooling should show this setting as read-only.
    #[serde(default, skip_serializing_if = "is_false")]
    pub locked: bool,
}

impl ModConfigSetting {
    /// Creates a setting with no constraints, description, or UI markers.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        value_type: ConfigValueType,
        default: ConfigValue,
        scope: ConfigScope,
        reload: ConfigReloadPolicy,
        authority: ConfigAuthority,
    ) -> Self {
        Self {
            key: key.into(),
            value_type,
            default,
            min: None,
            max: None,
            allowed_values: Vec::new(),
            description: None,
            scope,
            reload,
            authority,
            hidden: false,
            locked: false,
        }
    }

    /// Validates this setting without checking uniqueness against other settings.
    pub fn validate(&self) -> Result<(), ModConfigSchemaError> {
        if self.key.trim().is_empty() {
            return Err(ModConfigSchemaError::EmptySettingKey);
        }

        self.validate_value_type(&self.default, ValueField::Default)?;
        self.validate_numeric_bound(self.min.as_ref(), ValueField::Min)?;
        self.validate_numeric_bound(self.max.as_ref(), ValueField::Max)?;

        if let (Some(min), Some(max)) = (&self.min, &self.max)
            && numeric_cmp(min, max).is_some_and(|ordering| ordering.is_gt())
        {
            return Err(ModConfigSchemaError::MinGreaterThanMax {
                key: self.key.clone(),
            });
        }

        if let Some(min) = &self.min
            && numeric_cmp(&self.default, min).is_some_and(|ordering| ordering.is_lt())
        {
            return Err(ModConfigSchemaError::DefaultBelowMin {
                key: self.key.clone(),
            });
        }

        if let Some(max) = &self.max
            && numeric_cmp(&self.default, max).is_some_and(|ordering| ordering.is_gt())
        {
            return Err(ModConfigSchemaError::DefaultAboveMax {
                key: self.key.clone(),
            });
        }

        if self.value_type == ConfigValueType::Enum && self.allowed_values.is_empty() {
            return Err(ModConfigSchemaError::EmptyAllowedValues {
                key: self.key.clone(),
            });
        }

        for (index, value) in self.allowed_values.iter().enumerate() {
            self.validate_value_type(value, ValueField::AllowedValue { index })?;
        }

        if !self.allowed_values.is_empty() && !self.allowed_values.contains(&self.default) {
            return Err(ModConfigSchemaError::DefaultNotAllowed {
                key: self.key.clone(),
            });
        }

        Ok(())
    }

    fn validate_numeric_bound(
        &self,
        value: Option<&ConfigValue>,
        field: ValueField,
    ) -> Result<(), ModConfigSchemaError> {
        let Some(value) = value else {
            return Ok(());
        };

        if !self.value_type.is_numeric() {
            return Err(ModConfigSchemaError::NumericBoundOnNonNumericSetting {
                key: self.key.clone(),
                field,
                value_type: self.value_type,
            });
        }

        self.validate_value_type(value, field)
    }

    fn validate_value_type(
        &self,
        value: &ConfigValue,
        field: ValueField,
    ) -> Result<(), ModConfigSchemaError> {
        let actual = ConfigValueType::from(value);
        if !self.value_type.accepts_value(value) {
            return Err(ModConfigSchemaError::ValueTypeMismatch {
                key: self.key.clone(),
                field,
                expected: self.value_type,
                actual,
            });
        }

        if value.is_non_finite_float() {
            return Err(ModConfigSchemaError::NonFiniteFloat {
                key: self.key.clone(),
                field,
            });
        }

        Ok(())
    }
}

/// Declared value type for a configuration setting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConfigValueType {
    /// Boolean value.
    Bool,
    /// Signed 64-bit integer value.
    Int,
    /// 64-bit floating-point value.
    Float,
    /// Free-form UTF-8 string value.
    String,
    /// String value constrained by `allowed_values`.
    Enum,
}

impl ConfigValueType {
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }

    #[must_use]
    pub fn accepts_value(self, value: &ConfigValue) -> bool {
        matches!(
            (self, value),
            (Self::Bool, ConfigValue::Bool(_))
                | (Self::Int, ConfigValue::Int(_))
                | (Self::Float, ConfigValue::Float(_))
                | (Self::String | Self::Enum, ConfigValue::String(_))
        )
    }
}

impl From<&ConfigValue> for ConfigValueType {
    fn from(value: &ConfigValue) -> Self {
        match value {
            ConfigValue::Bool(_) => Self::Bool,
            ConfigValue::Int(_) => Self::Int,
            ConfigValue::Float(_) => Self::Float,
            ConfigValue::String(_) => Self::String,
        }
    }
}

/// Literal value used in mod-declared config defaults and constraints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    /// Boolean literal.
    Bool(bool),
    /// Signed 64-bit integer literal.
    Int(i64),
    /// 64-bit floating-point literal.
    Float(f64),
    /// UTF-8 string literal.
    String(String),
}

impl ConfigValue {
    #[must_use]
    pub fn is_non_finite_float(&self) -> bool {
        matches!(self, Self::Float(value) if !value.is_finite())
    }
}

/// Scope at which a setting is resolved.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConfigScope {
    /// Resolved once during process or runtime startup.
    Startup,
    /// Resolved per server world.
    ServerWorld,
    /// Resolved per local client user.
    ClientUser,
}

/// Required action when a setting value changes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConfigReloadPolicy {
    /// A full restart is required.
    Restart,
    /// The active world must be restarted.
    WorldRestart,
    /// The client must reconnect.
    Reconnect,
    /// The value may be applied while running.
    Runtime,
}

/// Authority responsible for resolving a setting value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConfigAuthority {
    /// The server chooses the effective value.
    Server,
    /// The client chooses the effective value.
    Client,
    /// The end user chooses the effective value.
    User,
}

/// Validation failure for a mod configuration schema.
#[derive(Debug, Clone, PartialEq)]
pub enum ModConfigSchemaError {
    UnsupportedSchema {
        found: u32,
        supported: u32,
    },
    EmptySettingKey,
    DuplicateSettingKey {
        key: String,
    },
    ValueTypeMismatch {
        key: String,
        field: ValueField,
        expected: ConfigValueType,
        actual: ConfigValueType,
    },
    NumericBoundOnNonNumericSetting {
        key: String,
        field: ValueField,
        value_type: ConfigValueType,
    },
    MinGreaterThanMax {
        key: String,
    },
    DefaultBelowMin {
        key: String,
    },
    DefaultAboveMax {
        key: String,
    },
    EmptyAllowedValues {
        key: String,
    },
    DefaultNotAllowed {
        key: String,
    },
    NonFiniteFloat {
        key: String,
        field: ValueField,
    },
}

impl fmt::Display for ModConfigSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { found, supported } => write!(
                f,
                "unsupported mod config schema version {found}, expected {supported}",
            ),
            Self::EmptySettingKey => f.write_str("config setting key must not be empty"),
            Self::DuplicateSettingKey { key } => {
                write!(f, "config setting key '{key}' is declared more than once")
            }
            Self::ValueTypeMismatch {
                key,
                field,
                expected,
                actual,
            } => write!(
                f,
                "config setting '{key}' {field} has type {actual:?}, expected {expected:?}",
            ),
            Self::NumericBoundOnNonNumericSetting {
                key,
                field,
                value_type,
            } => write!(
                f,
                "config setting '{key}' {field} is only valid for numeric settings, got {value_type:?}",
            ),
            Self::MinGreaterThanMax { key } => {
                write!(f, "config setting '{key}' min must not be greater than max")
            }
            Self::DefaultBelowMin { key } => {
                write!(f, "config setting '{key}' default is below min")
            }
            Self::DefaultAboveMax { key } => {
                write!(f, "config setting '{key}' default is above max")
            }
            Self::EmptyAllowedValues { key } => {
                write!(f, "config setting '{key}' enum requires allowed_values")
            }
            Self::DefaultNotAllowed { key } => {
                write!(f, "config setting '{key}' default is not in allowed_values")
            }
            Self::NonFiniteFloat { key, field } => {
                write!(f, "config setting '{key}' {field} must be a finite float")
            }
        }
    }
}

impl Error for ModConfigSchemaError {}

/// Field being validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueField {
    Default,
    Min,
    Max,
    AllowedValue { index: usize },
}

impl fmt::Display for ValueField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => f.write_str("default"),
            Self::Min => f.write_str("min"),
            Self::Max => f.write_str("max"),
            Self::AllowedValue { index } => write!(f, "allowed_values[{index}]"),
        }
    }
}

fn numeric_cmp(left: &ConfigValue, right: &ConfigValue) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (ConfigValue::Int(left), ConfigValue::Int(right)) => Some(left.cmp(right)),
        (ConfigValue::Float(left), ConfigValue::Float(right)) => left.partial_cmp(right),
        _ => None,
    }
}

const fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(key: &str, value_type: ConfigValueType, default: ConfigValue) -> ModConfigSetting {
        ModConfigSetting::new(
            key,
            value_type,
            default,
            ConfigScope::ServerWorld,
            ConfigReloadPolicy::Runtime,
            ConfigAuthority::Server,
        )
    }

    #[test]
    fn validates_bool_int_float_string_and_enum_settings() {
        let mut max_players = setting("max_players", ConfigValueType::Int, ConfigValue::Int(16));
        max_players.min = Some(ConfigValue::Int(1));
        max_players.max = Some(ConfigValue::Int(64));

        let mut gravity = setting("gravity", ConfigValueType::Float, ConfigValue::Float(9.8));
        gravity.min = Some(ConfigValue::Float(0.0));
        gravity.max = Some(ConfigValue::Float(20.0));

        let mut difficulty = setting(
            "difficulty",
            ConfigValueType::Enum,
            ConfigValue::String("normal".to_owned()),
        );
        difficulty.allowed_values = vec![
            ConfigValue::String("easy".to_owned()),
            ConfigValue::String("normal".to_owned()),
            ConfigValue::String("hard".to_owned()),
        ];

        let schema = ModConfigSchema {
            schema: MOD_CONFIG_SCHEMA_V1,
            settings: vec![
                setting("enabled", ConfigValueType::Bool, ConfigValue::Bool(true)),
                max_players,
                gravity,
                setting(
                    "motd",
                    ConfigValueType::String,
                    ConfigValue::String("Welcome".to_owned()),
                ),
                difficulty,
            ],
        };

        assert_eq!(schema.validate(), Ok(()));
    }

    #[test]
    fn rejects_invalid_default_type() {
        let err = setting(
            "enabled",
            ConfigValueType::Bool,
            ConfigValue::String("yes".to_owned()),
        )
        .validate()
        .expect_err("string default should not validate as bool");

        assert_eq!(
            err,
            ModConfigSchemaError::ValueTypeMismatch {
                key: "enabled".to_owned(),
                field: ValueField::Default,
                expected: ConfigValueType::Bool,
                actual: ConfigValueType::String,
            }
        );
    }

    #[test]
    fn rejects_invalid_min_and_max() {
        let mut non_numeric = setting(
            "motd",
            ConfigValueType::String,
            ConfigValue::String("Welcome".to_owned()),
        );
        non_numeric.min = Some(ConfigValue::String("A".to_owned()));

        assert!(matches!(
            non_numeric.validate(),
            Err(ModConfigSchemaError::NumericBoundOnNonNumericSetting {
                key,
                field: ValueField::Min,
                value_type: ConfigValueType::String,
            }) if key == "motd"
        ));

        let mut inverted = setting("max_players", ConfigValueType::Int, ConfigValue::Int(16));
        inverted.min = Some(ConfigValue::Int(64));
        inverted.max = Some(ConfigValue::Int(1));

        assert_eq!(
            inverted.validate(),
            Err(ModConfigSchemaError::MinGreaterThanMax {
                key: "max_players".to_owned(),
            })
        );
    }

    #[test]
    fn validates_allowed_values() {
        let mut wrong_allowed_type = setting(
            "difficulty",
            ConfigValueType::Enum,
            ConfigValue::String("normal".to_owned()),
        );
        wrong_allowed_type.allowed_values = vec![
            ConfigValue::String("normal".to_owned()),
            ConfigValue::Int(1),
        ];

        assert_eq!(
            wrong_allowed_type.validate(),
            Err(ModConfigSchemaError::ValueTypeMismatch {
                key: "difficulty".to_owned(),
                field: ValueField::AllowedValue { index: 1 },
                expected: ConfigValueType::Enum,
                actual: ConfigValueType::Int,
            })
        );

        let mut default_outside_allowed = setting(
            "difficulty",
            ConfigValueType::Enum,
            ConfigValue::String("normal".to_owned()),
        );
        default_outside_allowed.allowed_values = vec![ConfigValue::String("hard".to_owned())];

        assert_eq!(
            default_outside_allowed.validate(),
            Err(ModConfigSchemaError::DefaultNotAllowed {
                key: "difficulty".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_empty_and_duplicate_keys() {
        assert_eq!(
            ModConfigSchema {
                schema: 2,
                settings: Vec::new(),
            }
            .validate(),
            Err(ModConfigSchemaError::UnsupportedSchema {
                found: 2,
                supported: MOD_CONFIG_SCHEMA_V1,
            })
        );

        for key in ["", "   "] {
            assert_eq!(
                setting(key, ConfigValueType::Bool, ConfigValue::Bool(true)).validate(),
                Err(ModConfigSchemaError::EmptySettingKey)
            );
        }

        let schema = ModConfigSchema {
            schema: MOD_CONFIG_SCHEMA_V1,
            settings: vec![
                setting("enabled", ConfigValueType::Bool, ConfigValue::Bool(true)),
                setting("enabled", ConfigValueType::Bool, ConfigValue::Bool(false)),
            ],
        };

        assert_eq!(
            schema.validate(),
            Err(ModConfigSchemaError::DuplicateSettingKey {
                key: "enabled".to_owned(),
            })
        );
    }

    #[test]
    fn serde_roundtrip_shape() {
        let mut setting = setting("max_players", ConfigValueType::Int, ConfigValue::Int(16));
        setting.min = Some(ConfigValue::Int(1));
        setting.max = Some(ConfigValue::Int(64));
        setting.description = Some("Maximum players allowed in a world.".to_owned());
        setting.reload = ConfigReloadPolicy::WorldRestart;
        setting.locked = true;

        let schema = ModConfigSchema {
            schema: MOD_CONFIG_SCHEMA_V1,
            settings: vec![setting],
        };

        let value = serde_json::to_value(&schema).expect("serialize schema");
        assert_eq!(
            value,
            serde_json::json!({
                "schema": 1,
                "settings": [{
                    "key": "max_players",
                    "type": "int",
                    "default": 16,
                    "min": 1,
                    "max": 64,
                    "description": "Maximum players allowed in a world.",
                    "scope": "server_world",
                    "reload": "world_restart",
                    "authority": "server",
                    "locked": true
                }]
            })
        );

        let decoded: ModConfigSchema =
            serde_json::from_value(value).expect("deserialize schema shape");
        assert_eq!(decoded, schema);
    }
}
