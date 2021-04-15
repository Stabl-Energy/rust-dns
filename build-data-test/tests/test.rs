#![forbid(unsafe_code)]

#[test]
fn test() {
    assert!(safe_regex::regex!(br"[0-9]{4}-[0-9]{2}-[0-9]{2}Z")
        .is_match(env!("SOURCE_DATE").as_bytes()));
    assert!(safe_regex::regex!(br"[0-9]{2}:[0-9]{2}:[0-9]{2}Z")
        .is_match(env!("SOURCE_TIME").as_bytes()));
    assert!(
        safe_regex::regex!(br"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z")
            .is_match(env!("SOURCE_TIMESTAMP").as_bytes())
    );
    assert!(safe_regex::regex!(br"[0-9]+").is_match(env!("SOURCE_EPOCH_TIME").as_bytes()));
    assert!(
        safe_regex::regex!(br"[0-9]{4}-[0-9]{2}-[0-9]{2}Z").is_match(env!("BUILD_DATE").as_bytes())
    );
    assert!(
        safe_regex::regex!(br"[0-9]{2}:[0-9]{2}:[0-9]{2}Z").is_match(env!("BUILD_TIME").as_bytes())
    );
    assert!(
        safe_regex::regex!(br"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z")
            .is_match(env!("BUILD_TIMESTAMP").as_bytes())
    );
    assert!(safe_regex::regex!(br"[0-9]+").is_match(env!("BUILD_EPOCH_TIME").as_bytes()));
    assert!(safe_regex::regex!(br"[-_.a-zA-Z0-9]+").is_match(env!("BUILD_HOSTNAME").as_bytes()));
    assert!(safe_regex::regex!(br"[a-zA-Z0-9]+").is_match(env!("GIT_BRANCH").as_bytes()));
    assert!(safe_regex::regex!(br"[0-9a-f]{40}").is_match(env!("GIT_COMMIT").as_bytes()));
    assert!(safe_regex::regex!(br"[0-9a-f]{7}").is_match(env!("GIT_COMMIT_SHORT").as_bytes()));
    assert!(safe_regex::regex!(br"true|false").is_match(env!("GIT_DIRTY").as_bytes()));
    assert!(safe_regex::regex!(
        br"(?:rustc )?([0-9]+\.[0-9]+\.[0-9]+)(?:(-beta)|(-nightly))?(?: .*)?"
    )
    .is_match(env!("RUSTC_VERSION").as_bytes()));
    assert!(safe_regex::regex!(br"[0-9]+\.[0-9]+\.[0-9]+")
        .is_match(env!("RUSTC_VERSION_SEMVER").as_bytes()));
    assert!(safe_regex::regex!(br"stable|beta|nightly").is_match(env!("RUST_CHANNEL").as_bytes()));
}
