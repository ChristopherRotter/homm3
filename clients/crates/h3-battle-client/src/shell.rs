//! `h3-view shell` (E27 M2 T3a; CLT-Q103, decision 21: the player's program until E24 M5 hosts the
//! shell): the game shell's frame driven headless, a word a line (`h3_view::shell::text`): the
//! screen on is printed with the words it takes, each word read ends it and prints the next. The
//! words come from standard input, or from a script file (`--script`), each echoed after `> `; a
//! word a script's screen does not take ends the run with exit code 1. `--no-intro` starts at the
//! main menu (the setting `Show Intro` off). The window follows (E27 M2 T4).

use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::ExitCode;

use h3_view::shell::{Screen, Shell, text};

/// The command's options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    script: Option<PathBuf>,
    intro: bool,
}

impl Options {
    /// Parses `[--script <file>] [--no-intro]`.
    ///
    /// # Errors
    /// An unknown argument, or `--script` without its file.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            script: None,
            intro: true,
        };
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            match arg.as_str() {
                "--script" => {
                    let file = rest.next().ok_or("shell: `--script` needs a file")?;
                    options.script = Some(PathBuf::from(file));
                }
                "--no-intro" => options.intro = false,
                other => return Err(format!("shell: unknown argument `{other}`")),
            }
        }
        Ok(options)
    }
}

/// Runs the shell until its end or the input's.
#[must_use]
pub fn run(options: &Options) -> ExitCode {
    let mut shell = Shell::new(options.intro);
    println!("{}", text::describe(shell.screen()));
    let lines: Vec<String> = match &options.script {
        Some(path) => match fs::read_to_string(path) {
            Ok(script) => script.lines().map(String::from).collect(),
            Err(e) => {
                eprintln!("h3-view shell: {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        },
        None => {
            for line in io::stdin().lock().lines() {
                let Ok(line) = line else { break };
                println!("{}", text::answer(&mut shell, &line));
                if shell.screen() == Screen::End {
                    break;
                }
            }
            return ExitCode::SUCCESS;
        }
    };
    for line in lines {
        let refused = !line.trim().is_empty() && text::ending(shell.screen(), &line).is_none();
        println!("> {}", line.trim());
        println!("{}", text::answer(&mut shell, &line));
        if refused {
            return ExitCode::FAILURE;
        }
        if shell.screen() == Screen::End {
            break;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::Options;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(&args.iter().map(|a| String::from(*a)).collect::<Vec<_>>())
    }

    #[test]
    fn the_options_are_a_script_and_the_intro_off() {
        let none = parse(&[]).unwrap();
        assert_eq!((none.script, none.intro), (None, true));
        let both = parse(&["--no-intro", "--script", "menus.txt"]).unwrap();
        assert_eq!(both.script.unwrap().to_str(), Some("menus.txt"));
        assert!(!both.intro);
        assert!(parse(&["--script"]).is_err());
        assert!(parse(&["--intro"]).is_err());
    }
}
