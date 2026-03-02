/// Resolve a secret value from plain text, environment variable, or file.
///
/// Supported URI schemes:
///   - `env://VAR_NAME` — reads from environment variable
///   - `file:///path/to/secret` — reads from file (trimmed)
///   - anything else — returned as-is (plain text)
pub fn resolve(value: &str) -> Result<String, anyhow::Error> {
    if let Some(var) = value.strip_prefix("env://") {
        std::env::var(var).map_err(|_| anyhow::anyhow!("environment variable '{var}' not set"))
    } else if let Some(path) = value.strip_prefix("file://") {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| anyhow::anyhow!("failed to read secret file '{path}': {e}"))
    } else {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_passthrough() {
        assert_eq!(resolve("sk-ant-abc123").unwrap(), "sk-ant-abc123");
    }

    #[test]
    fn test_env_resolve() {
        // SAFETY: test runs sequentially, no other threads access this var
        unsafe {
            std::env::set_var("_TEST_SECRET_KEY", "my-secret-value");
        }
        assert_eq!(
            resolve("env://_TEST_SECRET_KEY").unwrap(),
            "my-secret-value"
        );
        unsafe {
            std::env::remove_var("_TEST_SECRET_KEY");
        }
    }

    #[test]
    fn test_env_resolve_missing() {
        let result = resolve("env://_NONEXISTENT_VAR_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.txt");
        std::fs::write(&path, "  file-secret-value\n  ").unwrap();
        let uri = format!("file://{}", path.display());
        assert_eq!(resolve(&uri).unwrap(), "file-secret-value");
    }

    #[test]
    fn test_file_resolve_missing() {
        let result = resolve("file:///nonexistent/path/secret.txt");
        assert!(result.is_err());
    }
}
