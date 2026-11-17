use std::io::{self, IsTerminal, Write};

use clap::Parser;
use rustyline::{DefaultEditor, error::ReadlineError};

#[allow(unused_imports)] // Available for the binary as tool support is added.
use rh::tool::Tool;

#[derive(Parser)]
#[command(about = "An experimental AI harness with a terminal chat interface")]
struct Args {
    /// Print diagnostic messages to stderr
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> rustyline::Result<()> {
    let args = Args::parse();
    if args.verbose {
        eprintln!("Starting chat; press Ctrl-C or Ctrl-D to exit.");
    }

    let prompt = if io::stdin().is_terminal() { "> " } else { "" };
    let mut editor = DefaultEditor::new()?;
    let mut output = io::stdout();

    loop {
        let line = match editor.readline(prompt) {
            Ok(line) => line,
            Err(ReadlineError::Eof | ReadlineError::Interrupted) => break,
            Err(error) => return Err(error),
        };
        editor.add_history_entry(line.as_str())?;

        let words = line
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(output, "got: {words}")?;
        output.flush()?;
    }

    Ok(())
}
