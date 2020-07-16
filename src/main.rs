use shortest_path_server::server;

fn main() -> Result<(), server::Error> {
    let cfg = server::Config {
        ip: Some("127.0.0.1".to_string()),
        port: Some(7777),
        read_timeout_millis: None,
    };
    server::run(&cfg)
}
