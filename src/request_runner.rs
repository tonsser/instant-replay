use request::{Request};
use hyper;
use hyper::{Client, Response, Method};
use hyper::error::{Error};
use hyper::client::{HttpConnector};
use tokio_core::reactor::Core;

pub trait PrepareHttpRequest {
    fn call(&self, req_def: &Request, mut request: hyper::Request) -> hyper::Request;
}

pub struct RequestRunner<T: PrepareHttpRequest> {
    pub core: Core,
    pub client: Client<HttpConnector>,
    pub prepare_http_request: T,
}

impl<T: PrepareHttpRequest> RequestRunner<T> {
    pub fn new(prepare_http_request: T) -> RequestRunner<T> {
        let core = Core::new().unwrap();
        let client = Client::new(&core.handle());

        RequestRunner {
            core: core,
            client: client,
            prepare_http_request: prepare_http_request,
        }
    }

    pub fn run_request(&mut self, host: &String, request: &Request) -> Result<Response, Error> {
        let uri = format!("{}/{}", host, request.url).parse().unwrap();

        let http_req: hyper::Request = hyper::Request::new(Method::Get, uri);
        let http_req = self.prepare_http_request.call(&request, http_req);

        let get = self.client.request(http_req);
        self.core.run(get)
    }
}
