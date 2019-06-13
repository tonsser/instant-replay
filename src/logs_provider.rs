use futures::future;
use futures::{Future, Stream};
use hyper::error::Error;
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::fs::File;
use std::io::prelude::*;
use tokio_core::reactor::Core;

pub trait LogsProvider {
    fn get_logs(&self) -> String;
}

pub struct LogsFromFile {
    pub file_path: String,
}

impl LogsProvider for LogsFromFile {
    fn get_logs(&self) -> String {
        let mut file = File::open(self.file_path.clone()).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");
        contents
    }
}

pub struct LogsFromRemoteFile {
    pub url: String,
}

impl LogsProvider for LogsFromRemoteFile {
    fn get_logs(&self) -> String {
        let mut core = Core::new().unwrap();

        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        let uri = self.url.parse().unwrap();
        let work = client.get(uri).and_then(|res| {
            res.body().fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&*chunk);
                future::ok::<_, Error>(acc)
            })
        });
        let data = core.run(work).unwrap();
        String::from_utf8_lossy(&data).to_string()
    }
}
