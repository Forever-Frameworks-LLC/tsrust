use zed_extension_api::{self as zed, Command, LanguageServerId, Worktree};

struct Tsrust;

impl zed::Extension for Tsrust {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _: &LanguageServerId,
        worktree: &Worktree,
    ) -> zed::Result<Command> {
        let command = worktree.which("tsrust-lsp").ok_or(
            "Install tsrust-lsp on PATH, or configure lsp.tsrust-lsp.binary.path in Zed settings.",
        )?;
        Ok(Command {
            command,
            args: Vec::new(),
            env: Vec::new(),
        })
    }
}

zed::register_extension!(Tsrust);
