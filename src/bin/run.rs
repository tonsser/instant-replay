extern crate instant_replay;
extern crate futures;
extern crate hyper;
extern crate postgres;

use instant_replay::{get_thread_count_from_args, AccessTokenLoader, InstantReplay};
use instant_replay::logs_provider::{LogsFromRemoteFile};
use std::time::Duration;
use postgres::{Connection, TlsMode};
use std::collections::HashMap;

struct LoadAccessTokenFromDatabase {
    connection: Connection,
    cache: HashMap<String, String>,
}

impl LoadAccessTokenFromDatabase {
    fn new() -> Self {
        let connection = Connection::connect(
            "postgres://postgres@localhost/tonsser-api_development",
            TlsMode::None,
            ).expect("failed to connect");
        LoadAccessTokenFromDatabase {
            connection: connection,
            cache: HashMap::new(),
        }
    }
}

impl AccessTokenLoader for LoadAccessTokenFromDatabase {
    fn access_token_from_user_slug(&mut self, user_slug: &String) -> Option<String> {
        println!("{}", user_slug);

        match self.cache.get(user_slug) {
            Some(token) => return Some(token.clone()),
            _ => (),
        }

        let mut token = None;

        let sql = r#"
            SELECT
                users.slug AS slug,
                oauth_access_tokens.token AS token
            FROM users
            INNER JOIN oauth_access_tokens ON resource_owner_id = users.id
            WHERE slug = $1
            LIMIT 1
            "#;

        let rows = &self.connection.query(sql, &[&user_slug]).expect("query failed");

        for row in rows {
            let users_token: String = row.get("token");
            token = Some(users_token.clone());
            self.cache.insert(user_slug.clone(), users_token);
        }

        token
    }
}

fn main() {
    let duration = Duration::from_secs(60);

    InstantReplay {
        access_token_loader: LoadAccessTokenFromDatabase::new(),
        logs_provider: LogsFromRemoteFile {
            url: "https://tonsser-prod-file-uploads.s3-eu-west-1.amazonaws.com/uploads/af50726397f580ca73d1-wtf".to_string()
        },
        thread_count: get_thread_count_from_args(),
        run_for: duration,
        host: "http://api.tonsser.com".to_string(),
    }.run();
}
