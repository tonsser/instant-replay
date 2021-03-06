extern crate hyper_tls;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate futures;
#[macro_use]
extern crate hyper;
extern crate tokio_core;

mod request;
pub use crate::request::*;

mod request_runner;
pub use crate::request_runner::PrepareHttpRequest;
use crate::request_runner::*;

pub mod logs_provider;
pub mod repeat;
pub mod time_iter;
use crate::logs_provider::LogsProvider;

use crate::repeat::repeat;
use crate::time_iter::repeat_for;
use hyper::header::Connection;
use std::env;
use std::io::{self, Write};
use std::process::exit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

pub struct InstantReplay<T, K, U>
where
    T: AccessTokenLoader,
    K: LogsProvider,
    U: PrepareHttpRequest + Copy + Send + 'static,
{
    pub access_token_loader: T,
    pub prepare_http_request: Option<U>,
    pub logs_provider: K,
    pub thread_count: i32,
    pub run_for: Duration,
    pub host: String,
}

impl<T, K, U> InstantReplay<T, K, U>
where
    T: AccessTokenLoader,
    K: LogsProvider,
    U: PrepareHttpRequest + Copy + Send,
{
    pub fn run(self) -> usize {
        let requests = Arc::new(Request::from_logs_file(
            &self.logs_provider.get_logs(),
            self.access_token_loader,
        ));

        let host = Arc::new(self.host.clone());
        let duration = self.run_for.clone();
        let requests_run = Arc::new(AtomicUsize::new(0));
        let prepare_http_request = self.prepare_http_request;

        let threads = repeat(self.thread_count)
            .map(|_| {
                let requests = Arc::clone(&requests);
                let host = Arc::clone(&host);
                let requests_run = Arc::clone(&requests_run);

                spawn(move || {
                    let mut request_prepper: Vec<Box<PrepareHttpRequest>> = vec![
                        Box::new(SetConnectionHeader),
                        Box::new(SetBenchmarkRequestHeader),
                    ];

                    match prepare_http_request {
                        Some(p) => request_prepper.push(Box::new(p)),
                        None => {}
                    }

                    let mut runner = RequestRunner::new(request_prepper);
                    let mut iteration = 0;

                    if requests.len() == 0 {
                        println!("No requests");
                        return;
                    }

                    for _ in repeat_for(duration) {
                        iteration += 1;

                        let index = iteration % requests.len();
                        let request = requests.get(index).expect("failed to get request");

                        match runner.run_request(&host, &request) {
                            Ok(_) => print!("."),
                            Err(_) => print!("f"),
                        };
                        requests_run.fetch_add(1, Ordering::SeqCst);

                        io::stdout().flush().expect("Flushing failed");
                    }
                })
            })
            .collect::<Vec<JoinHandle<()>>>();

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

header! { (XBenchmarkRequest, "X-Benchmark-Request") => [bool] }

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
