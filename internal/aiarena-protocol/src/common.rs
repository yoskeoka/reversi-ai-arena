use std::fmt;
use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

pub const JSONRPC_VERSION: &str = "2.0";
pub const TRANSPORT_STDIO_JSONRPC_NDJSON: &str = "stdio-jsonrpc-ndjson";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub game_version: String,
    pub ruleset_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Request {
    pub fn new<T: Serialize>(
        id: impl Into<String>,
        method: impl Into<String>,
        params: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some(id.into()),
            method: method.into(),
            params: Some(serde_json::to_value(params)?),
        })
    }

    pub fn notification<T: Serialize>(
        method: impl Into<String>,
        params: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: None,
            method: method.into(),
            params: Some(serde_json::to_value(params)?),
        })
    }

    pub fn parse_params<T: DeserializeOwned>(&self) -> Result<T, DecodeError> {
        let params = self.params.clone().unwrap_or(Value::Null);
        serde_json::from_value(params).map_err(DecodeError::MalformedPayload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,
}

impl Response {
    pub fn success<T: Serialize>(
        id: impl Into<String>,
        result: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: id.into(),
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    pub fn parse_result<T: DeserializeOwned>(&self) -> Result<T, DecodeError> {
        let result = self.result.clone().unwrap_or(Value::Null);
        serde_json::from_value(result).map_err(DecodeError::MalformedPayload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorObject {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transport {
    StdioJsonrpcNdjson,
}

impl Transport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StdioJsonrpcNdjson => TRANSPORT_STDIO_JSONRPC_NDJSON,
        }
    }
}

#[derive(Debug)]
pub struct Encoder<W> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode<T: Serialize>(&mut self, value: &T) -> Result<(), io::Error> {
        serde_json::to_writer(&mut self.writer, value)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()
    }
}

#[derive(Debug)]
pub struct Decoder<R> {
    reader: R,
    line: String,
}

impl<R: BufRead> Decoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line: String::new(),
        }
    }

    pub fn decode_request(&mut self) -> Result<Option<Request>, DecodeError> {
        match self.read_line()? {
            Some(line) => decode_request_line(&line).map(Some),
            None => Ok(None),
        }
    }

    pub fn decode_response(&mut self) -> Result<Option<Response>, DecodeError> {
        match self.read_line()? {
            Some(line) => decode_response_line(&line).map(Some),
            None => Ok(None),
        }
    }

    fn read_line(&mut self) -> Result<Option<String>, DecodeError> {
        self.line.clear();
        let bytes = self
            .reader
            .read_line(&mut self.line)
            .map_err(DecodeError::Io)?;
        if bytes == 0 {
            return Ok(None);
        }
        Ok(Some(self.line.trim_end_matches(['\r', '\n']).to_string()))
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Io(io::Error),
    MalformedJson(serde_json::Error),
    InvalidEnvelope(&'static str),
    MismatchedId { expected: String, actual: String },
    InvalidVersion(String),
    InvalidMetadata(&'static str),
    MalformedPayload(serde_json::Error),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::MalformedJson(err) => write!(f, "malformed json: {err}"),
            Self::InvalidEnvelope(message) => write!(f, "invalid envelope: {message}"),
            Self::MismatchedId { expected, actual } => {
                write!(f, "mismatched id: expected {expected:?}, got {actual:?}")
            }
            Self::InvalidVersion(version) => write!(f, "invalid version: {version:?}"),
            Self::InvalidMetadata(message) => write!(f, "invalid metadata: {message}"),
            Self::MalformedPayload(err) => write!(f, "malformed payload: {err}"),
        }
    }
}

impl std::error::Error for DecodeError {}

pub fn decode_request_line(line: &str) -> Result<Request, DecodeError> {
    let request: Request = serde_json::from_str(line).map_err(DecodeError::MalformedJson)?;
    validate_request(&request)?;
    Ok(request)
}

pub fn decode_response_line(line: &str) -> Result<Response, DecodeError> {
    let response: Response = serde_json::from_str(line).map_err(DecodeError::MalformedJson)?;
    validate_response(&response)?;
    Ok(response)
}

pub fn match_response_id(expected: &str, response: &Response) -> Result<(), DecodeError> {
    if response.id == expected {
        Ok(())
    } else {
        Err(DecodeError::MismatchedId {
            expected: expected.to_string(),
            actual: response.id.clone(),
        })
    }
}

fn validate_request(request: &Request) -> Result<(), DecodeError> {
    if request.jsonrpc != JSONRPC_VERSION {
        return Err(DecodeError::InvalidEnvelope("jsonrpc must be 2.0"));
    }
    if request.method.is_empty() {
        return Err(DecodeError::InvalidEnvelope("method is required"));
    }
    Ok(())
}

fn validate_response(response: &Response) -> Result<(), DecodeError> {
    if response.jsonrpc != JSONRPC_VERSION {
        return Err(DecodeError::InvalidEnvelope("jsonrpc must be 2.0"));
    }
    if response.id.is_empty() {
        return Err(DecodeError::InvalidEnvelope("id is required"));
    }
    let has_result = response.result.is_some();
    let has_error = response.error.is_some();
    if has_result == has_error {
        return Err(DecodeError::InvalidEnvelope(
            "exactly one of result or error is required",
        ));
    }
    if let Some(error) = &response.error
        && error.message.is_empty()
    {
        return Err(DecodeError::InvalidEnvelope("error.message is required"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataCompatibilityError {
    GameIdMismatch { expected: String, actual: String },
    GameVersionMajorMismatch { expected: String, actual: String },
    RulesetMismatch { expected: String, actual: String },
}

impl fmt::Display for MetadataCompatibilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameIdMismatch { expected, actual } => {
                write!(f, "game_id mismatch: expected {expected:?}, got {actual:?}")
            }
            Self::GameVersionMajorMismatch { expected, actual } => write!(
                f,
                "game_version major mismatch: expected {expected:?}, got {actual:?}"
            ),
            Self::RulesetMismatch { expected, actual } => write!(
                f,
                "ruleset_version mismatch: expected {expected:?}, got {actual:?}"
            ),
        }
    }
}

impl std::error::Error for MetadataCompatibilityError {}

pub fn metadata_compatible(
    expected: &GameMetadata,
    actual: &GameMetadata,
) -> Result<(), MetadataCompatibilityError> {
    if expected.game_id != actual.game_id {
        return Err(MetadataCompatibilityError::GameIdMismatch {
            expected: expected.game_id.clone(),
            actual: actual.game_id.clone(),
        });
    }
    if major_version(&expected.game_version) != major_version(&actual.game_version) {
        return Err(MetadataCompatibilityError::GameVersionMajorMismatch {
            expected: expected.game_version.clone(),
            actual: actual.game_version.clone(),
        });
    }
    if expected.ruleset_version != actual.ruleset_version {
        return Err(MetadataCompatibilityError::RulesetMismatch {
            expected: expected.ruleset_version.clone(),
            actual: actual.ruleset_version.clone(),
        });
    }
    Ok(())
}

pub fn major_version(version: &str) -> &str {
    version.split('.').next().unwrap_or(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn request_round_trip_uses_ndjson() {
        let request = Request::new(
            "turn-1",
            "turn",
            &serde_json::json!({"turn": 1, "visible_state": {"turn": 1}}),
        )
        .expect("request");

        let mut buffer = Vec::new();
        Encoder::new(&mut buffer).encode(&request).expect("encode");
        assert!(buffer.ends_with(b"\n"));

        let mut decoder = Decoder::new(Cursor::new(buffer));
        let decoded = decoder.decode_request().expect("decode").expect("request");
        assert_eq!(decoded.method, "turn");
        assert_eq!(decoded.id.as_deref(), Some("turn-1"));
    }

    #[test]
    fn response_round_trip_accepts_crlf() {
        let response = decode_response_line(
            "{\"jsonrpc\":\"2.0\",\"id\":\"init\",\"result\":{\"ready\":true}}\r\n",
        )
        .expect("response");
        assert_eq!(response.id, "init");
    }

    #[test]
    fn invalid_envelope_is_rejected() {
        let err = decode_request_line("{\"jsonrpc\":\"1.0\",\"id\":\"x\",\"method\":\"turn\"}")
            .expect_err("invalid");
        assert!(matches!(err, DecodeError::InvalidEnvelope(_)));
    }

    #[test]
    fn mismatched_id_is_reported() {
        let response =
            Response::success("turn-2", &serde_json::json!({"action":"pass"})).expect("response");
        let err = match_response_id("turn-1", &response).expect_err("mismatch");
        assert!(matches!(err, DecodeError::MismatchedId { .. }));
    }

    #[test]
    fn metadata_compatibility_requires_matching_major_and_ruleset() {
        let expected = GameMetadata {
            game_id: "reversi".to_string(),
            game_version: "1.2.0".to_string(),
            ruleset_version: "standard".to_string(),
        };
        let compatible = GameMetadata {
            game_id: "reversi".to_string(),
            game_version: "1.9.3".to_string(),
            ruleset_version: "standard".to_string(),
        };
        metadata_compatible(&expected, &compatible).expect("compatible");

        let incompatible = GameMetadata {
            game_id: "reversi".to_string(),
            game_version: "2.0.0".to_string(),
            ruleset_version: "standard".to_string(),
        };
        let err = metadata_compatible(&expected, &incompatible).expect_err("major mismatch");
        assert!(matches!(
            err,
            MetadataCompatibilityError::GameVersionMajorMismatch { .. }
        ));
    }

    #[test]
    fn parse_params_reports_malformed_payload() {
        let request = Request {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: Some("bad".to_string()),
            method: "turn".to_string(),
            params: Some(serde_json::json!({"turn": "not-a-number"})),
        };
        let err = request
            .parse_params::<crate::player::TurnParams<Value, Value>>()
            .expect_err("payload");
        assert!(matches!(err, DecodeError::MalformedPayload(_)));
    }
}
