//! What this adapter says it is, and the kit that serves its contract.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::dashboard;

use crate::dashy::Dashy;

/// Where Dashy's configuration directory is, in the adapter's container, which the
/// adapter shares.
pub const ADDRESS: &str = "/dashy";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "Dashy";

/// The Dashy release this adapter's recordings were taken from.
pub const RELEASE: &str = "4.9.0";

include!(concat!(env!("OUT_DIR"), "/fronted.rs"));

/// What this adapter says it is: the contract it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![lemonfiber_contract::spoken(
            dashboard::show::CAPABILITY,
            dashboard::show::MAJOR,
        )],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contract, served from `dashy` under `key`, ready while Dashy's configuration
/// directory is there to write.
#[must_use]
pub fn kit(key: String, dashy: Arc<Dashy>) -> Kit {
    let asked = Arc::clone(&dashy);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(dashboard::show::served(dashy))
}

#[cfg(test)]
mod tests;
