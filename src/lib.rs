extern crate regex;
extern crate hyper_tls;
#[macro_use] extern crate lazy_static;
extern crate futures;
#[macro_use] extern crate hyper;
extern crate tokio_core;

mod request;
pub use request::*;

mod request_runner;
pub use request_runner::*;

pub mod time_iter;
pub mod repeat;
pub mod logs_provider;
use logs_provider::{LogsProvider};

use repeat::repeat;
use std::env;
use std::io::{self, Write};
use std::process::{exit};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{spawn, JoinHandle};
use std::time::{Duration};
use time_iter::repeat_for;
use hyper::header::Connection;

pub struct InstantReplay<T: AccessTokenLoader, K: LogsProvider> {
    pub access_token_loader: T,
    pub logs_provider: K,
    pub thread_count: i32,
    pub run_for: Duration,
    pub host: String,
}

impl<T: AccessTokenLoader, K: LogsProvider> InstantReplay<T, K> {
    pub fn run(self) -> usize {
        let requests = Arc::new(
            Request::from_logs_file(
                &self.logs_provider.get_logs(),
                self.access_token_loader,
                )
            );

        let host = Arc::new(self.host.clone());
        let duration = self.run_for.clone();
        let requests_run = Arc::new(AtomicUsize::new(0));

        let threads = repeat(self.thread_count).map(|_| {
            let requests = Arc::clone(&requests);
            let host = Arc::clone(&host);
            let requests_run = Arc::clone(&requests_run);

            spawn(move || {
                let request_preppers: Vec<Box<PrepareHttpRequest>> = vec![
                    Box::new(SetConnectionHeader),
                    Box::new(SetAuthHeader),
                    Box::new(SetBenchmarkRequestHeader),
                ];

                let mut runner = RequestRunner::new(request_preppers);
                let mut iteration = 0;

                if requests.len() == 0 {
                    println!("No requests");
                    return;
                }

                for _ in repeat_for(duration) {
                    iteration += 1;

                    let index = iteration % requests.len();
                    let request = requests.get(index)
                        .expect("failed to get request");

                    match runner.run_request(&host, &request) {
                        Ok(_) => print!("."),
                        Err(_) => print!("f"),
                    };
                    requests_run.fetch_add(1, Ordering::SeqCst);

                    io::stdout().flush().expect("Flushing failed");
                }
            })
        }).collect::<Vec<JoinHandle<()>>>();

        for t in threads {
            t.join().expect("couldn't join thread");
        }

        requests_run.load(Ordering::SeqCst)
    }
}

pub fn get_thread_count_from_args() -> i32 {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        println!("Needs exactly one arg");
        exit(1);
    }

    args.last().unwrap().parse().unwrap()
}

header! { (Authorization, "Authorization") => [String] }
header! { (XBenchmarkRequest, "X-Benchmark-Request") => [bool] }

struct SetAuthHeader;
impl PrepareHttpRequest for SetAuthHeader {
    fn call(&self, req_def: &Request, mut request: hyper::Request) -> hyper::Request {
        let auth_header_value = format!("Bearer {}", req_def.access_token.clone());
        request.headers_mut().set(Authorization(auth_header_value));
        request
    }
}

struct SetConnectionHeader;
impl PrepareHttpRequest for SetConnectionHeader {
    fn call(&self, _req_def: &Request, mut request: hyper::Request) -> hyper::Request {
        request.headers_mut().set(Connection::close());
        request
    }
}

struct SetBenchmarkRequestHeader;
impl PrepareHttpRequest for SetBenchmarkRequestHeader {
    fn call(&self, _req_def: &Request, mut request: hyper::Request) -> hyper::Request {
        request.headers_mut().set(XBenchmarkRequest(true));
        request
    }
}

impl PrepareHttpRequest for Vec<Box<PrepareHttpRequest>> {
    fn call(&self, req_def: &Request, mut request: hyper::Request) -> hyper::Request {
        for p in self {
            request = p.call(req_def, request);
        }
        request
    }
}
