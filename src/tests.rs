#[cfg(test)]
mod proxy_tests {
    use std::env;
    use std::thread;
    use reqwest;
    use main;
    use std::io::Read;

    /* So to test we'll just cheat and use our hard coded test server */
    fn setup_http_env() {
        env::set_var("WAFFLES_listen.address", "127.0.0.1");
        env::set_var("WAFFLES_listen.port", "9876");
        env::set_var("WAFFLES_proxy.port", "80");
        env::set_var("WAFFLES_proxy.address", "example.org");
        env::set_var("WAFFLES_listen.secure", "false");
    }

    #[test]
    fn setup_http_tests() {
        setup_http_env();

        thread::spawn(move || {
            main();
        });
    }

    #[test]
    fn http_200_ok() {
        let mut res = simple_get("http://www.example.org/");

        if !res.status().is_success() {
            panic!("Response was not a success: {:?}", res);
        }

        let mut content = String::new();
        res.read_to_string(&mut content).unwrap();

        if !content.contains("<h1>Example Domain</h1>") {
            panic!("Response does not contain <h1>Example Domain</h1>");
        }
    }

    fn simple_get(url: &str) -> reqwest::Response {
        let client = reqwest::Client::builder()
            .expect("Unable to build reqwest builder")
            .proxy(
                reqwest::Proxy::http("http://127.0.0.1:9876")
                    .expect("Unable to build reqwest proxy"),
            )
            .build()
            .expect("Unable to build reqwest client");
        client.get(url).unwrap().send().unwrap()
    }
}
