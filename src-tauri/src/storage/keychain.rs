/// OS keychain storage via the `keyring` crate.
///
/// Keys are stored under service `"promptflow-stt"` with the provider
/// name (e.g. `"openai"`, `"groq"`) as the account identifier.
use crate::error::AppError;
use keyring::Entry;

pub struct KeychainStore;

impl KeychainStore {
    pub fn new() -> Self {
        Self
    }

    /// Saves the API key for `provider` to the OS keychain.
    pub fn set_api_key(&self, provider: &str, key: &str) -> Result<(), AppError> {
        Entry::new("promptflow-stt", provider)
            .map_err(|e| AppError::Storage(e.to_string()))?
            .set_password(key)
            .map_err(|e| AppError::Storage(e.to_string()))
    }

    /// Returns the API key for `provider`, or `None` if no entry exists.
    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, AppError> {
        match Entry::new("promptflow-stt", provider)
            .map_err(|e| AppError::Storage(e.to_string()))?
            .get_password()
        {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Storage(e.to_string())),
        }
    }

    /// Returns `true` if an API key is stored for `provider`.
    pub fn has_api_key(&self, provider: &str) -> Result<bool, AppError> {
        self.get_api_key(provider).map(|opt| opt.is_some())
    }

    /// Removes the API key for `provider` from the OS keychain.
    /// Idempotent: returns `Ok(())` even if no entry existed.
    pub fn delete_api_key(&self, provider: &str) -> Result<(), AppError> {
        match Entry::new("promptflow-stt", provider)
            .map_err(|e| AppError::Storage(e.to_string()))?
            .delete_password()
        {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // already absent — idempotent
            Err(e) => Err(AppError::Storage(e.to_string())),
        }
    }
}

impl Default for KeychainStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Install the in-memory mock credential store so these tests are
    /// hermetic — they pass identically on a developer machine with a real
    /// OS keychain, in a headless CI runner with no secret service, and in a
    /// sandboxed container. Without this, `keyring::Entry` would attempt to
    /// reach the platform secret service (D-Bus / Secret Service on Linux)
    /// and fail with a transport error rather than the expected `NoEntry`.
    ///
    /// `set_default_credential_builder` may only be called before the first
    /// keyring access in the process, so we guard it with `Once`.
    fn init_mock_keystore() {
        INIT.call_once(|| {
            keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        });
    }

    /// `get_api_key` returns `Ok(None)` for a provider that was never stored —
    /// the `NoEntry` case must be mapped to `None`, not propagated as `Err`.
    #[test]
    fn test_get_api_key_missing_returns_none() {
        init_mock_keystore();
        let store = KeychainStore::new();
        let result = store.get_api_key("__promptflow_test_nonexistent_provider__");
        assert!(
            matches!(result, Ok(None)),
            "expected Ok(None), got {result:?}"
        );
    }

    /// `has_api_key` reports `false` (not an error) for a missing provider.
    #[test]
    fn test_has_api_key_missing_does_not_error() {
        init_mock_keystore();
        let store = KeychainStore::new();
        let result = store.has_api_key("__promptflow_test_nonexistent_provider__");
        assert!(
            matches!(result, Ok(false)),
            "expected Ok(false), got {result:?}"
        );
    }

    /// `delete_api_key` is idempotent — deleting a non-existent entry returns `Ok(())`.
    #[test]
    fn test_delete_api_key_idempotent() {
        init_mock_keystore();
        let store = KeychainStore::new();
        let result = store.delete_api_key("__promptflow_test_nonexistent_provider__");
        assert!(
            result.is_ok(),
            "delete_api_key must return Ok(()) even when no entry exists"
        );
    }

    /// Full round-trip: set → get → has → delete → get again.
    ///
    /// Requires a real OS keychain because the in-memory mock store uses
    /// `EntryOnly` persistence (state is not shared across the separate
    /// `keyring::Entry` instances that `KeychainStore` creates per call).
    /// Run locally / on a machine with a secret service via:
    /// `cargo test -- --ignored`.
    #[test]
    #[ignore = "requires a real OS keychain / secret service"]
    fn test_set_get_delete_round_trip() {
        let store = KeychainStore::new();
        let provider = "__promptflow_test_roundtrip__";
        let secret = "test-secret-value-abc123";

        store
            .set_api_key(provider, secret)
            .expect("set_api_key failed");

        // Read back and verify.
        let got = store.get_api_key(provider).expect("get_api_key failed");
        assert_eq!(got.as_deref(), Some(secret));

        // has_api_key must report true.
        assert!(store.has_api_key(provider).expect("has_api_key failed"));

        // Delete, then a second delete is idempotent.
        store
            .delete_api_key(provider)
            .expect("delete_api_key failed");
        store
            .delete_api_key(provider)
            .expect("second delete_api_key failed");

        // After deletion get returns None.
        let after = store
            .get_api_key(provider)
            .expect("get_api_key after delete failed");
        assert!(after.is_none());
    }
}
