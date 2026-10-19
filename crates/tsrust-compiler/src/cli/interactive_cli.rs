//! Terminal prompts only collect arguments; ordinary commands keep execution ownership.
use std::io::{self, BufRead, IsTerminal, Write};

pub(super) fn available() -> bool {
    io::stdin().is_terminal() && io::stderr().is_terminal()
}

fn read_line(input: &mut impl BufRead, output: &mut impl Write, label: &str) -> Result<Option<String>, String> {
    write!(output, "{label}: ").and_then(|_| output.flush()).map_err(|e| e.to_string())?;
    let mut line = String::new();
    if input.read_line(&mut line).map_err(|e| e.to_string())? == 0 { return Ok(None); }
    let line = line.trim().to_owned();
    Ok(if matches!(line.as_str(), "q" | "quit") { None } else { Some(line) })
}

pub(super) fn text(label: &str) -> Result<Option<String>, String> {
    read_line(&mut io::stdin().lock(), &mut io::stderr().lock(), label)
}

fn choose_from(input: &mut impl BufRead, output: &mut impl Write, title: &str, choices: &[&str]) -> Result<Option<usize>, String> {
    writeln!(output, "{title}").map_err(|e| e.to_string())?;
    for (index, choice) in choices.iter().enumerate() {
        writeln!(output, "  {}) {choice}", index + 1).map_err(|e| e.to_string())?;
    }
    loop {
        let Some(line) = read_line(input, output, "Choose a number (q to cancel)")? else { return Ok(None); };
        if let Ok(number) = line.parse::<usize>() && number > 0 && number <= choices.len() {
            return Ok(Some(number - 1));
        }
        writeln!(output, "Enter a number from 1 to {}, or q to cancel.", choices.len()).map_err(|e| e.to_string())?;
    }
}

pub(super) fn choose(title: &str, choices: &[&str]) -> Result<Option<usize>, String> {
    choose_from(&mut io::stdin().lock(), &mut io::stderr().lock(), title, choices)
}

pub(super) fn command() -> Result<Option<Vec<String>>, String> {
    let project = super::project_config::discover()?;
    let mut choices = vec![("Create a project", vec!["new"])];
    if let Some(project) = &project {
        eprintln!("Project: {}", project.root.display());
        choices.extend([
            ("Start development", vec!["dev"]),
            ("Run the project", vec!["run"]),
            ("Build the project", vec!["build"]),
            ("Check the project", vec!["check"]),
            ("Run tests", vec!["test"]),
            ("Check formatting", vec!["format", "--check"]),
        ]);
    }
    choices.extend([
        ("Add a bundled UI component", vec!["ui", "add"]),
        ("List bundled UI components", vec!["ui", "list", "--bundled"]),
        ("Show help", vec!["--help"]),
        ("Exit", vec![]),
    ]);
    let labels = choices.iter().map(|(label, _)| *label).collect::<Vec<_>>();
    Ok(choose("Tsrust", &labels)?.and_then(|index| {
        let args = &choices[index].1;
        (!args.is_empty()).then(|| args.iter().map(|arg| (*arg).to_owned()).collect())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_retries_invalid_input_and_cancels_without_defaults() {
        for (input, expected) in [("wrong\n0\n3\n2\n", Some(1)), ("q\n", None), ("", None)] {
            let mut output = Vec::new();
            assert_eq!(choose_from(&mut input.as_bytes(), &mut output, "Action", &["one", "two"]).unwrap(), expected);
            assert!(String::from_utf8(output).unwrap().contains("q to cancel"));
        }
    }
}
