//! Pure decision core of petoi-the-dog-walk.
//!
//! `no_std`: this crate cannot touch files, the network, clocks, threads or randomness, so every
//! function here is a pure function of its arguments. See docs/design/02-architecture.md §2.1.
#![cfg_attr(not(test), no_std)]

/// Crate version, as a placeholder until the first real types land (roadmap phase 1a).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_matches_workspace_version() {
        assert_eq!(VERSION, "0.0.0");
    }
}
