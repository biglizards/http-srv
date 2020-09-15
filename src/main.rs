use std::io::{Take, Empty};
use std::io::prelude::*;
use std::net::TcpListener;
use std::thread;

use derivative::*;

use crate::network::handle_client;

mod response;
mod request_handler;
mod request_parser;
mod network;


static DEFAULT_BODY_LIMIT: u64 = 1048576;  // 1MiB

#[derive(Debug)]
struct Body<T: BufRead> {
    ready: bool,
    content: Vec<u8>,
    buffer: Option<Take<T>>
}

// if anyone actually attempts to compare bodies, we'll probably panic,
// but this is only used in tests so it's fine
impl<T: BufRead> std::cmp::PartialEq for Body<T> {
    fn eq(&self, other: &Self) -> bool {
        if !(self.ready && other.ready) {
            panic!("bodies not yet ready! call `body.get()` before comparision!")
        };
        self.content.eq(&other.content)
    }

    fn ne(&self, other: &Self) -> bool {
        if !(self.ready && other.ready) {
            panic!("bodies not yet ready! call `body.get()` before comparision!")
        };
        self.content.ne(&other.content)
    }
}
impl<T: BufRead> std::cmp::Eq for Body<T> {}

impl<T: BufRead> Body<T> {
    fn new(buffer: T) -> Body<T> {
        Body {
            ready: false,
            content: vec![],
            buffer: Some(buffer.take(DEFAULT_BODY_LIMIT))
        }
    }

    fn get(&mut self) -> &Vec<u8> {
        if self.ready == true {
            return &self.content
        }
        if let Some(buffer) = &mut self.buffer {
            buffer.read_to_end(&mut self.content);
            self.buffer = None;
            self.ready = true;
            &self.content
        } else {
            self.ready = true;
            &self.content
        }
    }
}
fn empty_body() -> Body<Empty> {
    Body {
        ready: true,
        content: vec![],
        buffer: None
    }
}


#[derive(Derivative)]
#[derivative(PartialEq(bound=""))]
#[derive(Debug, Eq)]
struct Request<T: BufRead> {
    method: Method,
    route: String,
    version: String,
    headers: Vec<Header>,
    body: Option<Body<T>>,
}

#[derive(Debug, Eq, PartialEq)]
enum Method {
    GET,
    POST
}

#[derive(Debug, Eq, PartialEq)]
struct Header {
    name: String,
    value: String
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            thread::spawn(
                || {
                    handle_client(stream);
                }
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // todo insert integration tests here
}