Instant Replay
==============

What is this?
-------------

When benchmarking an API it is very beneficial to be able to simulate real live traffic. Hitting just a few endpoints doesn't give you very accurate data on how your app performs when you have 10.000 users online. But simulating live traffic is tricky. One way to do it is to record actual traffic in a log file, find the requests in that file, and perform those requests in a loop across lots of threads. This project aims to remove the boilerplate from that process.

The project is currently tailored to parse logs from a Rails app running on Heroku, but could be extended to parse logs from anywhere.

Why Rust?
---------

Because Rust is very fast. We found that we weren't able to generate useful amounts of load using Ruby.

Sample setup
------------

The things that differ from app to app are:

- Where to find the logs.
- The URL to the API.
- How many threads to use and for how long to perform the benchmark.
- How to identify which user made a request and how to authenticate the request as that user.

Because of these reasons we decided to build this a library crate so you can easily write code to customize how to handle each of these things.

```rust
extern crate instant_replay;
extern crate postgres;

use instant_replay::{get_thread_count_from_args, AccessTokenLoader, InstantReplay};
use instant_replay::logs_provider::{LogsFromRemoteFile, LogsProvider};
use std::time::Duration;
use postgres::{Connection, TlsMode};
use std::collections::HashMap;

fn main() {
    InstantReplay {
        // Object that implements `AccessTokenLoader`
        // This particular implementation will load the access token from a postgres database running locally
        access_token_loader: LoadAccessTokenFromDatabase::new(),

        // Object that implements `LogsProvider`
        // This'll fetch the logs from a logs file uploaded to Amazon S3
        logs_provider: LogsFromRemoteFile {
            url: "https://my-bucket.s3-eu-west-1.amazonaws.com/logs".to_string()
        },

        thread_count: 200,

        run_for: Duration::from_secs(10),
    }.run();
}

// An example how to load access tokens from a database
struct LoadAccessTokenFromDatabase {
    connection: Connection,
    cache: HashMap<String, String>,
}

impl LoadAccessTokenFromDatabase {
    fn new() -> Self {
        let connection = Connection::connect(
            "postgres://postgres@localhost/database_name",
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
```

`InstantReplay` assumes your logs looks like this:

```
2017-10-10T10:47:53.385562+00:00 app[web.3]: [44bb3720-6e34-4951-8ab6-9304165172cc] [/26/teams/gj-la-haye-fouassier-u18-d2-masculin-groupe-d-93230] Started GET "/foo/bar" for 123.123.123.123 at 2017-10-10 10:47:53 +0000
2017-10-10T10:47:53.397556+00:00 app[web.3]: [44bb3720-6e34-4951-8ab6-9304165172cc] [/26/teams/gj-la-haye-fouassier-u18-d2-masculin-groupe-d-93230] current_user.slug => christian-planck
2017-10-10T10:47:53.435682+00:00 app[web.3]: [44bb3720-6e34-4951-8ab6-9304165172cc] [/26/teams/gj-la-haye-fouassier-u18-d2-masculin-groupe-d-93230] Completed 200 OK in 48ms (ActiveRecord: 12.8ms)
```

In particular:

- That `/Started GET "(.*?)" for/` can be used to parse the URLs.
- That `/\]: \[(.*?)\]/` can be used to parse the request ids.
- That `/current_user\.slug => (.*)/` can be used to parse the slug of the user performing the request. These are the slugs that'll be given to `access_token_loader`.

Once you have this setup you can run the requests from your own machine or in the cloud if you need even more load.
