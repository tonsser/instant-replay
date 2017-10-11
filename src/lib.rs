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

use std::env;
use std::process::{exit};
use std::time::{Duration};
use std::sync::Arc;
use time_iter::repeat_for;
use repeat::repeat;
use std::thread::{spawn, JoinHandle};
use std::io::{self, Write};

pub struct InstantReplay<T: AccessTokenLoader, K: LogsProvider> {
    pub access_token_loader: T,
    pub logs_provider: K,
    pub thread_count: i32,
    pub run_for: Duration,
    pub host: String,
}

impl<T: AccessTokenLoader, K: LogsProvider> InstantReplay<T, K> {
    pub fn run(self) {
        let requests = Arc::new(
            Request::from_logs_file(
                &self.logs_provider.get_logs(),
                self.access_token_loader,
                )
            );

        println!("thread_count: {}", self.thread_count);

        let host = Arc::new(self.host.clone());
        let duration = self.run_for.clone();

        let threads = repeat(self.thread_count).map(|_| {
            let requests = Arc::clone(&requests);
            let host = Arc::clone(&host);

            spawn(move || {
                let mut runner = RequestRunner::new();
                let mut iteration = 0;

                if requests.len() == 0 {
                    println!("No requests");
                    return;
                }

                println!("running for: {:?}", duration);
                for _ in repeat_for(duration) {
                    iteration += 1;

                    let index = iteration % requests.len();
                    let request = requests.get(index)
                        .expect("failed to get request");

                    match runner.run_request(&host, &request) {
                        Ok(_) => print!("."),
                        Err(_) => print!("f"),
                    };

                    io::stdout().flush().expect("Flushing failed");
                }
            })
        }).collect::<Vec<JoinHandle<()>>>();

        for t in threads {
            t.join().expect("couldn't join thread");
        }

        println!("\nthread_count: {}", self.thread_count);
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
