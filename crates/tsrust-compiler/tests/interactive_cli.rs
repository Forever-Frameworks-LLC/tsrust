mod support;

#[test]
fn redirected_bare_command_prints_help_without_prompting_or_writing() {
    let root = support::TempDirectory::new();
    let output = support::run(support::cli().current_dir(&root.0).stdin(std::process::Stdio::null()), 10);
    support::success(&output);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("interactive command menu") && text.contains("Commands:"));
    assert!(output.stderr.is_empty());
    assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
    for args in [vec!["new"], vec!["ui", "add"], vec!["ui", "add", "--json"], vec!["ui", "add", "--yes"]] {
        let output = support::run(support::cli().args(args).current_dir(&root.0).stdin(std::process::Stdio::null()), 10);
        assert!(!output.status.success());
        assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
    }
}

#[cfg(unix)]
mod terminal {
    use super::support;
    use std::{fs::File, io::{Read, Write}, os::fd::FromRawFd, process::{Child, Stdio}, time::{Duration, Instant}};

    struct Terminal { child: Child, master: File, text: String, consumed: usize }
    impl Terminal {
        fn start(root: &std::path::Path, args: &[&str]) -> Self {
            let (mut master, mut slave) = (-1, -1);
            assert_eq!(unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()) }, 0);
            let master = unsafe { File::from_raw_fd(master) };
            let slave = unsafe { File::from_raw_fd(slave) };
            use std::os::fd::AsRawFd;
            let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
            assert!(flags >= 0);
            assert_eq!(unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) }, 0);
            let child = support::cli().args(args).current_dir(root)
                .stdin(Stdio::from(slave.try_clone().unwrap()))
                .stdout(Stdio::from(slave.try_clone().unwrap()))
                .stderr(Stdio::from(slave)).spawn().unwrap();
            Self { child, master, text: String::new(), consumed: 0 }
        }
        fn read(&mut self) {
            let mut buffer = [0; 8192];
            match self.master.read(&mut buffer) {
                Ok(count) => self.text.push_str(&String::from_utf8_lossy(&buffer[..count])),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock || error.raw_os_error() == Some(libc::EIO) => {},
                Err(error) => panic!("{error}"),
            }
        }
        fn answer(&mut self, prompt: &str, answer: &str) {
            let deadline = Instant::now() + Duration::from_secs(20);
            loop {
                self.read();
                if let Some(offset) = self.text[self.consumed..].find(prompt) {
                    self.consumed += offset + prompt.len();
                    self.master.write_all(answer.as_bytes()).unwrap();
                    return;
                }
                assert!(self.child.try_wait().unwrap().is_none(), "Exited before {prompt}: {}", self.text);
                assert!(Instant::now() < deadline, "Missing {prompt}: {}", self.text);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        fn finish_status(&mut self, success: bool) {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                self.read();
                if let Some(status) = self.child.try_wait().unwrap() {
                    self.read();
                    assert_eq!(status.success(), success, "{}", self.text);
                    return;
                }
                assert!(Instant::now() < deadline, "Terminal did not exit: {}", self.text);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        fn finish(&mut self) { self.finish_status(true); }
    }
    impl Drop for Terminal {
        fn drop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); }
    }

    #[test]
    fn bare_menu_guides_project_creation_and_dispatches_project_check() {
        let root = support::TempDirectory::new();
        let mut terminal = Terminal::start(&root.0, &[]);
        terminal.answer("q to cancel): ", "99\n");
        terminal.answer("q to cancel): ", "1\n");
        terminal.answer("Project name or path (q to cancel): ", "missing-parent/hello\n");
        terminal.answer("Project name or path (q to cancel): ", "hello\n");
        assert!(!root.0.join("hello").exists());
        terminal.answer("Project type", "1\n");
        terminal.finish();
        let project = root.0.join("hello");
        for path in ["src/main.tsr", "tests/greeting.test.tsr", "tsrust.config.json"] { assert!(project.join(path).is_file(), "{path}"); }
        let mut terminal = Terminal::start(&project.join("src"), &[]);
        terminal.answer("5) Check the project", "5\n");
        terminal.finish();
        let output = support::run(support::cli().args(["format", "--check"]).current_dir(&project), 10);
        support::success(&output);
        let bytes = std::fs::read(project.join("tsrust.config.json")).unwrap();
        let mut terminal = Terminal::start(&root.0, &["new"]);
        terminal.answer("Project name or path (q to cancel): ", "hello\n");
        terminal.answer("Project name or path (q to cancel): ", "q\n");
        terminal.finish();
        assert_eq!(std::fs::read(project.join("tsrust.config.json")).unwrap(), bytes);
    }

    #[test]
    fn guided_ui_uses_exact_offline_versions_and_waits_for_target_selection() {
        let root = support::TempDirectory::new();
        let mut terminal = Terminal::start(&root.0, &["ui", "add"]);
        terminal.answer("2) button@1.0.0", "2\n");
        assert!(!root.0.join("tsrust.config.json").exists());
        terminal.answer("Choose one or both (for example 1,2): ", "2\n");
        terminal.finish();
        let config: serde_json::Value = serde_json::from_slice(&std::fs::read(root.0.join("tsrust.config.json")).unwrap()).unwrap();
        assert_eq!(config["ui"]["target"], serde_json::json!({"native":false,"web":true,"embedded":false}));
        assert!(root.0.join("src/components/ui/button.tsrx").is_file());
        assert!(!root.0.join("src/components/ui/card.tsrx").exists());
        let lock: serde_json::Value = serde_json::from_slice(&std::fs::read(root.0.join("ui-components.lock.json")).unwrap()).unwrap();
        assert!(lock["installed"].as_array().unwrap().iter().any(|item| item["identity"]["name"]=="button" && item["identity"]["version"]=="1.0.0"));
        let output = support::run(support::cli().args(["check"]).current_dir(&root.0), 20);
        support::success(&output);

        let mut terminal = Terminal::start(&root.0, &["new", "--ui"]);
        terminal.answer("Project name or path (q to cancel): ", "counter\n");
        terminal.answer("Choose one or both (for example 1,2): ", "2\n");
        terminal.finish();
        let output = support::run(support::cli().args(["format", "--check"]).current_dir(root.0.join("counter")), 20);
        support::success(&output);
    }

    #[test]
    fn menu_and_component_cancellation_leave_no_files() {
        for args in [&[][..], &["ui"][..], &["ui", "add"][..], &["new"][..]] {
            let root = support::TempDirectory::new();
            let mut terminal = Terminal::start(&root.0, args);
            terminal.answer("q to cancel", "q\n");
            terminal.finish();
            assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
        }
        let root = support::TempDirectory::new();
        let mut terminal = Terminal::start(&root.0, &[]);
        terminal.answer("q to cancel): ", "\x04");
        terminal.finish();
        assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
        for flag in ["--json", "--yes", "--dry-run"] {
            let root = support::TempDirectory::new();
            let mut terminal = Terminal::start(&root.0, &["ui", "add", "button@1.0.0", flag]);
            terminal.finish_status(false);
            assert!(terminal.text.contains("--target"), "{}", terminal.text);
            assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
        }
    }
}
