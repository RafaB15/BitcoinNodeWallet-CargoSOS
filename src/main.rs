use std::env;
use settings_implementation::settings::Settings;
//use settings_implementation::p2p_protocol::ProtocolVersionP2P;
//use settings_implementation::ibd_methods::IBDMethod;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ruta = args.get(1).unwrap();
    let configuración = Settings::new(ruta);
    println!("{:?}", configuración);
}
