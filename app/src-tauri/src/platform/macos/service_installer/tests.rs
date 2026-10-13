use std::fs;

use super::*;

const PRIVILEGES_CLAUSE: &str = " with administrator privileges";
const PROMPT_CLAUSE: &str =
    r#" with prompt "Grumpo needs your password to install the service that blocks sites.""#;
const HOSTILE_NAME: &str = "a b'c\"d $(exit 1) `id` ; &|<>";
const OWNER_UID: u32 = 501;
const BOOTSTRAP_AT: usize = 8;
const LOCKED_WORK_DIRS: &str = "/private/var/tmp/grumpo-install.";

fn bundle_with(files: &[&str]) -> (tempfile::TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let contents = root.path().join("Grumpo.app").join(CONTENTS_DIR);
    for file in files {
        let path = contents.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }
    let executable = contents.join(EXECUTABLE_DIR).join("Grumpo");
    (root, executable)
}

fn complete_bundle() -> (tempfile::TempDir, PathBuf) {
    bundle_with(&[INSTALLER_SCRIPT, DAEMON_BINARY])
}

fn bundled_in(root: &Path) -> BundledInstaller {
    BundledInstaller {
        installer_dir: root.join("installer"),
        daemon: root.join("focusd"),
    }
}

fn text(arguments: &[OsString]) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect()
}

fn status_line(code: i32) -> String {
    format!("execution error: something went wrong ({code})\n")
}

#[test]
fn a_complete_bundle_locates_the_script_and_the_daemon() {
    let (root, executable) = complete_bundle();
    let contents = root.path().join("Grumpo.app").join(CONTENTS_DIR);

    let bundled = BundledInstaller::locate(&executable).unwrap();

    assert_eq!(bundled.installer_dir, contents.join(INSTALLER_DIR));
    assert_eq!(bundled.daemon, contents.join(DAEMON_BINARY));
}

#[test]
fn a_bundle_without_the_installer_script_is_unavailable() {
    let (_root, executable) = bundle_with(&[DAEMON_BINARY]);
    assert_eq!(BundledInstaller::locate(&executable), None);
}

#[test]
fn a_bundle_without_the_daemon_is_unavailable() {
    let (_root, executable) = bundle_with(&[INSTALLER_SCRIPT]);
    assert_eq!(BundledInstaller::locate(&executable), None);
}

#[test]
fn a_build_run_outside_a_bundle_is_unavailable() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("target").join("debug").join("focus-app");
    assert_eq!(BundledInstaller::locate(&executable), None);
    assert_eq!(BundledInstaller::locate(Path::new("/")), None);
}

#[test]
fn an_executable_outside_contents_macos_does_not_vouch_for_the_files() {
    let (_root, executable) = complete_bundle();
    let contents = executable.parent().unwrap().parent().unwrap();
    let elsewhere = contents.join("Resources").join("Grumpo");
    assert_eq!(BundledInstaller::locate(&elsewhere), None);
}

#[test]
fn root_starts_from_an_empty_environment_and_a_fixed_bootstrap() {
    let (_root, executable) = complete_bundle();
    let bundled = BundledInstaller::locate(&executable).unwrap();

    let arguments = text(&bundled.arguments(OWNER_UID));

    assert_eq!(
        arguments[..8],
        [
            ENV,
            "-i",
            "PATH=/usr/bin:/bin:/usr/sbin:/sbin",
            "HOME=/var/root",
            SHELL,
            "--noprofile",
            "--norc",
            "-c"
        ]
    );
    assert_eq!(arguments[BOOTSTRAP_AT], BOOTSTRAP);
    assert_eq!(arguments[BOOTSTRAP_AT + 1], BOOTSTRAP_NAME);
    assert_eq!(
        arguments[BOOTSTRAP_AT + 2],
        bundled.installer_dir.to_string_lossy()
    );
    assert_eq!(
        arguments[BOOTSTRAP_AT + 3],
        bundled.daemon.to_string_lossy()
    );
    assert_eq!(arguments[BOOTSTRAP_AT + 4], "501");
    assert_eq!(arguments.len(), BOOTSTRAP_AT + 5);
}

#[test]
fn the_script_source_is_the_fixed_text_and_the_values_follow_as_arguments() {
    let (_root, executable) = complete_bundle();
    let bundled = BundledInstaller::locate(&executable).unwrap();
    let command = osascript(&SCRIPT, &bundled.arguments(OWNER_UID));

    let all = text(&command.get_args().map(OsString::from).collect::<Vec<_>>());
    let (script_part, values) = all.split_at(SCRIPT.len() * 2);
    let inline: Vec<&str> = script_part
        .chunks(2)
        .map(|pair| {
            assert_eq!(pair[0], "-e");
            pair[1].as_str()
        })
        .collect();

    assert_eq!(command.get_program(), OSASCRIPT);
    assert_eq!(inline, SCRIPT);
    assert_eq!(values, text(&bundled.arguments(OWNER_UID)));
}

#[test]
fn every_value_is_shell_quoted_before_it_runs_as_root() {
    assert!(SCRIPT.contains(&"set end of quoted to quoted form of item n of argv"));
    assert!(SCRIPT
        .iter()
        .any(|line| line.ends_with(PRIVILEGES_CLAUSE) && line.contains("quoted as text")));
}

#[test]
fn the_password_dialog_explains_who_is_asking_and_why() {
    assert!(SCRIPT.iter().any(|line| line
        .strip_suffix(PRIVILEGES_CLAUSE)
        .is_some_and(|rest| rest.ends_with(PROMPT_CLAUSE))));
}

#[test]
fn hostile_paths_reach_the_bootstrap_as_plain_arguments() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(HOSTILE_NAME);
    fs::create_dir(&dir).unwrap();
    let record = root.path().join("record");
    let bundled = bundled_in(&dir);
    let mut arguments = bundled.arguments(OWNER_UID);
    let capture = format!("printf '%s\\n' \"$@\" > '{}'", record.display());
    arguments[BOOTSTRAP_AT] = capture.into();
    let unprivileged: Vec<&str> = SCRIPT
        .iter()
        .map(|line| line.strip_suffix(PRIVILEGES_CLAUSE).unwrap_or(line))
        .collect();

    let status = osascript(&unprivileged, &arguments).status().unwrap();

    assert!(status.success());
    let recorded = fs::read_to_string(&record).unwrap();
    assert_eq!(
        recorded.lines().collect::<Vec<_>>(),
        [
            bundled.installer_dir.to_str().unwrap(),
            bundled.daemon.to_str().unwrap(),
            "501"
        ]
    );
}

const FAKE_INSTALLER: &str = r#"#!/bin/bash
{
  printf 'script=%s\n' "$0"
  printf 'args=%s\n' "$*"
  printf 'home=%s\n' "${HOME-}"
  printf 'path=%s\n' "${PATH-}"
  printf 'tmpdir=%s\n' "${TMPDIR-unset}"
  printf 'marker=%s\n' "${GRUMPO_TEST_MARKER-unset}"
  printf 'linked=%s\n' "$(test -L "$(dirname "$0")/lib/outside.sh" && echo yes || echo no)"
  printf 'lib=%s\n' "$(cat "$(dirname "$0")/lib/outside.sh" 2>/dev/null)"
  printf 'workmode=%s\n' "$(stat -f %Lp "$(dirname "$0")/../..")"
} > '@RECORD@'
"#;

struct Sandbox {
    root: tempfile::TempDir,
    bundled: BundledInstaller,
    record: PathBuf,
}

impl Sandbox {
    fn new(daemon_source: &Path) -> Self {
        let root = tempfile::tempdir().unwrap();
        let installer_dir = root.path().join("installer");
        let scripts = installer_dir.join("scripts");
        fs::create_dir_all(scripts.join("lib")).unwrap();
        let record = root.path().join("record");
        let script = FAKE_INSTALLER.replace("@RECORD@", &record.display().to_string());
        fs::write(scripts.join("install.sh"), script).unwrap();
        let outside = root.path().join("outside.sh");
        fs::write(&outside, "outside").unwrap();
        std::os::unix::fs::symlink(&outside, scripts.join("lib").join("outside.sh")).unwrap();
        let daemon = root.path().join("focusd");
        fs::copy(daemon_source, &daemon).unwrap();
        let bundled = BundledInstaller {
            installer_dir,
            daemon,
        };
        Self {
            root,
            bundled,
            record,
        }
    }

    fn run(&self) -> std::process::ExitStatus {
        let arguments = self.bundled.arguments(OWNER_UID);
        Command::new(&arguments[0])
            .args(&arguments[1..])
            .env("GRUMPO_TEST_MARKER", "leaked")
            .env("TMPDIR", self.root.path())
            .status()
            .unwrap()
    }

    fn recorded(&self) -> String {
        fs::read_to_string(&self.record).unwrap_or_default()
    }
}

fn recorded_value<'a>(record: &'a str, key: &str) -> &'a str {
    record
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("no {key} in {record}"))
}

const SIGNED_BINARY: &str = "/usr/bin/true";

#[test]
fn the_installer_runs_from_a_private_copy_with_a_clean_environment() {
    let sandbox = Sandbox::new(Path::new(SIGNED_BINARY));

    let status = sandbox.run();

    let record = sandbox.recorded();
    assert!(status.success(), "{record}");
    let script = recorded_value(&record, "script");
    assert!(script.starts_with(LOCKED_WORK_DIRS), "{script}");
    assert!(!script.contains(sandbox.root.path().to_str().unwrap()));
    let args = recorded_value(&record, "args");
    assert!(
        args.starts_with(&format!("--binary {LOCKED_WORK_DIRS}")),
        "{args}"
    );
    assert!(args.ends_with("/focusd --owner 501"), "{args}");
    assert_eq!(recorded_value(&record, "home"), "/var/root");
    assert_eq!(recorded_value(&record, "tmpdir"), "unset");
    assert_eq!(recorded_value(&record, "marker"), "unset");
    assert_eq!(
        recorded_value(&record, "path"),
        "/usr/bin:/bin:/usr/sbin:/sbin"
    );
    assert_eq!(recorded_value(&record, "workmode"), "700");
}

#[test]
fn links_inside_the_bundle_are_copied_as_files_and_cannot_point_root_outside() {
    let sandbox = Sandbox::new(Path::new(SIGNED_BINARY));

    sandbox.run();

    let record = sandbox.recorded();
    assert_eq!(recorded_value(&record, "linked"), "no");
    assert_eq!(recorded_value(&record, "lib"), "outside");
}

#[test]
fn the_private_copy_is_removed_when_the_installer_ends() {
    let sandbox = Sandbox::new(Path::new(SIGNED_BINARY));

    sandbox.run();

    let record = sandbox.recorded();
    let script = Path::new(recorded_value(&record, "script"));
    let work = script.ancestors().nth(3).unwrap();
    assert!(!work.exists(), "{}", work.display());
}

#[test]
fn a_daemon_that_fails_the_signature_check_stops_the_install_before_the_installer_runs() {
    let unsigned = tempfile::NamedTempFile::new().unwrap();
    fs::write(unsigned.path(), "#!/bin/sh\nexit 0\n").unwrap();
    let sandbox = Sandbox::new(unsigned.path());

    let status = sandbox.run();

    assert!(!status.success());
    assert_eq!(sandbox.recorded(), "", "the installer never ran");
}

#[test]
fn a_zero_exit_is_an_installed_service() {
    assert_eq!(outcome_of(true, "").unwrap(), InstallOutcome::Installed);
}

#[test]
fn dismissing_the_prompt_is_a_cancellation() {
    let stderr = "133:177: execution error: User canceled. (-128)\n";
    assert_eq!(
        outcome_of(false, stderr).unwrap(),
        InstallOutcome::Cancelled
    );
}

#[test]
fn any_other_failure_is_a_failed_install() {
    for stderr in [
        status_line(1),
        status_line(-60007),
        "boom".to_owned(),
        String::new(),
    ] {
        let error = outcome_of(false, &stderr).unwrap_err();
        assert!(matches!(error, ServiceInstallError::Failed(_)), "{stderr}");
    }
}

#[test]
fn the_failure_carries_the_status_and_none_of_the_installer_output() {
    let stderr = "error: no executable focusd at /Users/someone/Grumpo.app (7)";
    let error = outcome_of(false, stderr).unwrap_err();
    assert_eq!(
        error.to_string(),
        "the installer failed: the installer ended with status 7"
    );
}

#[test]
fn installer_output_cannot_pose_as_a_cancellation() {
    let stderr = "error: refused (-128) by the installer\nexecution error: boom (1)\n";
    assert!(outcome_of(false, stderr).is_err());
}

#[test]
fn the_status_is_read_from_the_last_parenthesis_group() {
    assert_eq!(trailing_error_number("x (-128)"), Some(-128));
    assert_eq!(trailing_error_number("x (3) (4)\n"), Some(4));
    assert_eq!(trailing_error_number("x (abc)"), None);
    assert_eq!(trailing_error_number("x (3) done"), None);
    assert_eq!(trailing_error_number(""), None);
}

#[test]
fn only_the_end_of_a_long_output_is_logged() {
    let long = format!("{}tail (1)\n", "a".repeat(5000));
    let kept = tail(&long);
    assert!(kept.ends_with("tail (1)"));
    assert_eq!(kept.chars().count(), LOGGED_TAIL_CHARS + 1);
    assert_eq!(tail("short"), "short");
    assert_eq!(
        tail(&"é".repeat(5000)).chars().count(),
        LOGGED_TAIL_CHARS + 1
    );
}

#[test]
fn the_spawn_and_timeout_failures_say_what_happened() {
    let timed_out = failed_to_run(RunError::Timeout, false).to_string();
    assert!(timed_out.contains("did not finish"));
    let spawn = failed_to_run(
        RunError::Spawn(std::io::Error::other("no osascript")),
        false,
    );
    assert!(spawn.to_string().contains("no osascript"));
}

#[test]
fn a_timeout_while_root_still_installs_is_busy_and_not_a_failure() {
    let error = failed_to_run(RunError::Timeout, true);
    assert!(matches!(error, ServiceInstallError::Busy), "{error}");
}

#[test]
fn a_spawn_failure_stays_a_failure_even_with_a_lock_around() {
    let error = failed_to_run(RunError::Spawn(std::io::Error::other("no osascript")), true);
    assert!(matches!(error, ServiceInstallError::Failed(_)), "{error}");
}

#[test]
fn a_live_process_counts_as_running_and_a_gone_one_does_not() {
    let own = i32::try_from(std::process::id()).unwrap();
    assert!(process_exists(own));
    assert!(process_exists(1), "launchd answers EPERM to a user");
    let mut child = Command::new("/usr/bin/true").spawn().unwrap();
    let gone = i32::try_from(child.id()).unwrap();
    child.wait().unwrap();
    assert!(!process_exists(gone));
}

#[test]
fn home_folders_are_blanked_in_the_logged_output() {
    let output =
        "error: cannot find /Users/jane.doe/Library/x and \"/Users/jane.doe\" then /Users/bob\nend";
    assert_eq!(
        redact_homes(output),
        "error: cannot find /Users/~/Library/x and \"/Users/~\" then /Users/~\nend"
    );
    assert_eq!(redact_homes("nothing here"), "nothing here");
    assert_eq!(redact_homes("/Users/"), "/Users/~");
}
