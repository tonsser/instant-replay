use request::{Request};
use hyper::{Client, Response};
use hyper::error::{Error};
use hyper::client::{HttpConnector};
use tokio_core::reactor::Core;
use std::thread;
use std::time::{Duration, Instant};

pub struct RequestRunner {
    pub core: Core,
    pub client: Client<HttpConnector>,
}

impl RequestRunner {
    pub fn new() -> RequestRunner {
        let mut core = Core::new().unwrap();
        let client = Client::new(&core.handle());

        RequestRunner {
            core: core,
            client: client,
        }
    }

    pub fn run_request(&mut self, request: &Request) -> Result<Response, Error> {
        let uri = format!("http://api.tonsser.com{}", request.url).parse().unwrap();
        let request = self.client.get(uri);
        self.core.run(request)
    }
}
