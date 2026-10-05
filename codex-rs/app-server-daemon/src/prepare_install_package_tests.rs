//! Exercises daemon installation with the complete native release artifact.

use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use codex_install_context::CodexPackageManifest;
use pretty_assertions::assert_eq;

use super::InstallMode;
use super::prepare_from_package;
use crate::Daemon;
use crate::managed_install;
use crate::settings::DaemonSettings;

#[tokio::test]
#[ignore = "requires CCODEX_TEST_PACKAGE_DIR from the native release build"]
async fn packaged_cli_seeds_runnable_daemon() -> Result<()> {
    let source = PathBuf::from(
        std::env::var_os("CCODEX_TEST_PACKAGE_DIR").context("missing native test package")?,
    );
    let manifest: CodexPackageManifest =
        serde_json::from_slice(&std::fs::read(source.join("codex-package.json"))?)?;
    let running_exe = source.join("bin").join(if cfg!(windows) {
        "ccodex.exe"
    } else {
        "ccodex"
    });
    let home = tempfile::TempDir::new()?;
    let state = home.path().join("app-server-daemon");
    let daemon = Daemon {
        log_diagnostics: false,
        socket_path: state.join("app-server.sock"),
        pid_file: state.join(crate::DAEMON_PID_FILE_NAME),
        update_pid_file: state.join(crate::DAEMON_UPDATE_PID_FILE_NAME),
        operation_lock_file: state.join("daemon.lock"),
        settings_file: state.join("settings.json"),
        managed_codex_bin: managed_install::managed_codex_bin(home.path()),
    };
    let settings = DaemonSettings::default();
    assert!(
        prepare_from_package(
            &daemon,
            &settings,
            InstallMode::Missing,
            Some(&source),
            &running_exe,
            |_| panic!("initial installation does not require replacement confirmation"),
        )
        .await?
    );

    let selected = managed_install::managed_codex_bin(home.path());
    assert_eq!(
        managed_install::managed_codex_version(&selected).await?,
        manifest.version.to_string()
    );
    assert_eq!(
        managed_install::executable_identity(&selected).await?,
        managed_install::executable_identity(&running_exe).await?
    );
    assert!(managed_install::supports_daemon_update_loop(&selected).await);
    super::validate_package(&home.path().join("packages/app-server-daemon/current"))?;
    Ok(())
}
