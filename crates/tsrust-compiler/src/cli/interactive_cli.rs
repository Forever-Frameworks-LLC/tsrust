//! Terminal presentation collects arguments; ordinary commands retain execution ownership.
use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
static HEADER: AtomicBool = AtomicBool::new(false);
struct DefaultTheme;
impl cliclack::Theme for DefaultTheme {}
struct SelectionTheme;
impl cliclack::Theme for SelectionTheme {
    fn format_header(&self,state:&cliclack::ThemeState,prompt:&str)->String {
        let header=cliclack::Theme::format_header(&DefaultTheme,state,prompt);
        if matches!(state,cliclack::ThemeState::Active|cliclack::ThemeState::Error(_)) {
            format!("{header}{}\n",self.bar_color(state).apply_to("│"))
        }else{header}
    }
    fn format_footer_with_message(&self,state:&cliclack::ThemeState,message:&str)->String {
        let footer=cliclack::Theme::format_footer_with_message(&DefaultTheme,state,message);
        if matches!(state,cliclack::ThemeState::Active|cliclack::ThemeState::Error(_)) {
            format!("{}\n{footer}",self.bar_color(state).apply_to("│"))
        }else{footer}
    }
}
fn visible_rows(height:u16)->usize { usize::from(height).saturating_sub(10).clamp(1,10) }

pub(super) fn available() -> bool { io::stdin().is_terminal() && io::stderr().is_terminal() }
pub(super) fn rich() -> bool { available() && std::env::var("TERM").as_deref() != Ok("dumb") }
pub(super) fn colors() {
    console::set_colors_enabled(super::color_enabled(io::stdout().is_terminal()));
    console::set_colors_enabled_stderr(super::color_enabled(io::stderr().is_terminal()));
}
pub(super) fn launcher(arguments: &[String]) -> Result<bool, String> {
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        let value = if argument == "--color" { arguments.next().map(String::as_str) }
            else if let Some(value) = argument.strip_prefix("--color=") { Some(value) }
            else { return Ok(false); };
        if !matches!(value, Some("auto" | "always" | "never")) { return Err("--color requires auto, always or never".into()); }
    }
    Ok(true)
}
fn header() -> Result<(), String> {
    if !rich() || HEADER.swap(true, Ordering::Relaxed) { return Ok(()); }
    cliclack::set_theme(DefaultTheme);
    let (height, width) = console::Term::stderr().size();
    if width >= 40 && height >= 20 {
        eprintln!();
        for line in ["▀█▀ █▀▀ █▀█ █ █ █▀▀ ▀█▀", " █  ▀▀█ █▀▄ █ █ ▀▀█  █ ", " ▀  ▀▀▀ ▀ ▀ ▀▀▀ ▀▀▀  ▀ "] {
            eprintln!("  {}", console::Style::new().for_stderr().cyan().bold().apply_to(line));
        }
        eprintln!();
    }
    cliclack::intro(format!("Tsrust {}  ·  native applications", env!("CARGO_PKG_VERSION"))).map_err(|e| e.to_string())
}
fn prompt_result<T>(result: io::Result<T>) -> Result<Option<T>, String> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}
// Keep cancellation keys raw between the library's per-key reads and redraws.
struct PromptInput {
    #[cfg(unix)]
    original: libc::termios,
}
impl PromptInput {
    fn enter() -> io::Result<Self> {
        #[cfg(unix)]
        let original = unsafe {
            let mut settings = std::mem::MaybeUninit::uninit();
            if libc::tcgetattr(0, settings.as_mut_ptr()) != 0 { return Err(io::Error::last_os_error()); }
            let original = settings.assume_init();
            let mut raw = original;
            libc::cfmakeraw(&mut raw);
            raw.c_oflag = original.c_oflag;
            if libc::tcsetattr(0, libc::TCSADRAIN, &raw) != 0 { return Err(io::Error::last_os_error()); }
            original
        };
        Ok(Self { #[cfg(unix)] original })
    }
}
impl Drop for PromptInput {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe { libc::tcsetattr(0, libc::TCSADRAIN, &self.original); }
        let _ = console::Term::stderr().show_cursor();
    }
}
fn interact<T>(prompt: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    let _input = PromptInput::enter()?;
    prompt()
}
fn interact_selection<T>(prompt:impl FnOnce()->io::Result<T>)->io::Result<T> {
    cliclack::set_theme(SelectionTheme);
    let result=interact(prompt);
    cliclack::set_theme(DefaultTheme);
    result
}
fn read_line(input: &mut impl BufRead, output: &mut impl Write, label: &str) -> Result<Option<String>, String> {
    write!(output, "{label}: ").and_then(|_| output.flush()).map_err(|e| e.to_string())?;
    let mut line = String::new();
    if input.read_line(&mut line).map_err(|e| e.to_string())? == 0 { return Ok(None); }
    let line = line.trim().to_owned();
    Ok(if matches!(line.as_str(), "q" | "quit") { None } else { Some(line) })
}
pub(super) fn text(label: &str) -> Result<Option<String>, String> {
    if !rich() { return read_line(&mut io::stdin().lock(), &mut io::stderr().lock(), label); }
    header()?;
    prompt_result(interact(|| cliclack::input(label).placeholder("my-app").interact::<String>()))
        .map(|value| value.map(|text| text.trim().to_owned()))
}
fn choose_from(input: &mut impl BufRead, output: &mut impl Write, title: &str, choices: &[(&str, &str)]) -> Result<Option<usize>, String> {
    writeln!(output, "{title}\n").map_err(|e| e.to_string())?;
    for (index, (label, hint)) in choices.iter().enumerate() { writeln!(output, "  {}) {label}  {hint}", index + 1).map_err(|e| e.to_string())?; }
    writeln!(output).map_err(|e|e.to_string())?;
    loop {
        let Some(line) = read_line(input, output, "Choose a number (q to cancel)")? else { return Ok(None); };
        if let Ok(number) = line.parse::<usize>() && number > 0 && number <= choices.len() { return Ok(Some(number - 1)); }
        writeln!(output, "Enter a number from 1 to {}, or q to cancel.", choices.len()).map_err(|e| e.to_string())?;
    }
}
pub(super) fn select(title: &str, choices: &[(&str, &str)], searchable: bool) -> Result<Option<usize>, String> {
    if !rich() { return choose_from(&mut io::stdin().lock(), &mut io::stderr().lock(), title, choices); }
    header()?;
    let rows = visible_rows(console::Term::stderr().size().0);
    let mut prompt = cliclack::select(format!("{title}  ·  ↑↓ move, Enter select, Esc cancel")).max_rows(rows);
    for (index, (label, hint)) in choices.iter().enumerate() { prompt = prompt.item(index, label, hint); }
    if searchable { prompt = prompt.filter_mode(); }
    prompt_result(interact_selection(|| prompt.interact()))
}
pub(super) fn choose(title: &str, choices: &[&str]) -> Result<Option<usize>, String> {
    select(title, &choices.iter().map(|label| (*label, "")).collect::<Vec<_>>(), false)
}
fn multiple_from(input:&mut impl BufRead,output:&mut impl Write,title:&str,choices:&[(&str,&str)])->Result<Option<Vec<usize>>,String> {
    writeln!(output,"{title}\n").map_err(|e|e.to_string())?;
    for (index,(label,hint)) in choices.iter().enumerate(){writeln!(output,"  {}) {label}  {hint}",index+1).map_err(|e|e.to_string())?;}
    writeln!(output).map_err(|e|e.to_string())?;
    loop {
        let Some(line)=read_line(input,output,"Choose numbers separated by commas (q to cancel)")? else{return Ok(None);};
        let selected=line.split(|ch:char|ch==',' || ch.is_whitespace()).filter(|part|!part.is_empty()).map(|part|part.parse::<usize>()).collect::<Result<Vec<_>,_>>();
        if let Ok(mut selected)=selected && !selected.is_empty() && selected.iter().all(|number|*number>0 && *number<=choices.len()) {
            selected.sort_unstable();selected.dedup();return Ok(Some(selected.into_iter().map(|number|number-1).collect()));
        }
        writeln!(output,"Choose at least one number from 1 to {}, or q to cancel.",choices.len()).map_err(|e|e.to_string())?;
    }
}
pub(super) fn multiselect(title:&str,choices:&[(&str,&str)])->Result<Option<Vec<usize>>,String> {
    if !rich(){return multiple_from(&mut io::stdin().lock(),&mut io::stderr().lock(),title,choices);}
    header()?;
    let mut prompt=cliclack::multiselect(format!("{title}  ·  ↑↓ move, Space toggle, Enter install, Esc cancel")).max_rows(visible_rows(console::Term::stderr().size().0)).required(true);
    for (index,(label,hint)) in choices.iter().enumerate(){prompt=prompt.item(index,label,hint);}
    prompt_result(interact_selection(||prompt.interact()))
}
pub(super) fn targets() -> Result<Vec<String>, String> {
    header()?;
    interact_selection(|| cliclack::multiselect("UI targets  ·  ↑↓ move, Space toggle, Enter confirm")
        .max_rows(visible_rows(console::Term::stderr().size().0))
        .item("native", "Native desktop", "GPU rendering with native windows")
        .item("web", "Web", "Browser output through the bounded Wasm builder")
        .required(true).interact()).map(|values| values.into_iter().map(str::to_owned).collect())
        .map_err(|error| if error.kind() == io::ErrorKind::Interrupted {"UI target selection cancelled; no files created".into()}else{error.to_string()})
}
pub(super) fn note(title: &str, text: &str) -> Result<(), String> {
    if rich() { cliclack::note(title, text).map_err(|e| e.to_string()) }
    else { eprintln!("{title}\n{text}"); Ok(()) }
}
fn action(title: &str, choices: &[(&str, &str, &[&str])]) -> Result<Option<Vec<String>>, String> {
    let labels = choices.iter().map(|(label, hint, _)| (*label, *hint)).chain([("Back", "Return to categories")]).collect::<Vec<_>>();
    Ok(select(title, &labels, false)?.map(|index| choices.get(index).map_or_else(Vec::new, |(_, _, args)| args.iter().map(|arg| (*arg).to_owned()).collect())))
}
pub(super) fn command() -> Result<Option<Vec<String>>, String> {
    let project = super::project_config::discover()?;
    header()?;
    if let Some(project) = &project {
        let path = project.root.display().to_string().chars().flat_map(|ch| if ch.is_control() {ch.escape_default().collect::<Vec<_>>()}else{vec![ch]}).collect::<String>();
        note("Workspace", &path)?;
    }
    loop {
        let mut categories = Vec::new();
        if project.is_some() { categories.extend([("Development", "Watch, run and build this application"), ("Quality", "Type checks, tests and formatting")]); }
        categories.extend([("UI", "Browse offline components and add editable source"), ("Project", "Create a console or UI application"), ("Help", "Command reference and examples"), ("Exit", "Return to your shell")]);
        let Some(category) = select("What would you like to do?", &categories, false)? else { return Ok(None); };
        let selection = match categories[category].0 {
            "Development" => action("Development", &[("Start development", "tsrust dev", &["dev"]), ("Run once", "tsrust run", &["run"]), ("Build executable", "tsrust build", &["build"])])?,
            "Quality" => action("Quality", &[("Check types", "tsrust check", &["check"]), ("Run tests", "tsrust test", &["test"]), ("Check formatting", "tsrust format --check", &["format", "--check"])])?,
            "Project" => action("Project", &[("Create a project", "Choose a destination, application type and UI targets", &["new"])])?,
            "UI" => loop {
                let Some(choice) = select("UI", &[("Browse components", "Search current offline components and inspect details"), ("Add components", "Select several and install editable source together"), ("Consolidate license notices", "Move full component notices into attributions/"), ("Back", "Return to categories")], false)? else { return Ok(None); };
                if choice == 3 { break Some(Vec::new()); }
                if choice == 1 { break Some(vec!["ui".into(), "add".into()]); }
                if choice == 2 { break Some(vec!["ui".into(), "attributions".into()]); }
                let Some(args) = super::ui_cli::browse()? else { return Ok(None); };
                if !args.is_empty() { break Some(std::iter::once("ui".into()).chain(args).collect()); }
            },
            "Help" => Some(vec!["--help".into()]),
            _ => { if rich() { cliclack::outro("See you next time.").map_err(|e| e.to_string())?; } return Ok(None); },
        };
        let Some(args) = selection else { return Ok(None); };
        if !args.is_empty() {
            if rich() { cliclack::outro(format!("tsrust {}", args.join(" "))).map_err(|e| e.to_string())?; }
            return Ok(Some(args));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selectors_pad_list_boundaries_and_keep_item_rows_compact() {
        use cliclack::{Theme,ThemeState};let theme=SelectionTheme;
        for state in [ThemeState::Active,ThemeState::Error("Choose an item".into())] {
            let header=theme.format_header(&state,"Choose");let footer=theme.format_footer_with_message(&state,"scroll");
            assert_eq!(header.lines().count(),2);assert!(header.lines().last().unwrap().contains('│'));
            assert_eq!(footer.lines().count(),2);assert!(footer.lines().next().unwrap().contains('│'));assert!(footer.contains("scroll")||matches!(state,ThemeState::Error(_)));
            for output in [theme.format_select_item(&state,true,"Choice","Hint"),theme.format_multiselect_item(&state,true,true,"Choice","Hint")] {
                assert_eq!(output.lines().count(),1);assert!(output.contains("Choice"));
            }
        }
        for state in [ThemeState::Submit,ThemeState::Cancel] {
            assert_eq!(theme.format_header(&state,"Choose"),DefaultTheme.format_header(&state,"Choose"));
            assert_eq!(theme.format_footer_with_message(&state,""),DefaultTheme.format_footer_with_message(&state,""));
        }
        assert_eq!(theme.format_select_item(&ThemeState::Submit,false,"Choice",""),"");
        for (height,rows) in [(8,1),(12,2),(20,10),(32,10)]{assert_eq!(visible_rows(height),rows);}
        for multiple in [false,true] {
            let mut output=Vec::new();let choices=[("one","first"),("two","second")];
            if multiple {multiple_from(&mut "1\n".as_bytes(),&mut output,"Choose",&choices).unwrap();}
            else {choose_from(&mut "1\n".as_bytes(),&mut output,"Choose",&choices).unwrap();}
            assert!(String::from_utf8(output).unwrap().starts_with("Choose\n\n  1) one  first\n  2) two  second\n\nChoose"));
        }
    }
    #[test]
    fn plain_multiselect_retries_deduplicates_and_cancels_without_defaults() {
        let mut output=Vec::new();let choices=[("one","first"),("two","second")];
        assert_eq!(multiple_from(&mut "\n0\n1,3\n1,bad\n2,1,2\n".as_bytes(),&mut output,"Components",&choices).unwrap(),Some(vec![0,1]));
        for input in ["q\n",""]{assert_eq!(multiple_from(&mut input.as_bytes(),&mut Vec::new(),"Components",&choices).unwrap(),None);}
    }
    #[test]
    fn plain_terminal_retries_and_cancels_without_defaults() {
        for (input, expected) in [("wrong\n0\n3\n2\n", Some(1)), ("q\n", None), ("", None)] {
            let mut output = Vec::new();
            assert_eq!(choose_from(&mut input.as_bytes(), &mut output, "Action", &[("one", "first"), ("two", "second")]).unwrap(), expected);
            assert!(String::from_utf8(output).unwrap().contains("q to cancel"));
        }
    }
    #[test]
    fn launcher_color_flags_and_cancellation_are_explicit() {
        for args in [vec![], vec!["--color", "never"], vec!["--color=always"]] { assert!(launcher(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).unwrap()); }
        assert!(!launcher(&["check".into(), "--color=never".into()]).unwrap());
        assert!(launcher(&["--color=wrong".into()]).is_err());
        assert_eq!(prompt_result::<usize>(Err(io::ErrorKind::Interrupted.into())).unwrap(), None);
        assert!(prompt_result::<usize>(Err(io::ErrorKind::BrokenPipe.into())).is_err());
    }
}
