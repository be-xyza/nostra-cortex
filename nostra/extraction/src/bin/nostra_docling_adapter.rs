fn main() {
    if let Err(err) = nostra_extraction::parser_bridge::main_for_backend("docling") {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
