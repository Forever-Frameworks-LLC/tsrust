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

#[test]
fn ui_commands_list_add_wrappers_and_create_desktop_projects() {
    let root = support::TempDirectory::new();
    let help = support::run(support::cli().arg("--help").current_dir(&root.0), 10);
    support::success(&help);
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("tsrust ui list") && text.contains("--ui") && !text.contains("--target native|web"), "{text}");
    for (args, message) in [(vec!["ui"], "Usage: tsrust ui"), (vec!["ui", "add", "nonexistent"], "Unknown component"), (vec!["dev", "--target", "web"], "Unknown or repeated dev option")] {
        let output = support::run(support::cli().args(&args).current_dir(&root.0).stdin(std::process::Stdio::null()), 10);
        assert!(!output.status.success(), "{args:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(message), "{args:?}: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(std::fs::read_dir(&root.0).unwrap().count(), 0);
    }
    let list = support::run(support::cli().args(["ui", "list"]).current_dir(&root.0), 10);
    support::success(&list);
    let listed = String::from_utf8(list.stdout).unwrap();
    assert!(listed.lines().any(|line| line.starts_with("Settings ")) && listed.contains("DataTable") && !listed.lines().any(|line| line.starts_with("SettingItem ")), "{listed}");
    support::success(&support::run(support::cli().args(["new", "counter", "--ui"]).current_dir(&root.0).stdin(std::process::Stdio::null()), 20));
    let project = root.0.join("counter");
    for file in ["tsrust.config.json", "src/main.tsrx", "src/locales/en.json", "icon.png", "README.md"] {assert!(project.join(file).is_file(), "{file}");}
    support::success(&support::run(support::cli().args(["ui", "add", "settings"]).current_dir(&project).stdin(std::process::Stdio::null()), 20));
    let wrapper = std::fs::read_to_string(project.join("src/components/Settings.tsrx")).unwrap();
    assert!(wrapper.contains("export function SettingItem(props: SettingItemProps): Node"), "{wrapper}");
    support::success(&support::run(support::cli().args(["check", "src/components"]).current_dir(&project), 60));
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
        terminal.answer("What would you like to do?", &[ENTER]);
        terminal.answer("Create a project", &[ENTER]);
        terminal.answer("Project name or path", &["missing/hello", ENTER]);
        terminal.answer("Parent directory does not exist", &["hello", ENTER]);
        assert!(!root.0.join("hello").exists());
        terminal.answer("Project type", &[DOWN, ENTER]);
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



    fn has_color(text: &str) -> bool {
        text.split("\x1b[").skip(1).any(|tail| {
            let count=tail.bytes().take_while(|byte|byte.is_ascii_digit() || *byte==b';').count();
            tail.as_bytes().get(count)==Some(&b'm')
        })
    }



}
