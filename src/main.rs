#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate config;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate pretty_env_logger;
#[cfg(test)]
extern crate reqwest;
extern crate tokio_core;
extern crate tokio_tls;

mod errors;
mod proxy;
mod tests;

use futures::{Future, Stream};

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use hyper::server::Http;
use errors::*;
use hyper::Client;
use std::io::Read;
use std::fs::File;
use native_tls::{Pkcs12, TlsAcceptor};
use config::Config;
use tokio_tls::TlsAcceptorExt;

fn run() -> Result<()> {
    info!("loading config from Waffles.toml");

    let mut config = config::Config::default();

    config
        .merge(config::File::with_name("Waffles"))
        .expect("Unable to load Waffles.toml")
        .merge(config::Environment::with_prefix("waffles"))
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


    if config
        .get::<bool>("listen.secure")
        .expect("listen::secure is missing from config?")
    {
        run_https(&config, &addr, &handle, &mut core);
    } else {
        run_http(&config, &addr, &handle, &mut core);
    }

    Ok(())
}

fn run_https(config: &Config, addr: &str, handle: &tokio_core::reactor::Handle, core: &mut Core) {
    let client = Client::new(handle);
    let http = Http::new();
    let sock = TcpListener::bind(&(addr.parse().expect("Invalid listen address?")), handle)
        .expect("Failed to setup listening server");

    info!(
        "Listening on https://{}",
        sock.local_addr()
            .expect("Unable to parse listening address")
    );

    let mut cert_file = File::open(&config
        .get::<String>("listen.certificate")
        .expect("No listen::certificate file specified"))
        .expect("Failed to open certificate file");
    let mut cert = Vec::new();

    cert_file
        .read_to_end(&mut cert)
        .expect("Failed to read in cert file");

    let pkcs12 = Pkcs12::from_der(
        &cert,
        &config
            .get::<String>("listen.password")
            .expect("No certificate password found in config"),
    ).expect("Failed to parse pkcs12 certificates, is the password correct?");

    let acceptor = TlsAcceptor::builder(pkcs12)
        .expect("Failed to build https builder")
        .build()
        .expect("Failed to build https listener");

    let server = sock.incoming().for_each(|(sock, remote_addr)| {
        let service = proxy::Proxy {
            config: config.clone(),
            client: client.clone(),
        };
        acceptor
            .accept_async(sock)
            .join(Ok(remote_addr))
            .and_then(|(sock, remote_addr)| {
                http.bind_connection(handle, sock, remote_addr, service);
                Ok(())
            })
            .or_else(|e| {
                info!("Error accepting TLS connection: {}", e);
                Ok(())
            })
    });
    core.run(server).expect("Unable to start https server");
}

fn run_http(config: &Config, addr: &str, handle: &tokio_core::reactor::Handle, core: &mut Core) {
    let client = Client::new(handle);
    let listener = TcpListener::bind(&(addr.parse().expect("Invalid listen address?")), handle)
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
            http.bind_connection(handle, sock, remote_addr, service);
            Ok(())
        })
    });

    core.run(server).expect("Unable to start server");
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
