//! Shared `reqwest` client.
//!
//! Every provider/STT engine previously built its own `reqwest::Client` per
//! instance (and a new instance is created on every request), losing connection
//! pooling and — worse — running with **no timeout**, so a hung upstream would
//! block the request indefinitely. A single shared client fixes both: it is
//! cheap to clone (internally reference-counted) and carries sane connect/read
//! timeouts that satisfy the contract in `docs/specs/06_AI_INTEGRATIONS.md` §3
//! (30 s for enhancement; STT overrides per-request to 60 s).

use std::sync::OnceLock;
use std::time::Duration;

/// Returns the process-wide shared HTTP client (built once, cheap to clone).
pub fn client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .build()
                .expect("failed to build shared HTTP client")
        })
        .clone()
}
