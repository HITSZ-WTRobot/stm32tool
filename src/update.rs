use crate::cli::UpdateArgs;
use anyhow::{Context, anyhow, bail};
use dialoguer::Confirm;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

const RELEASE_API_URL: &str =
    "https://api.github.com/repos/HITSZ-WTRobot/stm32tool/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DOWNLOAD_CHUNK_SIZE: usize = 64 * 1024;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

pub fn run(args: UpdateArgs) -> anyhow::Result<()> {
    debug!(?args, "Running update command");

    let release = fetch_latest_release()?;
    let latest = parse_tag_version(&release.tag_name)?;
    let current = Version::parse(CURRENT_VERSION)
        .with_context(|| format!("Invalid built-in version {CURRENT_VERSION}"))?;
    debug!(current = %current, latest = %latest, "Resolved local and remote versions");

    if latest <= current {
        info!(
            "stm32tool is up to date (v{current}); latest release is {}",
            release.tag_name
        );
        return Ok(());
    }

    info!("New version available: v{current} -> {}", release.tag_name);

    if args.check {
        debug!("Check-only run; skipping download");
        return Ok(());
    }

    let asset = select_asset(&release)?;
    let expected_digest = asset_digest(asset)?;

    if !args.yes {
        if !io::stderr().is_terminal() {
            bail!("Confirmation required but stderr is not a terminal; re-run with --yes");
        }

        let confirmed = Confirm::new()
            .with_prompt(format!("Update stm32tool to {}?", release.tag_name))
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            info!("Update cancelled");
            return Ok(());
        }
    }

    let exe =
        std::env::current_exe().context("Failed to locate the running stm32tool executable")?;
    if is_dev_build_path(&exe) {
        bail!(
            "{} is a Cargo build output; install a release build before running `stm32tool update`",
            exe.display()
        );
    }

    let exe_name = exe
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let temp = exe.with_file_name(format!("{exe_name}.new"));

    info!("Downloading {}", asset.name);
    if let Err(error) = download_asset(asset, &temp, &expected_digest) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    debug!(temp = %temp.display(), "Downloaded and verified release asset");

    replace_binary(&temp, &exe)?;

    info!("Updated stm32tool to {}", release.tag_name);
    warn!("Restart any shell or IDE that has the old binary cached in `PATH`.");

    Ok(())
}

/// Queries the GitHub API for the newest release of this repository.
fn fetch_latest_release() -> anyhow::Result<Release> {
    info!("Checking the latest stm32tool release");
    let mut response = http_agent()
        .get(RELEASE_API_URL)
        .header("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("Failed to query {RELEASE_API_URL}"))?;

    let body = response
        .body_mut()
        .read_to_string()
        .context("Failed to read the GitHub release response")?;
    debug!(bytes = body.len(), "Fetched release metadata");

    serde_json::from_str(&body).context("Failed to parse the GitHub release response")
}

/// GitHub requires a `User-Agent`; the timeout bounds a stalled download.
fn http_agent() -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .user_agent(concat!("stm32tool/", env!("CARGO_PKG_VERSION")))
            .timeout_global(Some(DOWNLOAD_TIMEOUT))
            .build(),
    )
}

/// Streams the asset to `target` while hashing it, then verifies the digest.
fn download_asset(
    asset: &ReleaseAsset,
    target: &Path,
    expected_digest: &str,
) -> anyhow::Result<()> {
    let mut response = http_agent()
        .get(&asset.browser_download_url)
        .call()
        .with_context(|| format!("Failed to download {}", asset.browser_download_url))?;

    let mut file = fs::File::create(target)
        .with_context(|| format!("Failed to create {}", target.display()))?;
    let mut reader = response.body_mut().as_reader();
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; DOWNLOAD_CHUNK_SIZE];

    loop {
        let read = reader
            .read(&mut buffer)
            .with_context(|| format!("Failed while downloading {}", asset.name))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read])
            .with_context(|| format!("Failed to write {}", target.display()))?;
    }

    file.sync_all()
        .with_context(|| format!("Failed to flush {}", target.display()))?;

    let actual_digest = to_hex(&hasher.finalize());
    if actual_digest != expected_digest {
        bail!(
            "Checksum mismatch for {}: expected {expected_digest}, got {actual_digest}",
            asset.name
        );
    }
    debug!(digest = %actual_digest, "Verified release asset checksum");

    Ok(())
}

/// Picks the asset published for the current platform.
fn select_asset(release: &Release) -> anyhow::Result<&ReleaseAsset> {
    let expected = asset_name(std::env::consts::OS, &release.tag_name).ok_or_else(|| {
        anyhow!(
            "No release asset is published for {}; update this platform manually",
            std::env::consts::OS
        )
    })?;
    debug!(expected, "Looking for the platform release asset");

    release
        .assets
        .iter()
        .find(|asset| asset.name == expected)
        .ok_or_else(|| {
            let available = release
                .assets
                .iter()
                .map(|asset| asset.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow!(
                "Release {} does not contain {expected}; available assets: {available}",
                release.tag_name
            )
        })
}

/// Only the platforms that `.github/workflows/publish.yml` builds have assets.
fn asset_name(os: &str, tag: &str) -> Option<String> {
    match os {
        "windows" => Some(format!("stm32tool-windows-{tag}.exe")),
        "linux" => Some(format!("stm32tool-linux-{tag}")),
        _ => None,
    }
}

fn parse_tag_version(tag: &str) -> anyhow::Result<Version> {
    let version = tag.trim_start_matches(['v', 'V']);
    Version::parse(version).with_context(|| format!("Unsupported release tag {tag}"))
}

/// Refuses to install an asset whose digest GitHub does not report.
fn asset_digest(asset: &ReleaseAsset) -> anyhow::Result<String> {
    let digest = asset.digest.as_deref().ok_or_else(|| {
        anyhow!(
            "Release asset {} has no sha256 digest; refusing to install an unverified binary",
            asset.name
        )
    })?;

    parse_sha256_digest(digest)
        .ok_or_else(|| anyhow!("Unsupported digest format for {}: {digest}", asset.name))
}

fn parse_sha256_digest(digest: &str) -> Option<String> {
    let hex = digest.strip_prefix("sha256:")?.to_ascii_lowercase();
    let is_hex = hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit());
    is_hex.then_some(hex)
}

fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").expect("writing into a String cannot fail");
    }
    hex
}

/// Cargo output directories are not install locations; replacing them destroys dev builds.
fn is_dev_build_path(exe: &Path) -> bool {
    exe.ancestors()
        .any(|ancestor| ancestor.file_name() == Some(OsStr::new("target")))
}

fn replace_binary(temp: &Path, exe: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(temp, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to mark {} executable", temp.display()))?;
        fs::rename(temp, exe).with_context(|| format!("Failed to replace {}", exe.display()))?;
        Ok(())
    }

    #[cfg(windows)]
    {
        // A running executable cannot be overwritten, but it can be renamed aside.
        let exe_name = exe
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let backup = exe.with_file_name(format!("{exe_name}.old"));
        let _ = fs::remove_file(&backup);

        fs::rename(exe, &backup)
            .with_context(|| format!("Failed to move {} aside", exe.display()))?;
        if let Err(error) = fs::rename(temp, exe) {
            let _ = fs::rename(&backup, exe);
            return Err(error).with_context(|| {
                format!(
                    "Failed to replace {}; restored the previous binary",
                    exe.display()
                )
            });
        }
        debug!(backup = %backup.display(), "Previous binary kept for deletion on the next run");

        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        bail!("Updating stm32tool is not supported on this platform")
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_name, is_dev_build_path, parse_sha256_digest, parse_tag_version};
    use semver::Version;
    use std::path::Path;

    #[test]
    fn resolves_platform_asset_names() {
        assert_eq!(
            asset_name("windows", "v0.4.0").as_deref(),
            Some("stm32tool-windows-v0.4.0.exe")
        );
        assert_eq!(
            asset_name("linux", "v0.4.0").as_deref(),
            Some("stm32tool-linux-v0.4.0")
        );
        assert_eq!(asset_name("macos", "v0.4.0"), None);
    }

    #[test]
    fn parses_prefixed_release_tags() {
        assert_eq!(parse_tag_version("v0.4.0").unwrap(), Version::new(0, 4, 0));
        assert_eq!(parse_tag_version("V1.2.3").unwrap(), Version::new(1, 2, 3));
        assert!(parse_tag_version("nightly").is_err());
    }

    #[test]
    fn orders_fix_releases_below_their_base_version() {
        assert!(parse_tag_version("v0.3.1").unwrap() > Version::new(0, 3, 0));
        assert!(parse_tag_version("v0.3.0-fix1").unwrap() < Version::new(0, 3, 0));
    }

    #[test]
    fn accepts_only_sha256_digests() {
        let hex = "a".repeat(64);
        assert_eq!(
            parse_sha256_digest(&format!("sha256:{hex}")).as_deref(),
            Some(hex.as_str())
        );
        assert_eq!(parse_sha256_digest("sha256:abc"), None);
        assert_eq!(parse_sha256_digest(&hex), None);
        assert_eq!(parse_sha256_digest("md5:abc"), None);
    }

    #[test]
    fn detects_cargo_build_outputs() {
        assert!(is_dev_build_path(Path::new("/repo/target/debug/stm32tool")));
        assert!(is_dev_build_path(Path::new(
            "/repo/target/x86_64-pc-windows-msvc/release/stm32tool.exe"
        )));
        assert!(!is_dev_build_path(Path::new("/usr/local/bin/stm32tool")));
        assert!(!is_dev_build_path(Path::new(
            "/home/user/.local/bin/stm32tool"
        )));
    }
}
