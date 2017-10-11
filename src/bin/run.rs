extern crate instant_replay;
extern crate futures;
extern crate hyper;

use instant_replay::{Request, RequestRunner};
use instant_replay::time_iter::repeat_for;
use std::io::{self, Write};
use std::time::{Duration};
use std::thread::{spawn, JoinHandle};
use std::sync::Arc;

fn main() {
    let reqs = Request::from_logs_file(&"test_fixtures/logs".to_string());
    let req = Arc::new(reqs.first().unwrap().clone());

    let thread_count = 10;

    let threads = (1..(thread_count + 1)).map(|_| {
        let req = Arc::clone(&req);

        spawn(move || {
            let mut runner = RequestRunner::new();

            for _ in repeat_for(Duration::from_secs(10)) {
                match runner.run_request(&req) {
                    Ok(_) => print!("."),
                    Err(_) => print!("f"),
                };
                io::stdout().flush().expect("Flushing failed");
            }
        })
    }).collect::<Vec<JoinHandle<()>>>();

    for t in threads {
        t.join().unwrap();
    }
}
