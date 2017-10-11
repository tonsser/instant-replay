extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::io::{self, Write};
use hyper::Client;
use tokio_core::reactor::Core;
use std::thread;
use std::time::{Duration, Instant};
use std::env;
use std::process::{exit};

fn get_thread_count_from_args() -> i32 {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!("Needs exactly one arg");
        exit(1);
    }
    args.last().unwrap().parse().unwrap()
}

fn main() {
    let thread_count = get_thread_count_from_args();
    let duration = Duration::from_secs(60);
    let start_time = Instant::now();

    let threads: Vec<thread::JoinHandle<()>> = repeat(thread_count).map(move |_| {
        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let client = Client::new(&core.handle());

            loop {
                if start_time.elapsed() > duration {
                    break
                }

                let uri = "http://api.tonsser.com/bootstrap".parse().unwrap();
                let request = client.get(uri);
                match core.run(request) {
                    Ok(_) => print!("."),
                    Err(_) => print!("f"),
                }

                io::stdout().flush().expect("Flushing failed");

            }
        })
    }).collect();

    for thread in threads {
        thread.join().expect("Join failed");
    }
}

fn repeat(max: i32) -> Repeat {
    Repeat { max: max, iteration: 0 }
}

struct Repeat {
    max: i32,
    iteration: i32,
}

impl Iterator for Repeat {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        if self.iteration >= self.max {
            Option::None
        } else {
            let val = Option::Some(self.iteration);
            self.iteration += 1;
            val
        }
    }
}
