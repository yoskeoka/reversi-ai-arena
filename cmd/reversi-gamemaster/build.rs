use std::{env, fs, path::PathBuf};

fn main() {
    if env::var_os("CARGO_FEATURE_XOT").is_none() {
        return;
    }
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../games/reversi/data/xot");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo build output"));

    for (name, rows, digest) in [
        (
            "openingslarge.txt",
            10_784,
            "ccbeb0ed7b7ae23de20013a3992b9a6ede77610bbb5cacda82ccbaedaab72530",
        ),
        (
            "openingssmall.txt",
            3_623,
            "8a441b077b1093af7b0d1cb55fa5a0d3d1700ce200a1b2d5b6e28d7b17139580",
        ),
    ] {
        let path = source.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        let data = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        reversi_game::opening::validate_corpus(&data, rows, digest)
            .unwrap_or_else(|err| panic!("validate {}: {err}", path.display()));
        fs::write(output.join(name), data).expect("write validated XOT data");
    }
}
