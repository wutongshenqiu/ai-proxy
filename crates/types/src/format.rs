use serde::{Deserialize, Serialize};

/// Supported provider/API format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    OpenAI,
    Claude,
    Gemini,
    OpenAICompat,
}

impl Format {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
            Self::OpenAICompat => "openai-compat",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openai" => Ok(Self::OpenAI),
            "claude" => Ok(Self::Claude),
            "gemini" => Ok(Self::Gemini),
            "openai-compat" | "openai_compat" => Ok(Self::OpenAICompat),
            _ => Err(format!("unknown format: {s}")),
        }
    }
}

/// Wire API format for OpenAI-compatible providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WireApi {
    #[default]
    Chat,
    Responses,
}
