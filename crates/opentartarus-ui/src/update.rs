//! One look at GitHub for a newer release, made once when the window opens.
//!
//! The check is deliberately quiet. It asks a single public endpoint, sends
//! nothing about the person running it, and treats every failure as "no
//! news": being offline, a rate limit and a reply we cannot read all leave
//! the window exactly as it was.

use serde::Deserialize;
use std::process::{Command, Stdio};
use std::time::Duration;

/// The release list, newest first. GitHub's `releases/latest` endpoint is
/// not used: it answers 404 while every release is marked a pre-release,
/// which is how this project has published all of them so far, and the
/// check would then never report anything.
const RELEASE_LIST_URL: &str =
    "https://api.github.com/repos/PikkonMG/opentartarus/releases?per_page=5";

/// Where the notice sends the reader: the page a person can read, not the
/// endpoint the program asks.
pub const RELEASES_PAGE_URL: &str = "https://github.com/PikkonMG/opentartarus/releases";

/// GitHub turns away a request that sends no `User-Agent`. This one names
/// the program and the build, and nothing about the machine or the user.
const USER_AGENT_PREFIX: &str = "opentartarus/";

/// The reply format GitHub promises to keep stable.
const GITHUB_ACCEPT: &str = "application/vnd.github+json";

/// Set this variable to any value and no request is made. It gives a
/// packager whose own tools do the updating, and anyone who wants no call
/// out at all, a way to switch the check off without a settings file.
pub const OPT_OUT_ENV: &str = "OPENTARTARUS_NO_UPDATE_CHECK";

/// Short on purpose: the window is already open and waiting on nothing, so
/// a slow network must not leave a task running for minutes.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Major, minor, patch.
const VERSION_PART_COUNT: usize = 3;

/// A release tag may be written `v0.3.0` as well as `0.3.0`.
const TAG_PREFIX: char = 'v';

/// `0.3.0-rc1` is numbered `0.3.0`; the suffix is not part of the number.
const PRE_RELEASE_SEPARATOR: char = '-';

const VERSION_PART_SEPARATOR: char = '.';

/// The build the user is running.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The browser opener every desktop Linux session provides.
const BROWSER_OPENER: &str = "xdg-open";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    /// A draft is visible only to the people who can publish it, so it is
    /// never news. The reply carries the flag; read it rather than guess.
    #[serde(default)]
    draft: bool,
}

/// Reads a version into numbers so `0.10.0` sorts above `0.9.0`, which it
/// does not do as text. `None` when there is no number to read at all.
fn version_parts(text: &str) -> Option<[u64; VERSION_PART_COUNT]> {
    let numbers = text
        .trim()
        .trim_start_matches(TAG_PREFIX)
        .split(PRE_RELEASE_SEPARATOR)
        .next()?;
    let mut parts = [0u64; VERSION_PART_COUNT];
    let mut read = 0;
    for (slot, piece) in parts.iter_mut().zip(
        numbers
            .split(VERSION_PART_SEPARATOR)
            .take(VERSION_PART_COUNT),
    ) {
        *slot = piece.parse().ok()?;
        read += 1;
    }
    (read > 0).then_some(parts)
}

/// The tag of a release newer than `current`, read out of GitHub's reply.
/// `None` when the reply is unreadable, when there is no published release,
/// or when the newest one is the build already running — or older, which is
/// what a development build ahead of the releases sees.
pub fn newer_release_tag(reply: &str, current: &str) -> Option<String> {
    let releases: Vec<Release> = serde_json::from_str(reply).ok()?;
    let running = version_parts(current)?;
    releases
        .into_iter()
        .filter(|release| !release.draft)
        .filter_map(|release| {
            let parts = version_parts(&release.tag_name)?;
            Some((parts, release.tag_name))
        })
        .filter(|(parts, _)| *parts > running)
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, tag)| tag)
}

/// Whether the user or the packager has switched the check off.
pub fn check_disabled() -> bool {
    std::env::var_os(OPT_OUT_ENV).is_some()
}

fn request_latest_release() -> Option<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into();
    agent
        .get(RELEASE_LIST_URL)
        .header(
            "User-Agent",
            format!("{USER_AGENT_PREFIX}{CURRENT_VERSION}"),
        )
        .header("Accept", GITHUB_ACCEPT)
        .call()
        .ok()?
        .body_mut()
        .read_to_string()
        .ok()
}

/// Asks GitHub once and reports a newer release, if there is one. The call
/// blocks, so it runs on a thread of its own and never holds up a redraw.
pub async fn latest_release_tag() -> Option<String> {
    if check_disabled() {
        return None;
    }
    let reply = tokio::task::spawn_blocking(request_latest_release)
        .await
        .ok()??;
    newer_release_tag(&reply, CURRENT_VERSION)
}

/// Hands the releases page to the desktop's browser. Nothing is reported
/// back: if no opener is installed, the notice still shows the address.
pub fn open_releases_page() {
    let _ = Command::new(BROWSER_OPENER)
        .arg(RELEASES_PAGE_URL)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(tag: &str) -> String {
        format!(r#"[{{"tag_name":"{tag}","draft":false,"name":"ignored"}}]"#)
    }

    #[test]
    fn a_higher_release_is_news() {
        assert_eq!(
            newer_release_tag(&reply("v0.3.0"), "0.2.1"),
            Some(String::from("v0.3.0"))
        );
        assert_eq!(
            newer_release_tag(&reply("0.2.2"), "0.2.1"),
            Some(String::from("0.2.2"))
        );
        assert_eq!(
            newer_release_tag(&reply("1.0.0"), "0.9.9"),
            Some(String::from("1.0.0"))
        );
    }

    #[test]
    fn the_running_build_and_older_ones_are_not_news() {
        assert_eq!(newer_release_tag(&reply("v0.2.1"), "0.2.1"), None);
        assert_eq!(newer_release_tag(&reply("0.2.0"), "0.2.1"), None);
        assert_eq!(
            newer_release_tag(&reply("v0.1.9"), "0.2.1"),
            None,
            "a development build ahead of the newest release must stay quiet"
        );
    }

    #[test]
    fn version_numbers_are_compared_as_numbers_not_as_text() {
        assert_eq!(
            newer_release_tag(&reply("v0.10.0"), "0.9.0"),
            Some(String::from("v0.10.0")),
            "0.10.0 is newer than 0.9.0, though it sorts below it as text"
        );
        assert_eq!(newer_release_tag(&reply("v0.9.0"), "0.10.0"), None);
    }

    #[test]
    fn a_pre_release_suffix_is_not_part_of_the_number() {
        assert_eq!(version_parts("v0.3.0-rc1"), Some([0, 3, 0]));
        assert_eq!(newer_release_tag(&reply("v0.2.1-rc1"), "0.2.1"), None);
    }

    #[test]
    fn a_short_version_fills_the_missing_parts_with_zero() {
        assert_eq!(version_parts("0.3"), Some([0, 3, 0]));
        assert_eq!(version_parts("2"), Some([2, 0, 0]));
    }

    #[test]
    fn an_unreadable_reply_is_no_news() {
        assert_eq!(newer_release_tag("not json", "0.2.1"), None);
        assert_eq!(newer_release_tag("[]", "0.2.1"), None, "no release yet");
        assert_eq!(newer_release_tag("{}", "0.2.1"), None);
        assert_eq!(newer_release_tag(&reply("nightly"), "0.2.1"), None);
        assert_eq!(version_parts(""), None);
    }

    #[test]
    fn the_request_names_the_program_and_not_the_user() {
        let agent = format!("{USER_AGENT_PREFIX}{CURRENT_VERSION}");
        assert_eq!(agent, format!("opentartarus/{CURRENT_VERSION}"));
        assert!(!CURRENT_VERSION.is_empty());
    }

    #[test]
    fn the_reader_is_sent_to_the_page_not_to_the_endpoint() {
        assert!(RELEASES_PAGE_URL.starts_with("https://github.com/"));
        assert!(RELEASE_LIST_URL.starts_with("https://api.github.com/"));
        assert_ne!(RELEASES_PAGE_URL, RELEASE_LIST_URL);
    }

    #[test]
    fn a_pre_release_still_counts_as_a_release() {
        // Every release this project has published is marked a pre-release.
        let listed = r#"[{"tag_name":"v0.3.0","draft":false,"prerelease":true}]"#;
        assert_eq!(
            newer_release_tag(listed, "0.2.1"),
            Some(String::from("v0.3.0"))
        );
    }

    #[test]
    fn the_fields_github_adds_are_ignored() {
        // The real reply carries some thirty fields per release. Only two
        // are read, and an added field must not break the check.
        let listed = r#"[{"url":"https://api.github.com/x","id":454132957,
                          "tag_name":"v0.3.0","draft":false,"prerelease":true,
                          "published_at":"2026-11-22T02:06:42Z",
                          "assets":[],"body":"notes"}]"#;
        assert_eq!(
            newer_release_tag(listed, "0.2.1"),
            Some(String::from("v0.3.0"))
        );
    }

    #[test]
    fn a_draft_is_never_news() {
        let listed = r#"[{"tag_name":"v0.9.0","draft":true},
                         {"tag_name":"v0.3.0","draft":false}]"#;
        assert_eq!(
            newer_release_tag(listed, "0.2.1"),
            Some(String::from("v0.3.0")),
            "the draft is skipped and the newest published one reported"
        );
        let only_draft = r#"[{"tag_name":"v0.9.0","draft":true}]"#;
        assert_eq!(newer_release_tag(only_draft, "0.2.1"), None);
    }

    #[test]
    fn the_highest_version_wins_whatever_order_the_list_arrives_in() {
        let listed = r#"[{"tag_name":"v0.3.0","draft":false},
                         {"tag_name":"v0.4.0","draft":false},
                         {"tag_name":"v0.2.0","draft":false}]"#;
        assert_eq!(
            newer_release_tag(listed, "0.2.1"),
            Some(String::from("v0.4.0"))
        );
    }
}
