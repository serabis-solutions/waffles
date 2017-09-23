extern crate config;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate tokio_core;

mod errors;
mod proxy;

use futures::{Future, Stream};

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use hyper::server::Http;
use errors::errors::*;
use hyper::Client;

fn run() -> Result<()> {
    info!("loading config from Waffles.toml");

    let mut config = config::Config::default();

    config
        .merge(config::File::with_name("Waffles"))
        .expect("Unable to load Waffles.toml")
        .merge(config::Environment::with_prefix("WAFFLES"))
        .expect("Failed to read ENV");

    let addr = format!(
        "{}:{}",
        config
            .get::<String>("listen.address")
            .expect("listen::address missing from config?"),
        config
            .get::<String>("listen.port")
            .expect("listen::port missing from config?")
    );

    let mut core = Core::new().expect("Failed to get handle from core");
    let handle = core.handle();

    let client = Client::new(&handle);
    let listener = TcpListener::bind(&(addr.parse().expect("Invalid listen address?")), &handle)
        .expect("Failed to setup listening server");

    info!("Listening on http://{}", listener.local_addr().unwrap());

    let http = Http::new();
    let connections = listener.incoming();
    let server = connections.for_each(|(sock, remote_addr)| {
        let service = proxy::Proxy {
            config: config.clone(),
            client: client.clone(),
        };
        futures::future::ok(remote_addr).and_then(|remote_addr| {
            http.bind_connection(&handle, sock, remote_addr, service);
            Ok(())
        })
    });
    core.run(server).expect("Unable to start server");
    Ok(())
}

fn main() {
    pretty_env_logger::init().unwrap();

    if let Err(e) = run() {
        eprintln!("error: {}", e);

        for e in e.iter().skip(1) {
            eprintln!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            eprintln!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}
