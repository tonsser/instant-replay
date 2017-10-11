use request::{Request};
use hyper;
use hyper::{Client, Response, Method};
use hyper::error::{Error};
use hyper::client::{HttpConnector};
use tokio_core::reactor::Core;

header! { (Authorization, "Authorization") => [String] }

pub struct RequestRunner {
    pub core: Core,
    pub client: Client<HttpConnector>,
}

impl RequestRunner {
    pub fn new() -> RequestRunner {
        let core = Core::new().unwrap();
        let client = Client::new(&core.handle());

        RequestRunner {
            core: core,
            client: client,
        }
    }

    pub fn run_request(&mut self, host: &String, request: &Request) -> Result<Response, Error> {
        let uri = format!("{}/{}", host, request.url).parse().unwrap();
        let mut http_req = hyper::Request::new(Method::Get, uri);

        let header = format!("Bearer {}", request.access_token.clone());

        // println!(
        //     "token: {}, slug: {}, url: {}",
        //     request.access_token,
        //     request.user_slug,
        //     request.url,
        //     );

        http_req.headers_mut().set(Authorization(header));
        let get = self.client.request(http_req);
        self.core.run(get)
    }
}
