fn main() {
    if let Err(err) =
        reversi_fixture_bots::run_fixture_loop(reversi_fixture_bots::choose_first_legal)
    {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
