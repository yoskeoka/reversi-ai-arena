use std::io::{self, BufReader};

use aiarena_protocol::{
    Decoder, Encoder,
    player::{METHOD_GAME_OVER, METHOD_INIT, METHOD_TURN},
};
use reversi_rust_reference_player::{
    choose_action, decode_game_over_request, decode_init_request, decode_turn_request,
    game_over_ack_response, init_ready_response, turn_action_response,
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

    while let Some(request) = decoder
        .decode_request()
        .map_err(|err| format!("decode request: {err}"))?
    {
        let request_id = request
            .id
            .clone()
            .ok_or_else(|| format!("request {} is missing an id", request.method))?;
        match request.method.as_str() {
            METHOD_INIT => {
                let _params = decode_init_request(&request)
                    .map_err(|err| format!("decode init request: {err}"))?;
                let response = init_ready_response(&request_id)
                    .map_err(|err| format!("encode init response: {err}"))?;
                encoder
                    .encode(&response)
                    .map_err(|err| format!("write init response: {err}"))?;
            }
            METHOD_TURN => {
                let params = decode_turn_request(&request)
                    .map_err(|err| format!("decode turn request: {err}"))?;
                let action = choose_action(&params.visible_state, &params.legal_action_hint);
                let response = turn_action_response(&request_id, action)
                    .map_err(|err| format!("encode turn response: {err}"))?;
                encoder
                    .encode(&response)
                    .map_err(|err| format!("write turn response: {err}"))?;
            }
            METHOD_GAME_OVER => {
                let _params = decode_game_over_request(&request)
                    .map_err(|err| format!("decode game_over request: {err}"))?;
                let response = game_over_ack_response(&request_id)
                    .map_err(|err| format!("encode game_over response: {err}"))?;
                encoder
                    .encode(&response)
                    .map_err(|err| format!("write game_over response: {err}"))?;
            }
            other => return Err(format!("unsupported method: {other}")),
        }
    }

    Ok(())
}
