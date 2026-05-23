use std::io::{self, BufReader};

use aiarena_protocol::{
    Decoder, Encoder, ErrorObject, Request, Response,
    gamemaster::{
        self, METHOD_APPLY_DECISION_RESULTS, METHOD_CURRENT_EXPORTED_SNAPSHOT,
        METHOD_CURRENT_RESULT, METHOD_CURRENT_SNAPSHOT, METHOD_INITIALIZE_MATCH, METHOD_METADATA,
        METHOD_NEXT_DECISION_STEP, METHOD_NORMALIZE_ACTION, METHOD_SHUTDOWN,
    },
};
use reversi_game::{
    game_metadata,
    gamemaster::{ReversiGameMaster, SnapshotState},
};

struct RequestOutcome {
    response: Option<Response>,
    shutdown: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut decoder = Decoder::new(BufReader::new(stdin.lock()));
    let mut encoder = Encoder::new(stdout.lock());
    let mut game_master: Option<ReversiGameMaster> = None;

    while let Some(request) = decoder.decode_request().map_err(|err| err.to_string())? {
        let outcome = handle_request(&mut game_master, request);
        if let Some(response) = outcome.response {
            encoder.encode(&response).map_err(|err| err.to_string())?;
        }
        if outcome.shutdown {
            break;
        }
    }
    Ok(())
}

fn handle_request(game_master: &mut Option<ReversiGameMaster>, request: Request) -> RequestOutcome {
    let request_id = request.id.clone();
    let result = match request.method.as_str() {
        METHOD_METADATA => {
            success_response(request_id.as_deref(), &game_metadata()).map_err(|err| err.to_string())
        }
        METHOD_INITIALIZE_MATCH => {
            let params = request
                .parse_params::<gamemaster::InitializeMatchParams<
                    SnapshotState,
                    reversi_game::VisibleState,
                    reversi_game::Action,
                >>()
                .map_err(|err| err.to_string());
            match params {
                Ok(params) => {
                    let (match_id, resume_snapshot) = match params.resume_snapshot {
                        Some(snapshot) => (snapshot.match_id.clone(), Some(snapshot)),
                        None => ("reversi-match".to_string(), None),
                    };
                    match ReversiGameMaster::initialize(match_id, params.players, resume_snapshot) {
                        Ok((master, init_state)) => {
                            *game_master = Some(master);
                            success_response(
                                request_id.as_deref(),
                                &gamemaster::InitializeMatchResult { init_state },
                            )
                            .map_err(|err| err.to_string())
                        }
                        Err(err) => Err(err),
                    }
                }
                Err(err) => Err(err),
            }
        }
        METHOD_NEXT_DECISION_STEP => with_master(game_master, |master| {
            let step = master.next_decision_step();
            success_response(request_id.as_deref(), &step).map_err(|err| err.to_string())
        }),
        METHOD_NORMALIZE_ACTION => with_master(game_master, |master| {
            let params = request
                .parse_params::<gamemaster::NormalizeActionParams<
                    reversi_game::VisibleState,
                    reversi_game::LegalActionHint,
                    serde_json::Value,
                >>()
                .map_err(|err| err.to_string())?;
            let normalized = master.normalize_action(&params.request, &params.action_status);
            success_response(request_id.as_deref(), &normalized).map_err(|err| err.to_string())
        }),
        METHOD_APPLY_DECISION_RESULTS => with_master(game_master, |master| {
            let params = request
                .parse_params::<gamemaster::ApplyDecisionResultsParams<
                    reversi_game::VisibleState,
                    reversi_game::LegalActionHint,
                    reversi_game::Action,
                >>()
                .map_err(|err| err.to_string())?;
            master.apply_decision_results(&params.action_statuses)?;
            success_response(request_id.as_deref(), &serde_json::json!({ "ok": true }))
                .map_err(|err| err.to_string())
        }),
        METHOD_CURRENT_SNAPSHOT => with_master(game_master, |master| {
            success_response(request_id.as_deref(), &master.current_snapshot())
                .map_err(|err| err.to_string())
        }),
        METHOD_CURRENT_EXPORTED_SNAPSHOT => with_master(game_master, |master| {
            success_response(request_id.as_deref(), &master.current_exported_snapshot())
                .map_err(|err| err.to_string())
        }),
        METHOD_CURRENT_RESULT => with_master(game_master, |master| {
            success_response(request_id.as_deref(), &master.current_result())
                .map_err(|err| err.to_string())
        }),
        METHOD_SHUTDOWN => {
            *game_master = None;
            success_response(request_id.as_deref(), &serde_json::json!({ "ok": true }))
                .map_err(|err| err.to_string())
        }
        _ => Err(format!("unsupported method {}", request.method)),
    };

    match result {
        Ok(response) => RequestOutcome {
            response,
            shutdown: request.method == METHOD_SHUTDOWN,
        },
        Err(message) => RequestOutcome {
            response: error_response(request_id.as_deref(), message),
            shutdown: false,
        },
    }
}

fn with_master<F>(
    game_master: &mut Option<ReversiGameMaster>,
    f: F,
) -> Result<Option<Response>, String>
where
    F: FnOnce(&mut ReversiGameMaster) -> Result<Option<Response>, String>,
{
    let Some(master) = game_master.as_mut() else {
        return Err("match is not initialized".to_string());
    };
    f(master)
}

fn success_response<T: serde::Serialize>(
    request_id: Option<&str>,
    result: &T,
) -> Result<Option<Response>, serde_json::Error> {
    match request_id {
        Some(id) => Response::success(id, result).map(Some),
        None => Ok(None),
    }
}

fn error_response(request_id: Option<&str>, message: String) -> Option<Response> {
    let id = request_id?;
    Some(Response {
        jsonrpc: aiarena_protocol::JSONRPC_VERSION.to_string(),
        id: id.to_string(),
        result: None,
        error: Some(ErrorObject {
            code: -32000,
            message,
        }),
    })
}
