use std::io::BufRead;

use crate::{Body, Header, Method, Request};
use crate::response::{Response, StatusCode};
use crate::response::StatusCode::*;

pub(crate) fn get_message<Q: BufRead>(stream: Q) -> Result<Request<Q>, Response> {
    let mut lines = vec![];
    let mut line = String::new();

    // dont let the stream read any more than 8192 characters
    let mut limited_stream = stream.take(8192);
    let mut total_read = 0;

    loop {
        // todo if they just dont send anything we'll never time them out
        if let Ok(len) = limited_stream.read_line(&mut line) {
            if len == 0 {
                // the request ended before a CRLF CRLF, which is not allowed
                // (as far as i'm aware, all requests MUST contain CRLF CRLF, in between the
                //  header and body sections)
                return if total_read == 8192 {Err(_431.into())} else {Err(_400.into())}
            } else if line == String::from("\r\n") {
                // we've reached the body boundary, and it may contain raw data, so stop trying to
                // parse it
                break
            }
            lines.push(line);
            line = String::new();
            total_read += len;
        } else {
            // the client sent non-utf8 data pre-body, which is just not cool
            // iirc non-ascii isn't allowed, so we're being lenient here
            return Err(_400.into())
        }
    }

    let body = Body::new(limited_stream.into_inner());
    parse_request(lines, body)
}

fn parse_request<R: BufRead>(lines: Vec<String>, body: Body<R>) -> Result<Request<R>, Response> {
    // basically, the first line should be something like
    // GET /some/path.html HTML/1.1
    let mut lines_iter = lines.iter();

    let first_line = lines_iter.next().ok_or(_400)?;
    let segments: Vec<&str> = first_line.split(' ').collect();
    let method = *segments.get(0).ok_or(_400)?;
    let route = *segments.get(1).ok_or(_400)?;
    let version = *segments.get(2).ok_or(_400)?;

    // then the headers, which are (hopefully, i've not read all the cases in the spec)
    // field-name ":" [ field-value ]
    // it would be very rude if field-value was allowed to contain newline chars or ':'
    // so i'll just assume it doesn't
    let mut headers = vec![];
    for line in lines_iter {
        if line == "\r\n" {break}
        let (name, value) = split_once(line)?;
        headers.push(Header { name: name.to_string(), value: value.trim().to_string() })
    }

    match method {
        "GET" => {
            Ok(Request {
                method: Method::GET,
                route: route.to_string(),
                version: version.trim().to_string(),
                headers,
                body: None
            })
        },

        "POST" => {
            Ok(Request {
                method: Method::POST,
                route: route.to_string(),
                version: version.trim().to_string(),
                headers,
                body: Some(body)  // one told me
            })

        }

        _ => {
            return Err(_500.into())
        }
    }



}

fn split_once(in_string: &str) -> Result<(&str, &str), StatusCode> {
    let mut splitter = in_string.splitn(2, ':');
    let first = splitter.next().ok_or(_400)?;
    let second = splitter.next().ok_or(_400)?;

    Ok((first, second))
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Empty};

    use crate::empty_body;

    use super::*;



    #[test]
    fn test_parser() {
        let ans: Request<Empty> = Request {
            method: Method::GET,
            route: String::from("/"),
            version: String::from("HTTP/1.1"),
            headers: vec![],
            body: None
        };

        assert_eq!(
            parse_request(
                vec![String::from("GET / HTTP/1.1")],
                empty_body()
            ).unwrap(),
            ans
        )
    }

    #[test]
    fn test_get_message_with_big_request() {
        let big_request = "GET / HTTP/1.1\r\n\
        Host: localhost:8080\r\n\
        User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r\n\
        Accept-Language: en-GB,en;q=0.5\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Connection: keep-alive\r\n\
        Cookie: org.cups.sid=6a308215f6018e9104005ccdb4ea73c5\r\n\
        Upgrade-Insecure-Requests: 1\r\n\r\n";

        let br = BufReader::new(big_request.as_bytes());

        assert_eq!(
            get_message(br).unwrap(),
            Request {
                method: Method::GET,
                route: String::from("/"),
                version: String::from("HTTP/1.1"),
                headers: vec![Header { name: String::from("Host"), value: String::from("localhost:8080") },
                              Header { name: String::from("User-Agent"), value: String::from("Mozilla/5.0 (X11; Linux x86_64; rv:78.0) Gecko/20100101 Firefox/78.0") },
                              Header { name: String::from("Accept"), value: String::from("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8") },
                              Header { name: String::from("Accept-Language"), value: String::from("en-GB,en;q=0.5") },
                              Header { name: String::from("Accept-Encoding"), value: String::from("gzip, deflate") },
                              Header { name: String::from("Connection"), value: String::from("keep-alive") },
                              Header { name: String::from("Cookie"), value: String::from("org.cups.sid=6a308215f6018e9104005ccdb4ea73c5") },
                              Header { name: String::from("Upgrade-Insecure-Requests"), value: String::from("1") }],

                body: None
            }
        )
    }


    #[test]
    fn requests_with_long_headers_error() {
        let long_message = BufReader::new(&[b'A'; 9001][..]);
        let rv = get_message(long_message);
        assert!(rv.is_err());
        assert_eq!(rv.unwrap_err().code, StatusCode::_431)
    }

    #[test]
    fn requests_with_long_body_is_fine() {
        let mut base = "POST / HTTP/1.1\r\n\r\n".to_string();
        let body = "A".repeat(9001);
        base.push_str(&body);
        let long_message = BufReader::new(base.as_bytes());

        let rv = get_message(long_message);
        assert!(rv.is_ok());
    }

    #[test]
    fn request_with_invalid_utf8_body_is_fine() {
        let request_bytes = b"POST / HTTP/1.1\r\n\r\n\xc3\x28";

        let rv = get_message(BufReader::new(&request_bytes[..]));
        assert!(rv.is_ok());
    }

    #[test]
    fn request_with_invalid_utf8_headers_errors() {
        let request_bytes = b"POST / HTTP/1.1\r\n\xc3\x28\r\n\r\n";

        let rv = get_message(BufReader::new(&request_bytes[..]));
        assert!(rv.is_err());
        assert_eq!(rv.unwrap_err().code, StatusCode::_400)
    }

    #[test]
    fn invalid_methods_error() {
        let request_bytes = b"FHQWHGAD / HTTP/1.1\r\n\r\n";

        let rv = get_message(BufReader::new(&request_bytes[..]));
        assert!(rv.is_err());
        // really it should be a 400, but we dont have all the methods yet so it's 500 for now to be safe
        assert_eq!(rv.unwrap_err().code, StatusCode::_500)
    }

}