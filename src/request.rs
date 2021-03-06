use regex::Regex;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Request {
    pub url: String,
    pub id: String,
    pub user_slug: String,
    pub access_token: String,
}

pub trait AccessTokenLoader {
    fn access_token_from_user_slug(&mut self, user_slug: &String) -> Option<String>;
}

impl Request {
    pub fn from_logs_file<T: AccessTokenLoader>(
        logs: &String,
        access_token_loader: T,
    ) -> Vec<Request> {
        parse_requests(logs, access_token_loader)
    }
}

lazy_static! {
    static ref REQUEST_LINE_PATTERN: Regex = Regex::new(r#"Started GET "(.*?)" for"#).unwrap();
    static ref REQUEST_ID_PATTERN: Regex = Regex::new(r#"\]: \[(.*?)\] "#).unwrap();
    static ref USER_SLUG_PATTERN: Regex = Regex::new(r#"current_user\.slug => (.*)"#).unwrap();
}

fn get_request_lines(logs: &String) -> Vec<String> {
    logs.lines()
        .map(|line| line.to_string())
        .filter(|line| REQUEST_LINE_PATTERN.is_match(line))
        .collect()
}

fn parse_request_ids_and_user_slugs(logs: &String) -> HashMap<String, String> {
    logs.lines()
        .map(|line| line.to_string())
        .fold(HashMap::new(), |mut acc, line| {
            match USER_SLUG_PATTERN.captures(&line) {
                None => acc,
                Some(slug_captures) => {
                    let slug = slug_captures
                        .get(1)
                        .expect("Failed getting user slug capture")
                        .as_str()
                        .to_string();

                    let request_id = REQUEST_ID_PATTERN
                        .captures(&line)
                        .expect("Didn't match REQUEST_ID_PATTERN")
                        .get(1)
                        .expect("Failed getting request id capture")
                        .as_str()
                        .to_string();

                    acc.insert(request_id, slug);
                    acc
                }
            }
        })
}

fn parse_requests<T: AccessTokenLoader>(logs: &String, mut access_token_loader: T) -> Vec<Request> {
    let request_ids_to_user_slugs = parse_request_ids_and_user_slugs(logs);

    get_request_lines(logs)
        .iter()
        .fold(Vec::new(), |mut acc, line| {
            let id = request_id_from_request_line(&line);
            let user_slug = user_slug_for_request_id(&id, &request_ids_to_user_slugs);
            let access_token = user_slug
                .clone()
                .and_then(|user_slug| access_token_loader.access_token_from_user_slug(&user_slug));

            match (user_slug, access_token) {
                (Some(user_slug), Some(access_token)) => {
                    let req = Request {
                        url: url_from_request_line(&line),
                        id: id,
                        user_slug: user_slug,
                        access_token: access_token,
                    };
                    acc.push(req);
                }
                _ => (),
            }

            acc
        })
}

fn user_slug_for_request_id(
    id: &String,
    request_ids_to_user_slugs: &HashMap<String, String>,
) -> Option<String> {
    request_ids_to_user_slugs.get(id).map(|slug| slug.clone())
}

fn request_id_from_request_line(line: &str) -> String {
    REQUEST_ID_PATTERN
        .captures(line)
        .expect("REQUEST_ID_PATTERN didn't match")
        .get(1)
        .expect("No first match")
        .as_str()
        .to_string()
}

fn url_from_request_line(line: &str) -> String {
    REQUEST_LINE_PATTERN
        .captures(line)
        .expect("REQUEST_LINE_PATTERN didn't match")
        .get(1)
        .expect("No first match")
        .as_str()
        .to_string()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_finds_the_lines_that_are_get_requests() {
//         let line: String = "2017-10-10T10:47:53.385562+00:00 app[web.3]: [44bb3720-6e34-4951-8ab6-9304165172cc] [/26/teams/gj-la-haye-fouassier-u18-d2-masculin-groupe-d-93230] Started GET \"/26/teams/gj-la-haye-fouassier-u18-d2-masculin-groupe-d-93230\" for 80.215.91.225 at 2017-10-10 10:47:53 +0000".to_string();

//         let parsed_lines = get_request_lines(&"test_fixtures/logs".to_string());

//         assert_eq!(parsed_lines.first(), Some(&line));
//         assert_eq!(parsed_lines.len(), 89);
//     }

//     #[test]
//     fn it_parses_the_request_lines() {
//         let requests = parse_requests(&"test_fixtures/logs".to_string());

//         assert_eq!(requests.len(), 82);

//         let request = requests.first().unwrap();
//         assert_eq!(
//             request.url,
//             "/26/teams/gj-la-haye-fouassier-u18-d2-masculin-groupe-d-93230".to_string()
//             );
//         assert_eq!(
//             request.id,
//             "44bb3720-6e34-4951-8ab6-9304165172cc".to_string()
//             );
//         assert_eq!(
//             request.user_slug,
//             "matthias-lombard".to_string()
//             );
//     }

//     #[test]
//     fn it_parses_request_ids_and_user_slugs() {
//         let ids_and_slugs = parse_request_ids_and_user_slugs(&"test_fixtures/logs".to_string());

//         assert_eq!(
//             ids_and_slugs.get("44bb3720-6e34-4951-8ab6-9304165172cc"),
//             Some(&"matthias-lombard".to_string())
//             );
//         assert_eq!(ids_and_slugs.keys().len(), 117);
//     }
// }
