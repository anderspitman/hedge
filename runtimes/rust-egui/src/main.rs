use hedgebrowser::HedgeApp;

fn main() {

    let args: Vec<String> = std::env::args().collect();
    let wasm_path = &args[1];
    //hedge::main(wasm_path.into());

    let _ = eframe::run_native(
        "hedgebrowser",
        Default::default(),
        Box::new(|cc| Ok(Box::new(HedgeApp::new(cc, wasm_path.into())))),
    );
}
