use std::collections::HashMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

/// Price per 1M tokens (input and output separately).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelPrice {
    /// Cost per 1M input tokens in USD.
    pub input: f64,
    /// Cost per 1M output tokens in USD.
    pub output: f64,
}

/// Cost calculator with built-in price table and user overrides.
pub struct CostCalculator {
    prices: RwLock<HashMap<String, ModelPrice>>,
}

impl CostCalculator {
    pub fn new(overrides: &HashMap<String, ModelPrice>) -> Self {
        let mut prices = built_in_prices();
        // User overrides take precedence
        for (model, price) in overrides {
            prices.insert(model.clone(), price.clone());
        }
        Self {
            prices: RwLock::new(prices),
        }
    }

    /// Update prices (called on hot-reload).
    pub fn update_prices(&self, overrides: &HashMap<String, ModelPrice>) {
        let mut prices = built_in_prices();
        for (model, price) in overrides {
            prices.insert(model.clone(), price.clone());
        }
        if let Ok(mut p) = self.prices.write() {
            *p = prices;
        }
    }

    /// Calculate cost for a request in USD.
    /// Returns None if the model is not in the price table.
    pub fn calculate(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Option<f64> {
        let prices = self.prices.read().ok()?;

        // Try exact match first, then prefix match
        let price = prices.get(model).or_else(|| {
            // Try matching without provider prefix (e.g., "openai/gpt-4o" → "gpt-4o")
            let stripped = model.split('/').next_back().unwrap_or(model);
            prices.get(stripped)
        })?;

        let input_cost = (input_tokens as f64 / 1_000_000.0) * price.input;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * price.output;

        Some(input_cost + output_cost)
    }
}

/// Built-in price table for major models (USD per 1M tokens).
/// Prices as of early 2026.
fn built_in_prices() -> HashMap<String, ModelPrice> {
    let entries: Vec<(&str, f64, f64)> = vec![
        // Claude 4.x models (latest aliases)
        ("claude-opus-4-6", 15.0, 75.0),
        ("claude-sonnet-4-6", 3.0, 15.0),
        ("claude-opus-4-5", 15.0, 75.0),
        ("claude-sonnet-4-5", 3.0, 15.0),
        ("claude-haiku-4-5", 0.80, 4.0),
        // Claude 4.x models (dated versions)
        ("claude-opus-4-20250514", 15.0, 75.0),
        ("claude-sonnet-4-20250514", 3.0, 15.0),
        ("claude-haiku-4-20250514", 0.80, 4.0),
        ("claude-sonnet-4-5-20250929", 3.0, 15.0),
        ("claude-opus-4-5-20251101", 15.0, 75.0),
        ("claude-opus-4-1-20250805", 15.0, 75.0),
        ("claude-haiku-4-5-20251001", 0.80, 4.0),
        // Claude 3.x models
        ("claude-3-5-sonnet-20241022", 3.0, 15.0),
        ("claude-3-5-haiku-20241022", 0.80, 4.0),
        ("claude-3-opus-20240229", 15.0, 75.0),
        ("claude-3-sonnet-20240229", 3.0, 15.0),
        ("claude-3-haiku-20240307", 0.25, 1.25),
        // OpenAI models
        ("gpt-4o", 2.50, 10.0),
        ("gpt-4o-mini", 0.15, 0.60),
        ("gpt-4o-2024-11-20", 2.50, 10.0),
        ("gpt-4-turbo", 10.0, 30.0),
        ("gpt-4", 30.0, 60.0),
        ("gpt-3.5-turbo", 0.50, 1.50),
        ("o1", 15.0, 60.0),
        ("o1-mini", 3.0, 12.0),
        ("o1-pro", 150.0, 600.0),
        ("o3", 10.0, 40.0),
        ("o3-mini", 1.10, 4.40),
        ("o4-mini", 1.10, 4.40),
        // Gemini models
        ("gemini-2.5-pro-preview-06-05", 1.25, 10.0),
        ("gemini-2.5-flash-preview-05-20", 0.15, 0.60),
        ("gemini-2.0-flash", 0.10, 0.40),
        ("gemini-2.0-flash-lite", 0.075, 0.30),
        ("gemini-1.5-pro", 1.25, 5.0),
        ("gemini-1.5-flash", 0.075, 0.30),
        // DeepSeek models
        ("deepseek-chat", 0.27, 1.10),
        ("deepseek-reasoner", 0.55, 2.19),
        // Groq models
        ("llama-3.3-70b-versatile", 0.59, 0.79),
        ("llama-3.1-8b-instant", 0.05, 0.08),
    ];

    entries
        .into_iter()
        .map(|(model, input, output)| (model.to_string(), ModelPrice { input, output }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_known_model() {
        let calc = CostCalculator::new(&HashMap::new());
        // gpt-4o: $2.50/1M input, $10.0/1M output
        let cost = calc.calculate("gpt-4o", 1_000_000, 500_000);
        assert!(cost.is_some());
        let cost = cost.unwrap();
        // $2.50 (input) + $5.00 (output) = $7.50
        assert!((cost - 7.50).abs() < 0.001);
    }

    #[test]
    fn test_calculate_unknown_model() {
        let calc = CostCalculator::new(&HashMap::new());
        let cost = calc.calculate("unknown-model-xyz", 1000, 500);
        assert!(cost.is_none());
    }

    #[test]
    fn test_prefix_stripping() {
        let calc = CostCalculator::new(&HashMap::new());
        // Should match "gpt-4o" even with prefix
        let cost = calc.calculate("openai/gpt-4o", 1_000_000, 0);
        assert!(cost.is_some());
        assert!((cost.unwrap() - 2.50).abs() < 0.001);
    }

    #[test]
    fn test_user_override() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "my-custom-model".to_string(),
            ModelPrice {
                input: 1.0,
                output: 2.0,
            },
        );
        let calc = CostCalculator::new(&overrides);
        let cost = calc.calculate("my-custom-model", 1_000_000, 1_000_000);
        assert!(cost.is_some());
        // $1.00 + $2.00 = $3.00
        assert!((cost.unwrap() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_override_built_in() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "gpt-4o".to_string(),
            ModelPrice {
                input: 100.0,
                output: 200.0,
            },
        );
        let calc = CostCalculator::new(&overrides);
        let cost = calc.calculate("gpt-4o", 1_000_000, 0);
        assert!(cost.is_some());
        assert!((cost.unwrap() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_update_prices() {
        let calc = CostCalculator::new(&HashMap::new());
        assert!(calc.calculate("custom-model", 1000, 500).is_none());

        let mut overrides = HashMap::new();
        overrides.insert(
            "custom-model".to_string(),
            ModelPrice {
                input: 5.0,
                output: 10.0,
            },
        );
        calc.update_prices(&overrides);
        assert!(calc.calculate("custom-model", 1000, 500).is_some());
    }

    #[test]
    fn test_zero_tokens() {
        let calc = CostCalculator::new(&HashMap::new());
        let cost = calc.calculate("gpt-4o", 0, 0);
        assert!(cost.is_some());
        assert!((cost.unwrap()).abs() < 0.001);
    }
}
