use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{Action, ActionKind, Position};

const RECORD_JSON: &str = "record.json";
const HISTORY_JSON: &str = "history.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptEntry {
    Move(Position),
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcript {
    entries: Vec<TranscriptEntry>,
}

impl Transcript {
    pub fn entries(&self) -> &[TranscriptEntry] {
        &self.entries
    }

    pub fn render_compact(&self) -> Result<String, String> {
        render_entries(&self.entries, false)
    }

    pub fn render_lossless(&self) -> Result<String, String> {
        render_entries(&self.entries, true)
    }
}

pub fn load_artifact_transcript(input_dir: &Path) -> Result<Transcript, String> {
    let source = resolve_artifact_source(input_dir)?;
    let events = match source.kind {
        ArtifactSourceKind::Record => {
            let artifact: RecordArtifact = read_json(&source.path)?;
            artifact.event_log
        }
        ArtifactSourceKind::History => read_json(&source.path)?,
    };
    transcript_from_events(&events)
}

#[derive(Debug, Clone)]
struct ArtifactSource {
    path: PathBuf,
    kind: ArtifactSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtifactSourceKind {
    Record,
    History,
}

fn resolve_artifact_source(input_dir: &Path) -> Result<ArtifactSource, String> {
    if !input_dir.exists() {
        return Err(format!(
            "artifact directory does not exist: {}",
            input_dir.display()
        ));
    }
    if !input_dir.is_dir() {
        return Err(format!(
            "artifact input must be a directory: {}",
            input_dir.display()
        ));
    }
    if let Some(source) = select_source_in_dir(input_dir) {
        return Ok(source);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(input_dir)
        .map_err(|err| format!("read artifact directory {}: {err}", input_dir.display()))?
    {
        let entry = entry.map_err(|err| {
            format!(
                "read artifact directory entry under {}: {err}",
                input_dir.display()
            )
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(source) = select_source_in_dir(&path) {
            candidates.push(source);
        }
    }
    candidates.sort_by(|left, right| left.path.cmp(&right.path));

    match candidates.len() {
        0 => Err(format!(
            "no record.json or history.json found under {}",
            input_dir.display()
        )),
        1 => Ok(candidates.remove(0)),
        _ => Err(format!(
            "multiple match artifact directories found under {}; pass <output-dir>/<match-id> explicitly",
            input_dir.display()
        )),
    }
}

fn select_source_in_dir(dir: &Path) -> Option<ArtifactSource> {
    let record = dir.join(RECORD_JSON);
    if record.is_file() {
        return Some(ArtifactSource {
            path: record,
            kind: ArtifactSourceKind::Record,
        });
    }
    let history = dir.join(HISTORY_JSON);
    if history.is_file() {
        return Some(ArtifactSource {
            path: history,
            kind: ArtifactSourceKind::History,
        });
    }
    None
}

fn transcript_from_events(events: &[ArtifactEvent]) -> Result<Transcript, String> {
    let mut entries = Vec::new();
    for event in events {
        if event.kind != "turn_result" {
            continue;
        }
        let Some(payload) = event.payload.as_ref() else {
            return Err(format!(
                "turn_result at seq {} is missing payload",
                event.seq
            ));
        };
        if payload.action_status.as_deref() != Some("accepted") {
            continue;
        }
        let action = payload.action.clone().ok_or_else(|| {
            format!(
                "accepted turn_result at seq {} is missing action",
                event.seq
            )
        })?;
        match action.kind {
            ActionKind::Place => {
                let position = action.position.ok_or_else(|| {
                    format!(
                        "accepted placement at seq {} is missing position",
                        event.seq
                    )
                })?;
                entries.push(TranscriptEntry::Move(position));
            }
            ActionKind::Pass => {
                if action.position.is_some() {
                    return Err(format!(
                        "pass action at seq {} must not carry position",
                        event.seq
                    ));
                }
                entries.push(TranscriptEntry::Pass);
            }
        }
    }
    Ok(Transcript { entries })
}

fn render_entries(entries: &[TranscriptEntry], include_pass: bool) -> Result<String, String> {
    let mut rendered = String::new();
    for entry in entries {
        match entry {
            TranscriptEntry::Move(position) => rendered.push_str(&format_position(*position)?),
            TranscriptEntry::Pass if include_pass => rendered.push_str("pass"),
            TranscriptEntry::Pass => {}
        }
    }
    Ok(rendered)
}

fn format_position(position: Position) -> Result<String, String> {
    if position.col > 7 || position.row > 7 {
        return Err(format!(
            "position out of range for transcript rendering: row={}, col={}",
            position.row, position.col
        ));
    }
    let file = (b'a' + position.col) as char;
    let rank = (b'1' + position.row) as char;
    Ok(format!("{file}{rank}"))
}

fn read_json<T>(path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let data =
        fs::read(path).map_err(|err| format!("read artifact file {}: {err}", path.display()))?;
    serde_json::from_slice(&data)
        .map_err(|err| format!("decode artifact file {}: {err}", path.display()))
}

#[derive(Debug, Deserialize)]
struct RecordArtifact {
    event_log: Vec<ArtifactEvent>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactEvent {
    #[serde(default)]
    seq: i32,
    kind: String,
    payload: Option<TurnResultPayload>,
}

#[derive(Debug, Clone, Deserialize)]
struct TurnResultPayload {
    action_status: Option<String>,
    action: Option<Action>,
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn reads_transcript_from_match_directory_record_fixture() {
        let fixture_dir = fixture_root().join("record-with-pass");
        let transcript = load_artifact_transcript(&fixture_dir).expect("load transcript");
        assert_eq!(
            transcript.entries(),
            &[
                TranscriptEntry::Move(Position { row: 3, col: 2 }),
                TranscriptEntry::Move(Position { row: 2, col: 4 }),
                TranscriptEntry::Pass,
                TranscriptEntry::Move(Position { row: 4, col: 5 }),
            ]
        );
    }

    #[test]
    fn resolves_single_match_subdirectory_from_output_dir() {
        let temp_dir = unique_temp_dir("kifu-output-dir");
        let output_dir = temp_dir.join("output");
        let match_dir = output_dir.join("sample-match");
        fs::create_dir_all(&match_dir).expect("create match dir");
        fs::write(match_dir.join(HISTORY_JSON), fixture_history_only_bytes())
            .expect("write history");

        let transcript = load_artifact_transcript(&output_dir).expect("load transcript");
        assert_eq!(
            transcript.entries(),
            &[
                TranscriptEntry::Move(Position { row: 3, col: 2 }),
                TranscriptEntry::Move(Position { row: 2, col: 4 }),
                TranscriptEntry::Pass,
                TranscriptEntry::Move(Position { row: 4, col: 5 }),
            ]
        );
    }

    #[test]
    fn prefers_record_over_history_when_both_exist() {
        let temp_dir = unique_temp_dir("kifu-precedence");
        fs::create_dir_all(&temp_dir).expect("create dir");
        fs::write(temp_dir.join(RECORD_JSON), fixture_record_bytes()).expect("write record");
        fs::write(
            temp_dir.join(HISTORY_JSON),
            br#"[{"seq":1,"kind":"turn_result","payload":{"action_status":"accepted","action":{"kind":"place","position":{"row":0,"col":0}}}}]"#,
        )
        .expect("write history");

        let transcript = load_artifact_transcript(&temp_dir).expect("load transcript");
        assert_eq!(transcript.render_compact().expect("compact"), "c4e3f5");
    }

    #[test]
    fn compact_output_drops_pass() {
        let fixture_dir = fixture_root().join("record-with-pass");
        let transcript = load_artifact_transcript(&fixture_dir).expect("load transcript");
        assert_eq!(transcript.render_compact().expect("compact"), "c4e3f5");
    }

    #[test]
    fn lossless_output_keeps_pass() {
        let fixture_dir = fixture_root().join("history-only");
        let transcript = load_artifact_transcript(&fixture_dir).expect("load transcript");
        assert_eq!(
            transcript.render_lossless().expect("lossless"),
            "c4e3passf5"
        );
    }

    fn fixture_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/reversi/artifacts/kifu-export")
    }

    fn fixture_record_bytes() -> Vec<u8> {
        fs::read(fixture_root().join("record-with-pass").join(RECORD_JSON)).expect("read record")
    }

    fn fixture_history_only_bytes() -> Vec<u8> {
        fs::read(fixture_root().join("history-only").join(HISTORY_JSON)).expect("read history")
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "reversi-ai-arena-{prefix}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("unix time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
