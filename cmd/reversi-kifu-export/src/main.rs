use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(RunOutcome::Success) => ExitCode::SUCCESS,
        Ok(RunOutcome::Help(message)) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("reversi-kifu-export: {err}");
            ExitCode::from(2)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RunOutcome {
    Success,
    Help(String),
}

fn run<I>(args: I) -> Result<RunOutcome, String>
where
    I: IntoIterator<Item = String>,
{
    let config = parse_args(args)?;
    if config.help_requested {
        return Ok(RunOutcome::Help(usage()));
    }
    let Some(input_dir) = config.input_dir else {
        return Err(usage());
    };

    let transcript = reversi_game::load_artifact_transcript(&input_dir)?;
    let rendered = if config.include_pass {
        transcript.render_lossless()?
    } else {
        transcript.render_compact()?
    };
    println!("{rendered}");
    Ok(RunOutcome::Success)
}

#[derive(Debug, PartialEq, Eq)]
struct CliConfig {
    help_requested: bool,
    include_pass: bool,
    input_dir: Option<PathBuf>,
}

fn parse_args<I>(args: I) -> Result<CliConfig, String>
where
    I: IntoIterator<Item = String>,
{
    let mut help_requested = false;
    let mut include_pass = false;
    let mut input_dir: Option<PathBuf> = None;

    for arg in args {
        match arg.as_str() {
            "--include-pass" => include_pass = true,
            "--help" | "-h" => help_requested = true,
            _ if arg.starts_with('-') => {
                return Err(format!("unsupported option {arg}\n\n{}", usage()));
            }
            _ => {
                if input_dir.is_some() {
                    return Err(format!("expected one artifact directory\n\n{}", usage()));
                }
                input_dir = Some(PathBuf::from(arg));
            }
        }
    }

    Ok(CliConfig {
        help_requested,
        include_pass,
        input_dir,
    })
}

fn usage() -> String {
    "usage: reversi-kifu-export [--include-pass] <artifact-dir>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_flag_returns_help_outcome() {
        let result = run(["--help".to_string()]).expect("help outcome");
        assert_eq!(result, RunOutcome::Help(usage()));
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let err = parse_args(["--wat".to_string()]).expect_err("unknown flag should fail");
        assert!(err.contains("unsupported option --wat"));
    }
}
