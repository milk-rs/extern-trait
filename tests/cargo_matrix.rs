use std::{env, path::PathBuf, time::Duration};

#[test]
#[ignore = "host-only cargo invocation test"]
fn cargo_matrix() {
    let cases = trycmd::TestCases::new();
    cases
        .register_bin("cargo", PathBuf::from(env::var("CARGO").unwrap()))
        .timeout(Duration::from_secs(180))
        .case("tests/cmd/default-requires-feature.toml");

    if rustversion::cfg!(nightly) {
        cases.case("tests/cmd/nightly-weak-override.toml");
    } else {
        cases.case("tests/cmd/nightly-weak-requires-nightly.toml");
    }

    cases.run();
}
