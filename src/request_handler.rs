use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::{Header, Method, Request};
use crate::response::{internal_server_error, Response};
use crate::response::StatusCode::*;
use std::net::TcpStream;

pub(crate) fn handle_request<R: BufRead>(request: Request<R>) -> Result<Response, Response> {
    match request.method {
        Method::GET => handle_get_request(request),
        Method::POST => handle_post_request(request),
    }
}

fn handle_get_request<R: BufRead>(request: Request<R>) -> Result<Response, Response> {
    // let path = route_to_path(request.route)?;
    // let thing = fs::read_to_string(path)?;
    return Ok(Response {
        code: _200,
        headers: vec![],
        body: "".into()
    });
}

fn handle_post_request<R: BufRead>(mut request: Request<R>) -> Result<Response, Response> {
    // presumably to be able to post something, you'd need to be able to pass the data
    //   along to a scripting language for the relevant route/filetype.
    // for now, we'll just say "what that doesn't make sense" and pretend they
    //   wanted to GET the page, since that seems to be standard.

    // find the header that says how long it is, so we know when to stop reading
    let content_lengths: Vec<&Header> = request.headers.iter().filter(|item| item.name == "Content-Length").collect();
    match content_lengths.len() {
        0 => return Err(_411.into()),
        1 => {},
        _ => return Err(_400.into())
    }
    let content_length = content_lengths.first().unwrap().value.parse()?;
    request.body.as_mut().unwrap().buffer.as_mut().unwrap().set_limit(content_length);

    let raw_content = request.body.as_mut().unwrap().get().clone();
    // let str_content = String::from_utf8(raw_content)?;
    let content: Value = serde_json::from_slice(&*raw_content)?;
    let goal = &content["goal"];

    let actual_content = format!(
        "<@135483608491229184>\n{}\n{}",
        goal["graph_url"].to_string().trim_matches('"'),
        goal["headsum"].to_string().trim_matches('"')
    );

    let payload: Value = json!({
        "content": actual_content,
        "username": "beeminder bot",
    });

    let client = reqwest::blocking::Client::new();
    let res = client.post("https://discordapp.com/api/webhooks/741353378365440101/TiIK8F-ZihU0PA65bwU9OUa_9jdFZKE6AFbFseQlJq3liCQxQ1ZOlVG_hU5sP72bN0FE")
        .body(payload.to_string())
        .header("Content-Type", "application/json")
        .send()?;

    println!("{:?}", res);
    println!("{:?}", payload.to_string());

    Ok(_200.into())
}

fn route_to_path(route: String) -> Result<Box<Path>, Response> {
    // todo maybe some no breaking out of the sandbox checks
    // also make this function less completely awful
    let root_path = PathBuf::from("root").canonicalize()?;

    // step 1: make sure the path starts with the root dir
    let mut path1 = PathBuf::from(route);
    if path1.has_root() {
        path1 = path1.strip_prefix("/").unwrap_or(&*path1).to_owned()
    }
    let mut path: PathBuf = [root_path.clone(), path1].iter().collect();

    // step 2: if we're talking about a folder, we mean the index.html in that folder
    if None == path.file_stem() || path.is_dir() {
        path.push("index.html")
    };

    // step 3: if it doesn't exist, 404
    if !path.exists() {
        return Err(_404.into())
    }

    // step 4: if they cheated and got outside of the root dir, don't let them
    if !path.canonicalize()?.starts_with(root_path) {
        return Err(_404.into())
    }

    Ok(Box::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

//

    #[test]
    fn file_select_works() {
        let routes = vec!["/", "", "/index.html", "/foo.html", "/foo/bar/baz.html",
                          "/foo/bar/"];
        let answers = vec!["root/index.html", "root/index.html", "root/index.html", "root/foo.html", "root/foo/bar/baz.html",
                           "root/foo/bar/index.html"];
        let paths: Vec<Box<Path>> = answers.iter().map(|path_str| {
            Box::from(Path::new(path_str).canonicalize().unwrap())
        }).collect();

        let processed_paths: Vec<Box<Path>> = routes.iter().map(|x| route_to_path(String::from(*x)).unwrap()).collect();
        assert_eq!(
            processed_paths,
            paths
        );

        // also, check at no point does anything escape ./root
        let base_path = Path::new("root").canonicalize().unwrap();

        for path in processed_paths {
            assert!(path
                .canonicalize().unwrap()
                .ancestors().any(|x| x==base_path)
            );
        }
    }

    #[test]
    fn file_select_404s() {
        let routes = vec!["/foo/../../../../../../../../etc/passwd", "invalidpath"];

        let processed_paths: bool = routes.iter().map(|x| {
            route_to_path(String::from(*x)).is_err()
        }).all(
            |x| x
        );

        assert!(processed_paths)


    }

}