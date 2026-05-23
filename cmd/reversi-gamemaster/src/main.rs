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
        let response = handle_request(&mut game_master, request);
        encoder.encode(&response).map_err(|err| err.to_string())?;
        if response.id == "__shutdown__" {
            break;
        }
    }
    Ok(())
}

fn handle_request(game_master: &mut Option<ReversiGameMaster>, request: Request) -> Response {
    let request_id = request.id.clone().unwrap_or_default();
    let result = match request.method.as_str() {
        METHOD_METADATA => {
            Response::success(&request_id, &game_metadata()).map_err(|err| err.to_string())
        }
        METHOD_INITIALIZE_MATCH => {
            let params = request
                .parse_params::<gamemaster::InitializeMatchParams<SnapshotState>>()
                .map_err(|err| err.to_string());
            match params {
                Ok(params) => {
                    let resume_state = params
                        .resume_snapshot
                        .and_then(|snapshot| snapshot.game_state);
                    match ReversiGameMaster::initialize(
                        "reversi-match",
                        params.players,
                        resume_state,
                    ) {
                        Ok((master, init_state)) => {
                            *game_master = Some(master);
                            Response::success(
                                &request_id,
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
            Response::success(&request_id, &step).map_err(|err| err.to_string())
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
            Response::success(&request_id, &normalized).map_err(|err| err.to_string())
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
            Response::success(&request_id, &serde_json::json!({ "ok": true }))
                .map_err(|err| err.to_string())
        }),
        METHOD_CURRENT_SNAPSHOT => with_master(game_master, |master| {
            Response::success(&request_id, &master.current_snapshot())
                .map_err(|err| err.to_string())
        }),
        METHOD_CURRENT_EXPORTED_SNAPSHOT => with_master(game_master, |master| {
            Response::success(&request_id, &master.current_exported_snapshot())
                .map_err(|err| err.to_string())
        }),
        METHOD_CURRENT_RESULT => with_master(game_master, |master| {
            Response::success(&request_id, &master.current_result()).map_err(|err| err.to_string())
        }),
        METHOD_SHUTDOWN => {
            *game_master = None;
            Ok(Response {
                jsonrpc: aiarena_protocol::JSONRPC_VERSION.to_string(),
                id: "__shutdown__".to_string(),
                result: Some(serde_json::json!({ "ok": true })),
                error: None,
            })
        }
        _ => Err(format!("unsupported method {}", request.method)),
    };

    match result {
        Ok(response) => response,
        Err(message) => Response {
            jsonrpc: aiarena_protocol::JSONRPC_VERSION.to_string(),
            id: request_id,
            result: None,
            error: Some(ErrorObject {
                code: -32000,
                message,
            }),
        },
    }
}

fn with_master<F>(game_master: &mut Option<ReversiGameMaster>, f: F) -> Result<Response, String>
where
    F: FnOnce(&mut ReversiGameMaster) -> Result<Response, String>,
{
    let Some(master) = game_master.as_mut() else {
        return Err("match is not initialized".to_string());
    };
    f(master)
}
