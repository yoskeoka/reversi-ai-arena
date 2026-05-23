use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("reversi-kifu-export: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut include_pass = false;
    let mut input_dir: Option<PathBuf> = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--include-pass" => include_pass = true,
            "--help" | "-h" => return Err(usage()),
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

    let input_dir = input_dir.ok_or_else(usage)?;
    let transcript = reversi_game::load_artifact_transcript(&input_dir)?;
    let rendered = if include_pass {
        transcript.render_lossless()?
    } else {
        transcript.render_compact()?
    };
    println!("{rendered}");
    Ok(())
}

fn usage() -> String {
    "usage: reversi-kifu-export [--include-pass] <artifact-dir>".to_string()
}
