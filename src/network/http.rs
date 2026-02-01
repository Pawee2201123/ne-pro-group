// network/http.rs - Simple HTTP request parsing
//
// 🎓 Key Concepts:
// - HTTP is a text protocol (not binary!)
// - Requests are sent as strings over TCP
// - We parse the string to understand what the client wants

use std::collections::HashMap;

/// Represents an HTTP method
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    OPTIONS,  // For CORS preflight
}

/// Represents a parsed HTTP request
#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    /// Parse an HTTP request from raw bytes
    ///
    /// 🎓 HTTP Request Format:
    /// ```
    /// GET /path?key=value HTTP/1.1\r\n
    /// Header-Name: Header-Value\r\n
    /// Another-Header: Value\r\n
    /// \r\n
    /// Body content here
    /// ```
    pub fn parse(raw: &str) -> Result<Self, String> {
        // Split into lines
        let mut lines = raw.split("\r\n");

        // Parse the request line: "GET /path HTTP/1.1"
        let request_line = lines.next().ok_or("Empty request")?;
        let mut parts = request_line.split_whitespace();

        let method_str = parts.next().ok_or("No method")?;
        let full_path = parts.next().ok_or("No path")?;

        // Parse method
        let method = match method_str {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "OPTIONS" => Method::OPTIONS,
            _ => return Err(format!("Unknown method: {}", method_str)),
        };

        // Split path from query string
        let (path, query_params) = Self::parse_path_and_query(full_path);

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                // Empty line marks end of headers
                break;
            }

            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_lowercase(), value.to_string());
            }
        }

        // Remaining is the body
        let body = lines.collect::<Vec<_>>().join("\r\n");

        Ok(HttpRequest {
            method,
            path,
            query_params,
            headers,
            body,
        })
    }

    /// Parse path and query parameters
    /// Example: "/room/join?room_id=123" → ("/room/join", {"room_id": "123"})
    fn parse_path_and_query(full_path: &str) -> (String, HashMap<String, String>) {
        let mut params = HashMap::new();

        let (path, query) = if let Some((p, q)) = full_path.split_once('?') {
            (p, Some(q))
        } else {
            (full_path, None)
        };

        if let Some(query_str) = query {
            for pair in query_str.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }

        (path.to_string(), params)
    }

    /// Get a query parameter
    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    /// Get a header
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }
}

/// Build HTTP responses
pub struct HttpResponse;

impl HttpResponse {
    /// Build a simple HTTP response
    pub fn build(status: &str, content_type: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
            status,
            content_type,
            body.len(),
            body
        )
    }

    /// 200 OK response
    pub fn ok(body: &str, content_type: &str) -> String {
        Self::build("200 OK", content_type, body)
    }

    /// 404 Not Found
    pub fn not_found() -> String {
        Self::build("404 Not Found", "text/plain", "Not Found")
    }

    /// 400 Bad Request
    pub fn bad_request(message: &str) -> String {
        Self::build("400 Bad Request", "text/plain", message)
    }

    /// 500 Internal Server Error
    pub fn server_error(message: &str) -> String {
        Self::build("500 Internal Server Error", "text/plain", message)
    }

    /// SSE connection response (keeps connection open)
    pub fn sse_header() -> String {
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream\r\n\
         Cache-Control: no-cache\r\n\
         Connection: keep-alive\r\n\
         Access-Control-Allow-Origin: *\r\n\r\n"
            .to_string()
    }

    /// CORS preflight response
    pub fn cors_preflight() -> String {
        "HTTP/1.1 200 OK\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Content-Length: 0\r\n\r\n"
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let raw = "GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();

        assert_eq!(req.method, Method::GET);
        assert_eq!(req.path, "/test");
    }

    #[test]
    fn test_parse_with_query() {
        let raw = "GET /events?room_id=123&player=alice HTTP/1.1\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();

        assert_eq!(req.path, "/events");
        assert_eq!(req.query("room_id"), Some(&"123".to_string()));
        assert_eq!(req.query("player"), Some(&"alice".to_string()));
    }

    #[test]
    fn test_parse_post_with_body() {
        let raw = "POST /room/create HTTP/1.1\r\nContent-Type: text/plain\r\n\r\nHello World";
        let req = HttpRequest::parse(raw).unwrap();

        assert_eq!(req.method, Method::POST);
        assert_eq!(req.path, "/room/create");
        assert_eq!(req.body, "Hello World");
    }

    #[test]
    fn test_response_ok() {
        let response = HttpResponse::ok("Test", "text/plain");
        assert!(response.contains("200 OK"));
        assert!(response.contains("Test"));
    }
}
