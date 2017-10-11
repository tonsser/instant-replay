extern crate instant_replay;
extern crate futures;
extern crate hyper;

use instant_replay::{get_thread_count_from_args, AccessTokenLoader, InstantReplay};
use instant_replay::logs_provider::{LogsFromRemoteFile, LogsProvider};
use std::time::Duration;

struct LoadAccessTokenFromDatabase;

impl AccessTokenLoader for LoadAccessTokenFromDatabase {
    fn access_token_from_user_slug(&self, user_slug: &String) -> Option<String> {
        None
    }
}

fn main() {
    let remote_logs = LogsFromRemoteFile {
        url: "https://tonsser-prod-file-uploads.s3-eu-west-1.amazonaws.com/uploads/af50726397f580ca73d1-wtf".to_string(),
    };

    InstantReplay {
        // logs_provider: LogsFromFile { file_path: "test_fixtures/logs".to_string() },
        logs_provider: remote_logs,

        access_token_loader: LoadAccessTokenFromDatabase,
        thread_count: get_thread_count_from_args(),
        run_for: Duration::from_secs(10),
    }.run();
}
