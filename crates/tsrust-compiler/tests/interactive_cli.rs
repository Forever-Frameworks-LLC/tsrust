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

    struct Terminal { child: Child, master: File, observer: File, original: libc::tcflag_t, text: String, consumed: usize }
    impl Terminal {
        fn start(root: &std::path::Path, args: &[&str]) -> Self {
            Self::configured(root,args,&[],100,32)
        }
        fn configured(root:&std::path::Path,args:&[&str],env:&[(&str,&str)],width:u16,height:u16)->Self {
            let (mut master, mut slave) = (-1, -1);
            let mut size=libc::winsize{ws_row:height,ws_col:width,ws_xpixel:0,ws_ypixel:0};
            assert_eq!(unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), &mut size) }, 0);
            let master = unsafe { File::from_raw_fd(master) };
            let slave = unsafe { File::from_raw_fd(slave) };
            use std::os::fd::AsRawFd;
            let flags = unsafe { libc::fcntl(master.as_raw_fd(), libc::F_GETFL) };
            assert!(flags >= 0);
            assert_eq!(unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) }, 0);
            let observer=slave.try_clone().unwrap();
            let mut settings=std::mem::MaybeUninit::uninit();
            assert_eq!(unsafe{libc::tcgetattr(slave.as_raw_fd(),settings.as_mut_ptr())},0);
            let original=unsafe{settings.assume_init()}.c_lflag;
            let child = support::cli().args(args).current_dir(root).env("TERM","xterm-256color").env_remove("NO_COLOR").envs(env.iter().copied())
                .stdin(Stdio::from(slave.try_clone().unwrap()))
                .stdout(Stdio::from(slave.try_clone().unwrap()))
                .stderr(Stdio::from(slave)).spawn().unwrap();
            Self { child, master, observer, original, text: String::new(), consumed: 0 }
        }
        fn read(&mut self) {
            let mut buffer = [0; 8192];
            loop { match self.master.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => self.text.push_str(&String::from_utf8_lossy(&buffer[..count])),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock || error.raw_os_error() == Some(libc::EIO) => break,
                Err(error) => panic!("{error}"),
            } }
        }
        fn answer(&mut self, prompt: &str, keys: &[&str]) {
            let deadline = Instant::now() + Duration::from_secs(20);
            loop {
                self.read();
                if let Some(offset) = self.text[self.consumed..].find(prompt) {
                    self.consumed += offset + prompt.len();
                    for key in keys {
                        self.master.write_all(key.as_bytes()).unwrap();
                        std::thread::sleep(Duration::from_millis(40));
                    }
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
                    use std::os::fd::AsRawFd;
                    let mut settings=std::mem::MaybeUninit::uninit();
                    assert_eq!(unsafe{libc::tcgetattr(self.observer.as_raw_fd(),settings.as_mut_ptr())},0);
                    // PENDIN is kernel-owned queued-input state after canonical mode resumes.
                    assert_eq!(unsafe{settings.assume_init()}.c_lflag & !libc::PENDIN,self.original & !libc::PENDIN,"terminal flags not restored");
                    if let Some(hidden)=self.text.rfind("\x1b[?25l") {assert!(self.text.rfind("\x1b[?25h").is_some_and(|shown|shown>hidden),"cursor not restored");}
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

    const DOWN: &str = "\x1b[B";
    const ENTER: &str = "\r";
    const CANCEL: &str = "\x03";

    #[test]
    fn keyboard_categories_create_and_check_a_project() {
        let root = support::TempDirectory::new();
        let mut terminal = Terminal::start(&root.0, &[]);
        terminal.answer("What would you like to do?", &[DOWN, ENTER]);
        terminal.answer("Create a project", &[ENTER]);
        terminal.answer("Project name or path", &["missing/hello", ENTER]);
        terminal.answer("Parent directory does not exist", &["hello", ENTER]);
        assert!(!root.0.join("hello").exists());
        terminal.answer("Project type", &[ENTER]);
        terminal.finish();
        assert!(terminal.text.contains("native applications"));
        assert!(terminal.text.contains("▀█▀"));
        assert!(has_color(&terminal.text));
        let project = root.0.join("hello");
        for path in ["src/main.tsr", "tests/greeting.test.tsr", "tsrust.config.json"] { assert!(project.join(path).is_file(), "{path}"); }
        let mut terminal = Terminal::start(&project.join("src"), &[]);
        terminal.answer("What would you like to do?", &[DOWN, ENTER]);
        terminal.answer("Check types", &[ENTER]);
        terminal.finish();
        let output = support::run(support::cli().args(["format", "--check"]).current_dir(&project), 10);
        support::success(&output);
        let bytes = std::fs::read(project.join("tsrust.config.json")).unwrap();
        let mut terminal = Terminal::start(&root.0, &["new"]);
        terminal.answer("Project name or path", &["hello", ENTER]);
        terminal.answer("Existing projects are never overwritten", &[]);
        terminal.answer("Project name or path", &["\x1b"]);
        terminal.finish();
        assert_eq!(std::fs::read(project.join("tsrust.config.json")).unwrap(), bytes);
    }

    #[test]
    fn searchable_catalog_has_details_and_returns_to_categories_without_writes() {
        let root = support::TempDirectory::new();
        let mut terminal = Terminal::start(&root.0, &[]);
        terminal.answer("What would you like to do?", &[ENTER]);
        terminal.answer("Browse components", &[ENTER]);
        terminal.answer("Component catalog", &["button", ENTER]);
        terminal.answer("Component actions", &[DOWN, ENTER]);
        assert!(terminal.text.contains("Dependencies: tokens@1.0.0"));
        assert!(terminal.text.contains("Install component"));
        terminal.answer("Component catalog", &["Back to UI", ENTER]);
        terminal.answer("Browse components", &[DOWN, DOWN, DOWN, ENTER]);
        terminal.answer("What would you like to do?", &[CANCEL]);
        terminal.finish();
        assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
    }

    #[test]
    fn guided_ui_multiselect_installs_current_components_in_one_transaction() {
        let root = support::TempDirectory::new();
        let mut terminal = Terminal::start(&root.0, &["ui", "add"]);
        terminal.answer("Add components", &[ENTER]);
        terminal.answer("Input required", &[" ", DOWN, " ", ENTER]);
        assert!(!root.0.join("tsrust.config.json").exists());
        terminal.answer("UI targets", &[DOWN, " ", ENTER]);
        terminal.finish();
        let config: serde_json::Value = serde_json::from_slice(&std::fs::read(root.0.join("tsrust.config.json")).unwrap()).unwrap();
        assert_eq!(config["ui"]["target"], serde_json::json!({"native":false,"web":true,"embedded":false}));
        assert!(root.0.join("src/components/ui/button.tsrx").is_file());
        assert!(root.0.join("src/components/ui/card.tsrx").is_file());
        let lock: serde_json::Value = serde_json::from_slice(&std::fs::read(root.0.join("ui-components.lock.json")).unwrap()).unwrap();
        assert!(lock["installed"].as_array().unwrap().iter().any(|item| item["identity"]["name"]=="button" && item["identity"]["version"]=="2.0.0"));
        let output = support::run(support::cli().args(["check"]).current_dir(&root.0), 20);
        support::success(&output);
        let mut terminal = Terminal::start(&root.0, &["new", "--ui"]);
        terminal.answer("Project name or path", &["counter", ENTER]);
        terminal.answer("UI targets", &[" ", DOWN, " ", ENTER]);
        terminal.finish();
        let config: serde_json::Value = serde_json::from_slice(&std::fs::read(root.0.join("counter/tsrust.config.json")).unwrap()).unwrap();
        assert_eq!(config["ui"]["target"], serde_json::json!({"native":true,"web":true,"embedded":false}));
        let output = support::run(support::cli().args(["format", "--check"]).current_dir(root.0.join("counter")), 20);
        support::success(&output);
    }

    fn has_color(text: &str) -> bool {
        text.split("\x1b[").skip(1).any(|tail| {
            let count=tail.bytes().take_while(|byte|byte.is_ascii_digit() || *byte==b';').count();
            tail.as_bytes().get(count)==Some(&b'm')
        })
    }

    #[test]
    fn cancellation_color_policy_and_narrow_terminals_restore_terminal_state() {
        for (args,env,width,height,expected_color) in [
            (vec![],vec![],100,32,true),
            (vec!["--color","never"],vec![],100,32,false),
            (vec![],vec![("NO_COLOR","1")],100,32,false),
            (vec!["--color=always"],vec![("NO_COLOR","1")],100,32,true),
            (vec![],vec![],40,12,true),
        ] {
            let root=support::TempDirectory::new();
            let mut terminal=Terminal::configured(&root.0,&args,&env,width,height);
            terminal.answer("What would you like to do?", &[CANCEL]);terminal.finish();
            assert_eq!(has_color(&terminal.text),expected_color,"{}",terminal.text);
            if height<20 {assert!(!terminal.text.contains("▀█▀"));}
            assert_eq!(std::fs::read_dir(&root.0).unwrap().count(),0);
        }
        for args in [&["ui"][..], &["ui","add"][..], &["new"][..]] {
            let root=support::TempDirectory::new();let mut terminal=Terminal::start(&root.0,args);
            let prompt=if args==["new"]{"Project name or path"}else if args==["ui"]{"UI components"}else{"Add components"};
            terminal.answer(prompt,&[CANCEL]);terminal.finish();assert_eq!(std::fs::read_dir(&root.0).unwrap().count(),0);
        }
        for flag in ["--json","--yes","--dry-run"] {
            let root=support::TempDirectory::new();let mut terminal=Terminal::start(&root.0,&["ui","add","button@2.0.0",flag]);
            terminal.finish_status(false);assert!(terminal.text.contains("--target"));assert_eq!(std::fs::read_dir(&root.0).unwrap().count(),0);
        }
        let root=support::TempDirectory::new();let mut terminal=Terminal::configured(&root.0,&[],&[("TERM","dumb")],80,24);
        terminal.answer("q to cancel",&["q\n"]);terminal.finish();assert!(!terminal.text.contains('\x1b'));
    }

    #[test]
    fn catalog_output_is_styled_only_for_people_and_json_remains_clean() {
        let root=support::TempDirectory::new();let mut terminal=Terminal::start(&root.0,&["ui","list","--bundled"]);terminal.finish();
        assert!(terminal.text.contains("COMPONENTS")&&terminal.text.contains("FOUNDATIONS")&&has_color(&terminal.text));
        let output=support::run(support::cli().args(["ui","list","--bundled","--json"]).current_dir(&root.0),10);
        support::success(&output);let items:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();assert_eq!(items.as_array().unwrap().len(),8);assert!(!output.stdout.contains(&27));
        let identities=items.as_array().unwrap().iter().map(|item|(item["name"].as_str().unwrap(),item["variant"].as_str().unwrap())).collect::<std::collections::BTreeSet<_>>();assert_eq!(identities.len(),8);
        assert!(items.as_array().unwrap().iter().any(|item|item["name"]=="card" && item["version"]=="3.0.0"));
        let output=support::run(support::cli().args(["ui","list","--bundled"]).current_dir(&root.0),10);
        support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains("button@2.0.0 --variant default (MIT)"));assert!(!output.stdout.contains(&27));
    }

    #[test]
    fn batch_selection_cancels_without_writes_and_scrolls_in_short_terminals() {
        for (width,height) in [(100,32),(40,12)] {
            let root=support::TempDirectory::new();let mut terminal=Terminal::configured(&root.0,&["ui","add"],&[],width,height);
            terminal.answer("Add components",&[" ",DOWN," ",CANCEL]);terminal.finish();assert_eq!(std::fs::read_dir(&root.0).unwrap().count(),0);
        }
        let root=support::TempDirectory::new();let mut terminal=Terminal::configured(&root.0,&["ui","add"],&[("TERM","dumb")],80,24);
        terminal.answer("q to cancel",&["1,2\n"]);terminal.answer("UI targets",&["1\n"]);terminal.finish();
        for name in ["button","card"]{assert!(root.0.join(format!("src/components/ui/{name}.tsrx")).is_file());}
        assert!(!terminal.text.contains('\x1b'));
    }
}
