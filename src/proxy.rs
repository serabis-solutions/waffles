use futures;
use futures::Future;
use hyper;
use hyper::{Body, Client, StatusCode};
use hyper::client::HttpConnector;
use hyper::server::{Request, Response, Service};
use config::Config;

pub struct Proxy {
    pub config: Config,
    pub client: Client<HttpConnector, Body>,
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        proxy_request(&self.config, &self.client, &req)
    }
}


fn proxy_request(
    config: &Config,
    client: &Client<HttpConnector, Body>,
    req: &Request,
) -> Box<Future<Item = Response, Error = hyper::Error>> {
    let proxy_url = format!(
        "http://{}:{}",
        config
            .get::<String>("proxy.address")
            .expect("unable to load proxy address from config"),
        config
            .get::<String>("proxy.port")
            .expect("unable to load proxy address from config")
    );

    debug!("Proxy url: {}", proxy_url);
    let mut proxied_request: Request = hyper::client::Request::new(
        req.method().clone(),
        proxy_url
            .parse::<hyper::Uri>()
            .expect("Unable to parse URI"),
    );

    *proxied_request.headers_mut() = req.headers().clone();

    let req = client.request(proxied_request);

    Box::new(req.then(|res| if let Ok(res) = res {
        debug!("Got a response!");
        futures::future::ok(
            Response::new()
                .with_status(res.status())
                .with_headers(res.headers().clone())
                .with_body(res.body()),
        )
    } else {
        debug!("Didn't got a response! :(");
        debug!("Error? {:?}", res);
        futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable))
    }))
}
