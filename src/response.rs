
#[derive(Debug, Clone, std::cmp::PartialEq)]
pub(crate) enum StatusCode {
    // Informational responses
    // _100

    _200,  // OK
    // _201  // Created
    // _202  // Accepted
    // _203  // Non-Authoritative Information
    // _204  // No Content
    // _205  // Reset Content
    // _206  // Partial Content

    // Redirects
    // _300,

    // Client errors
    _400,
    _404,
    _411,
    _431,

    // Server errors
    _500,
}

pub(crate) fn code_to_str(code: &StatusCode) -> &'static str {
    match code {
        StatusCode::_200 => "200 - OK",
        StatusCode::_400 => "400 - Bad Request",
        StatusCode::_404 => "404 - Not Found",
        StatusCode::_411 => "411 - Length Required",
        StatusCode::_431 => "431 - Request Header Fields Too Large",
        StatusCode::_500 => "500 - Internal Server Error",
    }
}

pub(crate) fn code_to_html(code: &StatusCode) -> String {
    message_to_html(code_to_str(&code))
}

pub(crate) fn message_to_html(message: &str) -> String {
    format!("<!DOCTYPE html>\
    <html lang=\"en\">\
    <head>\
        <meta charset=\"UTF-8\">\
        <title>{}</title>\
    </head>\
    <body>\
        <h1>{}</h1>\
    </body>\
    </html>", message, message)
}


#[derive(Clone, Debug)]
pub(crate) struct Response {
    pub code: StatusCode,
    pub headers: Vec<String>,
    pub body: String,
}

// impl From<&str> for Response {
//     fn from(item: &str) -> Self {
//         Response {
//             code: StatusCode::_500,
//             headers: vec![],
//             body: item.to_string() + "\r\n"
//         }
//     }
// }

impl From<StatusCode> for Response {
    fn from(item: StatusCode) -> Self {
        Response {
            body: code_to_html(&item) + "\r\n",
            code: item,
            headers: vec![],
        }
    }
}


impl<T: std::error::Error> From<T> for Response {
    fn from(_item: T) -> Self {
        let code = StatusCode::_500;
        Response {
            body: code_to_html(&code),
            code,
            headers: vec![],
        }
    }
}

pub(crate) fn internal_server_error() -> Response {
    make_error(StatusCode::_500)
}

pub(crate) fn make_error(code: StatusCode) -> Response {
    Response {
        body: code_to_html(&code),
        code,
        headers: vec![],  // todo maybe add some headers
    }
}

pub(crate) fn to_string(response: Response) -> String {
    // todo add headers like content-length
    format!("HTTP/1.1 {}\r\n{}\r\n\r\n{}\r\n",
            code_to_str(&response.code),
            "",
            response.body)
}
