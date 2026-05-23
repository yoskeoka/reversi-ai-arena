fn main() {
    let mut args = std::env::args().skip(1);
    let Some(flag) = args.next() else {
        eprintln!("expected --moves <sequence>");
        std::process::exit(1);
    };
    if flag != "--moves" {
        eprintln!("expected --moves <sequence>");
        std::process::exit(1);
    }
    let Some(moves) = args.next() else {
        eprintln!("expected --moves <sequence>");
        std::process::exit(1);
    };
    let mut strategy = match reversi_fixture_bots::ScriptedStrategy::from_moves(&moves) {
        Ok(strategy) => strategy,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = reversi_fixture_bots::run_fixture_loop(|params| strategy.next_action(params))
    {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
