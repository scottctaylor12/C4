use extism::*;

fn main() {
    let mut plugin: Plugin = init();
    //let input = "https://ifconfig.so".to_string();

    match plugin.call::<String, String>("c4", "".to_string()) {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error while calling 'c4': {:?}", e);
        }
    }
}

fn init() -> Plugin {
    let wasm_bytes = include_bytes!("c4.wasm");
    let wasm = Wasm::data(wasm_bytes);
    let manifest = Manifest::new([wasm])
        .with_allowed_hosts(vec!["*".to_string()].into_iter());
    let plugin = Plugin::new(&manifest, [], true).unwrap();
    return plugin;
}