/// Public product identity shared by engine diagnostics and applications.
pub const ENGINE_NAME: &str = "Unbiased Bob Engine";
pub const ENGINE_SHORT_NAME: &str = "UBE";
pub const RELEASE_VERSION: &str = "nerd 0.1";

/// Cargo-compatible package version; the public release label is [`RELEASE_VERSION`].
pub const CARGO_PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_branding_uses_the_agreed_engine_and_release_names() {
        assert_eq!(ENGINE_NAME, "Unbiased Bob Engine");
        assert_eq!(ENGINE_SHORT_NAME, "UBE");
        assert_eq!(RELEASE_VERSION, "nerd 0.1");
        assert_eq!(
            format!("{ENGINE_SHORT_NAME} {RELEASE_VERSION}"),
            "UBE nerd 0.1"
        );
    }
}
