use std::str;
use std::sync::Arc;
use std::{fmt::Debug, io::{Read, Write}, net::TcpStream};
use dashmap::DashMap;

use crate::http::response::{PotentialResponse, Response};

use super::cookie::{Cookie, CookieJar};
use super::logger::{Logger, Logs};

#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match *self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
        }
    }
}

pub type Context = DashMap<String, String>;
pub type Headers = DashMap<String, String>;
pub type Params = DashMap<String, String>;

pub trait Contextable: Send + Sync + 'static {
    fn key(&self) -> &'static str;
} 

pub type RequestBuffer = [u8; 1024];

#[derive(Debug, Clone)]
pub struct Request {
    pub host: String,
    pub method_and_path: String,
    pub method: HttpMethod,
    pub path: String,
    pub protocol: String,
    pub body: String,
    pub headers: Headers,
    pub params: Params,
    pub context: Context,
    pub cookies: CookieJar,
}

impl Request {
    pub fn new(host: &String) -> Self {
        let request = Self {
            host: host.to_string(),
            method_and_path: "".to_string(),
            method: HttpMethod::GET,
            path: "".to_string(),
            protocol: "HTTP/1.1".to_string(),
            body: "".to_string(),
            headers: DashMap::new(),
            params: DashMap::new(),
            context: DashMap::new(),
            cookies: CookieJar::new(),
        };
        return request;
    }

    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }

    pub fn get_header(&self, key: &str) -> String {
        match self.headers.get(key) {
            Some(value) => {
                return value.to_string();
            },
            None => {
                return "".to_string();
            },
        }
    }

    pub fn header(self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

	pub fn get_context<K: Contextable>(&self, key: K) -> String {
        match self.context.get(key.key()) {
            Some(value) => {
                return value.to_string();
            },
            None => {
                return "".to_string();
            },
        }
    }

    pub fn set_context<K: Contextable>(&mut self, key: K, value: String) {
        self.context.insert(key.key().to_string(), value.to_string());
    }

    pub fn get_cookie(&self, key: &str) -> String {
        let cookie = self.cookies.get(key);
        match cookie {
            Some(cookie) => {
                return cookie.value.to_string();
            },
            None => {
                return "".to_string();
            },
        }
    }

    pub fn get_url(&self) -> String {
        self.host.clone() + &self.path
    }

    pub fn get_host(&self) -> String {
        self.host.clone()
    }

    pub fn raw(&self) -> String {
        let mut headers_str = String::new();
        for header in self.headers.iter() {
            headers_str.push_str(&format!("{}: {}\r\n", header.key(), header.value()));
        }
        let request = format!(
            "{} {} {}\r\n{}\r\n{}",
            self.method.as_str(),
            self.path,
            self.protocol,
            headers_str,
            self.body
        );
        return request;
    }

    pub fn send_raw(&self, raw_request: &String) -> Response {
        let stream = TcpStream::connect(&self.get_host());
        match stream {
            Ok(mut stream) => {
                match stream.write_all(raw_request.as_bytes()) {
                    Ok(_) => {
                        let mut response_bytes = Vec::new();
                        match stream.read_to_end(&mut response_bytes) {
                            Ok(_) => {
                                let response = Response::new_from_bytes(&response_bytes);
								return response;
                            },
							// TODO: what would cause this error to occur?
                            Err(e) => {
                                return Response::new()
									.status(500)
									.body(&format!("failed to read response: {:?}", e));
                            },
                        }

                    },
					// TODO: what would cause this error to occur?
                    Err(e) => {
						return Response::new()
							.status(500)
							.body(&format!("failed to read response: {:?}", e));
                    },
                }
            },
			// TODO: what would cause this error to occur?
            Err(e) => {
				return Response::new()
					.status(500)
					.body(&format!("failed to read response: {:?}", e));
            },
        }
    }

    pub fn send(&self) -> Response {
        let stream = TcpStream::connect(&self.get_host());
        match stream {
            Ok(mut stream) => {
                let request = self.raw();
                match stream.write_all(request.as_bytes()) {
                    Ok(_bytes_wrote) => {
                        let mut response_bytes = Vec::new();
                        match stream.read_to_end(&mut response_bytes) {
                            Ok(_unknown) => {
                                let response = Response::new_from_bytes(&response_bytes);
                                return response;
                            },
                            // TODO: figure out what triggers this error
                            Err(err) => {
                                let response = Response::new()
                                    .status(500)
                                    .body(&err.to_string());
                                return response
                            },
                        }
                    },
                    // this error occurs if the request payload is too large
                    Err(err) => {
                        let response = Response::new()
                        .status(500)
                        .body(&err.to_string());
                        return response
                    },
                }
            },
            // TODO: figure out what triggers this error
            Err(err) => {
                let response = Response::new()
                .status(500)
                .body(&err.to_string());
                return response
            },
        }
    }
    pub fn new_from_bytes(request_bytes: RequestBuffer) -> (Request, PotentialResponse) {
        let (request, potential_response) = Request::parse_request_bytes(request_bytes);
        return (request, potential_response);
    }
    pub fn parse_request_bytes(request_bytes: RequestBuffer) -> (Request, PotentialResponse) {
        let mut request = Request{
            method_and_path: "".to_string(),
            method: HttpMethod::GET,
            path: "".to_string(),
            protocol: "".to_string(),
            body: "".to_string(),
            host: "".to_string(),
            headers: DashMap::new(),
            context: DashMap::new(),
            params: DashMap::new(),
            cookies: CookieJar::new(),
        };
		// TODO: investagate this line
        let end = request_bytes.iter().position(|&x| x == 0).unwrap_or(request_bytes.len());
        let request_string = String::from_utf8(request_bytes[..end].to_vec());
		if let Err(request_string) = request_string {
			let err = "failed to parse request";
			return (request, Some(Response::new()
				.status(400)
				.body(err)
			));
		}
		let request_string = request_string.unwrap();
		let lines: Vec<&str> = request_string.lines().collect();
		for i in 0..lines.len() {
			let line = lines[i];
			// FIRST LINE
			// method, path, protocol
			if i == 0 {
				// COLLECTING PARTS OF FIRST LINE
				let parts = line.split(" ").collect::<Vec<&str>>();
				if parts.len() != 3 {
					let err = "malformed request: status line did not have exactly three parts";
					let res = Response::new()
						.status(400)
						.body(err);
					return (request, Some(res));
				}
				let method = parts[0];
				let path = parts[1];
				// EXTRACTING QUERY PARAMS FROM PATH
				let mut param_string = String::new();
				let params = path.split("?").collect::<Vec<&str>>();
				if params.len() > 1 {
					request.path = params[0].to_string();
					request.method_and_path = format!("{} {}", method, params[0]);
					param_string = params[1].to_string();
				} else {
					request.path = path.to_string();
					request.method_and_path = format!("{} {}", method, path);
				}
				let params = param_string.split("&").collect::<Vec<&str>>();
				for param in params {
					let parts = param.split("=").collect::<Vec<&str>>();
					if parts.len() != 2 {
						continue
					}
					let key = parts[0];
					let value = parts[1];
					request.params.insert(key.to_string(), value.to_string());
				}
				// EXTRACTING PROTOCOL
				// TODO: figure out how to handle other protocols
				let protocol = parts[2];
				match protocol {
					"HTTP/1.1" => {},
					_ => {
						return (request, Some(Response::new()
							.status(400)
							.body("protocol is missing or invalid: only HTTP/1.1 is supported")
						));
					},
				}
				// ENSURING THE METHOD IS VALID
				match method {
					"GET" => {
						request.method = HttpMethod::GET;
					},
					"POST" => {
						request.method = HttpMethod::POST;
					},
					"PUT" => {
						request.method = HttpMethod::PUT;
					},
					"DELETE" => {
						request.method = HttpMethod::DELETE;
					},
					"PATCH" => {
						request.method = HttpMethod::PATCH;
					},
					_ => {
						return (request, Some(Response::new()
							.status(400)
							.body("malformed request: method was extracted but found to be invalid")
						));
					},
				}
				request.protocol = protocol.to_string();
				continue
			}
			// LAST LINE
			// request body
			if i == lines.len() - 1 {
				request.body = line.to_string();
				continue
			}
			// EMPTY LINES
			if line.len() == 0 {
				continue
			}
			// HEADERS
			// ANY LINE THAT IS NOT THE FIRST OR LAST IS A HEADER
			if line.contains(":") == false {
				return (request, Some(Response::new()
					.status(400)
					.body("malformed request: header line did not contain a colon")
				));
			}
			let parts = line.split(":").collect::<Vec<&str>>();
			if parts.len() != 2 {
				continue
			}
			let key = parts[0];
			let value = parts[1];
            // ANY HEADER THAT IS NOT A COOKIE
			if key != "Cookie" {
				request.headers.insert(key.to_string(), value.to_string());
				continue
			}
            // COOKIES
			let cookies = value.split("; ").collect::<Vec<&str>>();
            for cookie in cookies {
                let parts = cookie.split("=").collect::<Vec<&str>>();
                if parts.len() != 2 {
                    continue
                }
                let key = parts[0];
                let value = parts[1];
                request.cookies.add(Cookie::new(key, value));
            }
		}
		return (request, None);
    }

}

