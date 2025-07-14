use crate::rpc::{JsonRpcResponse, RpcClient, RpcError};
use std::collections::HashMap;

const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10MB max response

// RPC request/response structures for HTTP
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpRpcRequest {
    pub url: String,
    pub options: HttpOptions,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum HttpBody {
    Text(String),
    Binary(Vec<u8>),
    Form(HashMap<String, String>),
    Multipart(Vec<MultipartField>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HttpOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<HttpBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_params: Option<HashMap<String, String>>,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            method: Some("GET".to_string()),
            headers: None,
            body: None,
            timeout: Some(30000), // 30 seconds default
            query_params: None,
        }
    }
}

impl HttpOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn method<S: Into<String>>(mut self, method: S) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }
        self.headers
            .as_mut()
            .unwrap()
            .insert(key.into(), value.into());
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn body<S: Into<String>>(mut self, body: S) -> Self {
        self.body = Some(HttpBody::Text(body.into()));
        self
    }

    pub fn body_binary(mut self, data: Vec<u8>) -> Self {
        self.body = Some(HttpBody::Binary(data));
        self
    }

    pub fn form(mut self, form_data: HashMap<String, String>) -> Self {
        self.body = Some(HttpBody::Form(form_data));
        self = self.header("Content-Type", "application/x-www-form-urlencoded");
        self
    }

    pub fn multipart(mut self, fields: Vec<MultipartField>) -> Self {
        self.body = Some(HttpBody::Multipart(fields));
        // Note: Content-Type with boundary will be set by the host function
        self
    }

    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }

    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Result<Self, HttpError> {
        let json_body = serde_json::to_string(data).map_err(|_| HttpError::SerializationError)?;
        self.body = Some(HttpBody::Text(json_body));
        self = self.header("Content-Type", "application/json");
        Ok(self)
    }

    pub fn basic_auth<U: Into<String>, P: Into<String>>(self, username: U, password: P) -> Self {
        let credentials = format!("{}:{}", username.into(), password.into());
        let encoded = base64::encode_config(credentials.as_bytes(), base64::STANDARD);
        self.header("Authorization", format!("Basic {}", encoded))
    }

    pub fn bearer_auth<T: Into<String>>(self, token: T) -> Self {
        self.header("Authorization", format!("Bearer {}", token.into()))
    }

    pub fn query_param<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        if self.query_params.is_none() {
            self.query_params = Some(HashMap::new());
        }
        self.query_params
            .as_mut()
            .unwrap()
            .insert(key.into(), value.into());
        self
    }

    pub fn query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params = Some(params);
        self
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MultipartValue {
    Text(String),
    Binary {
        data: Vec<u8>,
        filename: Option<String>,
        content_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MultipartField {
    pub name: String,
    pub value: MultipartValue,
}

impl MultipartField {
    pub fn text<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: MultipartValue::Text(value.into()),
        }
    }

    pub fn binary<N: Into<String>>(
        name: N,
        data: Vec<u8>,
        filename: Option<String>,
        content_type: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: MultipartValue::Binary {
                data,
                filename,
                content_type,
            },
        }
    }

    pub fn file<N: Into<String>, F: Into<String>>(
        name: N,
        data: Vec<u8>,
        filename: F,
        content_type: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            value: MultipartValue::Binary {
                data,
                filename: Some(filename.into()),
                content_type,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: String,
}

impl HttpResponse {
    pub fn text(&self) -> Result<String, HttpError> {
        String::from_utf8(self.body.clone()).map_err(|_| HttpError::Utf8Error)
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, HttpError> {
        let text = self.text()?;
        serde_json::from_str(&text).map_err(|_| HttpError::JsonParseError)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HttpResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HttpError {
    InvalidUrl,
    SerializationError,
    JsonParseError,
    Utf8Error,
    EmptyResponse,
    RequestFailed(String),
    NetworkError,
    Timeout,
    RpcError(RpcError),
    Unknown(u32),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::InvalidUrl => write!(f, "Invalid URL provided"),
            HttpError::SerializationError => write!(f, "Failed to serialize request data"),
            HttpError::JsonParseError => write!(f, "Failed to parse JSON response"),
            HttpError::Utf8Error => write!(f, "Invalid UTF-8 in response"),
            HttpError::EmptyResponse => write!(f, "Empty response received"),
            HttpError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            HttpError::NetworkError => write!(f, "Network error occurred"),
            HttpError::Timeout => write!(f, "Request timed out"),
            HttpError::RpcError(e) => write!(f, "RPC error: {}", e),
            HttpError::Unknown(code) => write!(f, "Unknown error (code: {})", code),
        }
    }
}

impl From<RpcError> for HttpError {
    fn from(e: RpcError) -> Self {
        HttpError::RpcError(e)
    }
}

impl std::error::Error for HttpError {}

pub struct HttpClient {
    default_headers: Option<HashMap<String, String>>,
    timeout: Option<u32>,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            default_headers: self.default_headers.clone(),
            timeout: self.timeout,
        }
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            default_headers: None,
            timeout: Some(30000), // 30 seconds default
        }
    }

    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    // HTTP verb methods - return RequestBuilder for chaining
    pub fn get<U: Into<String>>(&self, url: U) -> RequestBuilder {
        self.request("GET", url)
    }

    pub fn post<U: Into<String>>(&self, url: U) -> RequestBuilder {
        self.request("POST", url)
    }

    pub fn put<U: Into<String>>(&self, url: U) -> RequestBuilder {
        self.request("PUT", url)
    }

    pub fn patch<U: Into<String>>(&self, url: U) -> RequestBuilder {
        self.request("PATCH", url)
    }

    pub fn delete<U: Into<String>>(&self, url: U) -> RequestBuilder {
        self.request("DELETE", url)
    }

    pub fn head<U: Into<String>>(&self, url: U) -> RequestBuilder {
        self.request("HEAD", url)
    }

    pub fn request<U: Into<String>>(&self, method: &str, url: U) -> RequestBuilder {
        let mut headers = HashMap::new();
        if let Some(ref default_headers) = self.default_headers {
            headers.extend(default_headers.clone());
        }

        RequestBuilder {
            client: self.clone(),
            method: method.to_string(),
            url: url.into(),
            headers,
            query_params: HashMap::new(),
            body: None,
            timeout: self.timeout,
        }
    }

    fn execute(&self, builder: &RequestBuilder) -> Result<HttpResponse, HttpError> {
        let options = HttpOptions {
            method: Some(builder.method.clone()),
            headers: if builder.headers.is_empty() {
                None
            } else {
                Some(builder.headers.clone())
            },
            body: builder.body.clone(),
            timeout: builder.timeout,
            query_params: if builder.query_params.is_empty() {
                None
            } else {
                Some(builder.query_params.clone())
            },
        };

        self.make_request(&builder.url, options)
    }

    fn make_request(&self, url: &str, options: HttpOptions) -> Result<HttpResponse, HttpError> {
        if url.is_empty() {
            return Err(HttpError::InvalidUrl);
        }

        // Build final URL with query parameters
        let final_url = if let Some(ref params) = options.query_params {
            build_url_with_params(url, params)
        } else {
            url.to_string()
        };

        let request = HttpRpcRequest {
            url: final_url,
            options,
        };
        let mut rpc_client = RpcClient::with_buffer_size(MAX_RESPONSE_SIZE);
        let response: JsonRpcResponse<HttpResult> =
            rpc_client.call("http.request", Some(request))?;

        if let Some(error) = response.error {
            return Err(HttpError::RequestFailed(format!(
                "RPC error: {} (code: {})",
                error.message, error.code
            )));
        }
        let http_result = response.result.ok_or(HttpError::EmptyResponse)?;

        if !http_result.success {
            let error_msg = http_result
                .error
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(HttpError::RequestFailed(error_msg));
        }

        http_result.data.ok_or(HttpError::EmptyResponse)
    }
}

pub struct HttpClientBuilder {
    default_headers: Option<HashMap<String, String>>,
    timeout: Option<u32>,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            default_headers: None,
            timeout: Some(30000),
        }
    }

    pub fn default_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.default_headers = Some(headers);
        self
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> HttpClient {
        HttpClient {
            default_headers: self.default_headers,
            timeout: self.timeout,
        }
    }
}
pub struct RequestBuilder {
    client: HttpClient,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
    body: Option<HttpBody>,
    timeout: Option<u32>,
}

impl RequestBuilder {
    pub fn header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    pub fn query<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    pub fn query_params(mut self, params: HashMap<String, String>) -> Self {
        self.query_params.extend(params);
        self
    }

    pub fn basic_auth<U: Into<String>, P: Into<String>>(
        mut self,
        username: U,
        password: P,
    ) -> Self {
        let credentials = format!("{}:{}", username.into(), password.into());
        let encoded = base64::encode_config(credentials.as_bytes(), base64::STANDARD);
        self.headers
            .insert("Authorization".to_string(), format!("Basic {}", encoded));
        self
    }

    pub fn bearer_auth<T: Into<String>>(mut self, token: T) -> Self {
        self.headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", token.into()),
        );
        self
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn body<S: Into<String>>(mut self, body: S) -> Self {
        self.body = Some(HttpBody::Text(body.into()));
        self
    }

    pub fn body_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = Some(HttpBody::Binary(body));
        self
    }

    pub fn form(mut self, form: HashMap<String, String>) -> Self {
        self.body = Some(HttpBody::Form(form));
        self.headers.insert(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );
        self
    }

    pub fn multipart(mut self, form: Vec<MultipartField>) -> Self {
        self.body = Some(HttpBody::Multipart(form));
        self
    }

    pub fn json<T: serde::Serialize>(mut self, json: &T) -> Result<Self, HttpError> {
        let json_body = serde_json::to_string(json).map_err(|_| HttpError::SerializationError)?;
        self.body = Some(HttpBody::Text(json_body));
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn send(self) -> Result<HttpResponse, HttpError> {
        self.client.execute(&self)
    }
}

// ====================
// Utility Functions
// ====================

pub fn build_url_with_params(base_url: &str, params: &HashMap<String, String>) -> String {
    if params.is_empty() {
        return base_url.to_string();
    }
    match url::Url::parse(base_url) {
        Ok(mut url) => {
            for (key, value) in params {
                url.query_pairs_mut().append_pair(key, value);
            }
            url.to_string()
        }
        Err(_) => {
            // Fallback for invalid URLs - append query parameters manually
            let mut url = base_url.to_string();
            let separator = if url.contains('?') { '&' } else { '?' };
            url.push(separator);

            let encoded_params: Vec<String> = params
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        url::form_urlencoded::byte_serialize(k.as_bytes()).collect::<String>(),
                        url::form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>()
                    )
                })
                .collect();
            url.push_str(&encoded_params.join("&"));
            url
        }
    }
}

// ====================
// Module-level Convenience Functions
// ====================

pub fn get<U: Into<String>>(url: U) -> RequestBuilder {
    HttpClient::new().get(url)
}

pub fn post<U: Into<String>>(url: U) -> RequestBuilder {
    HttpClient::new().post(url)
}

pub fn put<U: Into<String>>(url: U) -> RequestBuilder {
    HttpClient::new().put(url)
}

pub fn patch<U: Into<String>>(url: U) -> RequestBuilder {
    HttpClient::new().patch(url)
}

pub fn delete<U: Into<String>>(url: U) -> RequestBuilder {
    HttpClient::new().delete(url)
}

pub fn head<U: Into<String>>(url: U) -> RequestBuilder {
    HttpClient::new().head(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_array_body() {
        let json_str = r#"{"success":true,"data":{"status":200,"headers":{"content-type":"application/json"},"body":[123,34,104,101,108,108,111,34,58,34,119,111,114,108,100,34,125],"url":"https://httpbin.org/get"}}"#;

        let result: HttpResult = serde_json::from_str(json_str).unwrap();
        assert!(result.success);

        let response = result.data.unwrap();
        assert_eq!(response.status, 200);

        // The body should be: {"hello":"world"}
        let expected_body = b"{\"hello\":\"world\"}";
        assert_eq!(response.body, expected_body);

        let body_text = response.text().unwrap();
        assert_eq!(body_text, "{\"hello\":\"world\"}");
    }

    #[test]
    fn test_multipart_field_creation() {
        let text_field = MultipartField::text("name", "value");
        assert_eq!(text_field.name, "name");
        match text_field.value {
            MultipartValue::Text(ref v) => assert_eq!(v, "value"),
            _ => panic!("Expected text value"),
        }

        let binary_field =
            MultipartField::binary("file", vec![1, 2, 3], Some("test.bin".to_string()), None);
        assert_eq!(binary_field.name, "file");
        match binary_field.value {
            MultipartValue::Binary {
                ref data,
                ref filename,
                ..
            } => {
                assert_eq!(data, &vec![1, 2, 3]);
                assert_eq!(filename.as_ref().unwrap(), "test.bin");
            }
            _ => panic!("Expected binary value"),
        }
    }

    #[test]
    fn test_url_building() {
        let mut params = HashMap::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "value with spaces".to_string());

        let url = build_url_with_params("https://example.com/api", &params);
        assert!(url.contains("key1=value1"));
        assert!(url.contains("key2=value+with+spaces"));
        assert!(url.starts_with("https://example.com/api?"));
    }

    #[test]
    fn test_url_building_special_chars() {
        let mut params = HashMap::new();
        params.insert("special".to_string(), "!@#$%^&*()".to_string());
        params.insert("utf8".to_string(), "こんにちは".to_string());
        params.insert("reserved".to_string(), "test&foo=bar".to_string());

        let url = build_url_with_params("https://example.com/api", &params);

        // Check that special characters are properly encoded
        // Note: url crate uses + for spaces and different encoding for some chars
        assert!(url.contains("special=%21%40%23%24%25%5E%26*%28%29"));
        assert!(url.contains("reserved=test%26foo%3Dbar"));
        // UTF-8 characters should be percent-encoded
        assert!(url.contains("utf8=%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF"));
    }

    #[test]
    fn test_url_building_with_existing_query() {
        let mut params = HashMap::new();
        params.insert("new_param".to_string(), "new_value".to_string());

        let url = build_url_with_params("https://example.com/api?existing=param", &params);
        assert!(url.contains("existing=param"));
        assert!(url.contains("new_param=new_value"));
        assert!(url.contains("&"));
    }

    #[test]
    fn test_url_building_empty_params() {
        let params = HashMap::new();
        let url = build_url_with_params("https://example.com/api", &params);
        assert_eq!(url, "https://example.com/api");
    }

    #[test]
    fn test_client_builder() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Blockless-SDK/1.0".to_string());

        let client = HttpClient::builder()
            .default_headers(headers)
            .timeout(10000)
            .build();

        assert!(client.default_headers.is_some());
        assert_eq!(client.timeout, Some(10000));
    }

    #[test]
    fn test_request_builder() {
        let client = HttpClient::new();
        let request = client
            .post("https://httpbin.org/post")
            .header("Content-Type", "application/json")
            .query("search", "test")
            .query("limit", "10")
            .body("test body")
            .timeout(5000);

        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://httpbin.org/post");
        assert_eq!(
            request.headers.get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(request.query_params.get("search").unwrap(), "test");
        assert_eq!(request.query_params.get("limit").unwrap(), "10");
        assert_eq!(request.timeout, Some(5000));

        match request.body.as_ref().unwrap() {
            HttpBody::Text(ref body) => assert_eq!(body, "test body"),
            _ => panic!("Expected text body"),
        }
    }

    #[test]
    fn test_basic_auth() {
        let client = HttpClient::new();
        let request = client
            .get("https://httpbin.org/basic-auth/user/pass")
            .basic_auth("username", "password");

        let auth_header = request.headers.get("Authorization").unwrap();
        assert!(auth_header.starts_with("Basic "));

        // Verify it's properly base64 encoded "username:password"
        let encoded_part = &auth_header[6..]; // Remove "Basic " prefix
        let decoded = base64::decode_config(encoded_part, base64::STANDARD).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, "username:password");
    }

    #[test]
    fn test_bearer_auth() {
        let client = HttpClient::new();
        let request = client
            .get("https://httpbin.org/bearer")
            .bearer_auth("test-token-123");

        let auth_header = request.headers.get("Authorization").unwrap();
        assert_eq!(auth_header, "Bearer test-token-123");
    }

    #[test]
    fn test_query_params_integration() {
        let mut params1 = HashMap::new();
        params1.insert("base".to_string(), "param".to_string());

        let client = HttpClient::new();
        let request = client
            .get("https://api.example.com/search")
            .query_params(params1)
            .query("additional", "value")
            .query("special chars", "test & encode");

        assert_eq!(request.query_params.get("base").unwrap(), "param");
        assert_eq!(request.query_params.get("additional").unwrap(), "value");
        assert_eq!(
            request.query_params.get("special chars").unwrap(),
            "test & encode"
        );

        // Test URL building
        let url = build_url_with_params("https://api.example.com/search", &request.query_params);
        assert!(url.contains("base=param"));
        assert!(url.contains("additional=value"));
        assert!(url.contains("special+chars=test+%26+encode"));
    }

    #[test]
    fn test_module_level_functions() {
        // Test that module-level convenience functions work
        let _get_request = get("https://httpbin.org/get");
        let _post_request = post("https://httpbin.org/post");
        let _put_request = put("https://httpbin.org/put");
        let _patch_request = patch("https://httpbin.org/patch");
        let _delete_request = delete("https://httpbin.org/delete");

        // These should all return RequestBuilder objects
        let request = get("https://httpbin.org/get")
            .query("test", "value")
            .header("User-Agent", "test");

        assert_eq!(request.method, "GET");
        assert_eq!(request.url, "https://httpbin.org/get");
        assert_eq!(request.query_params.get("test").unwrap(), "value");
        assert_eq!(request.headers.get("User-Agent").unwrap(), "test");
    }
}
