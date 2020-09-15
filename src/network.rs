use std::net::TcpStream;
use crate::response::{Response, make_error, code_to_str, to_string};
use std::io::{BufReader, Write};
use crate::request_parser::get_message;
use crate::request_handler::handle_request;
use crate::response::StatusCode::_500;

fn handle_client_2(stream: &mut TcpStream) -> Result<Response, Response> {
    let cloned_stream = stream.try_clone()?;
    let buf_stream = BufReader::new(cloned_stream);
    let rq = get_message(buf_stream)?;
    Ok(handle_request(rq)?)
}

pub fn handle_client(mut stream: TcpStream) {
    let response = extract_response(handle_client_2(&mut stream));
    let _ = stream.write(to_string(response).as_bytes());
}

fn extract_response<T>(thing: Result<T, T>) -> T {
    match thing {
        Ok(x) => x,
        Err(x) => x,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn take_works_how_i_think_it_does() {
        let mut reader = BufReader::new("hi this is some text wow isn't it cool".as_bytes()).take(7);

        let mut string = String::new();
        reader.read_to_string(&mut string).unwrap();
        assert_eq!(string, "hi this".to_string());

        reader.set_limit(3);
        let mut string = String::new();
        reader.read_to_string(&mut string).unwrap();
        assert_eq!(string, " is".to_string());

        let mut reader = reader.into_inner();
        let mut string = String::new();
        reader.read_to_string(&mut string).unwrap();
        assert_eq!(string, " some text wow isn't it cool".to_string());


    }

}