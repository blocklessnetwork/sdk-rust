use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// FFI bindings for the new unified RPC interface
#[cfg(not(feature = "mock-ffi"))]
#[link(wasm_import_module = "bless")]
extern "C" {
    #[link_name = "rpc_call"]
    fn rpc_call(
        request_ptr: *const u8,
        request_len: u32,
        response_ptr: *mut u8,
        response_max_len: u32,
        bytes_written: *mut u32,
    ) -> u32;
}

#[cfg(feature = "mock-ffi")]
#[allow(unused_variables)]
mod mock_ffi {
    pub unsafe fn rpc_call(
        _request_ptr: *const u8,
        _request_len: u32,
        _response_ptr: *mut u8,
        _response_max_len: u32,
        _bytes_written: *mut u32,
    ) -> u32 {
        // Mock implementation for testing
        0
    }
}

#[cfg(feature = "mock-ffi")]
use mock_ffi::*;

#[derive(Debug, Clone)]
pub enum RpcError {
    InvalidJson,
    MethodNotFound,
    InvalidParams,
    InternalError,
    BufferTooSmall,
    Utf8Error,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::InvalidJson => write!(f, "Invalid JSON format"),
            RpcError::MethodNotFound => write!(f, "Method not found"),
            RpcError::InvalidParams => write!(f, "Invalid parameters"),
            RpcError::InternalError => write!(f, "Internal error"),
            RpcError::BufferTooSmall => write!(f, "Buffer too small"),
            RpcError::Utf8Error => write!(f, "UTF-8 conversion error"),
        }
    }
}

impl std::error::Error for RpcError {}

// JSON-RPC 2.0 structures
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Unified RPC client for calling host functions
///
/// # Example Usage
///
/// ```rust
/// use blockless_sdk::rpc::RpcClient;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct HttpRequest {
///     url: String,
///     method: String,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct HttpResponse {
///     status: u16,
///     body: String,
/// }
///
/// // Create client with default 4KB buffer
/// let mut client = RpcClient::new();
///
/// // Create client with custom buffer size (e.g., 10MB for HTTP responses)
/// let mut client = RpcClient::with_buffer_size(10 * 1024 * 1024);
///
/// // Type-safe method call
/// let request = HttpRequest {
///     url: "https://api.example.com".to_string(),
///     method: "GET".to_string(),
/// };
///
/// let response: JsonRpcResponse<HttpResponse> = client.call("http.request", Some(request))?;
///
/// // Convenience methods
/// let pong = client.ping()?;
/// let echo_result = client.echo("Hello World!")?;
/// let version = client.version()?;
///
/// // Modify buffer size after creation
/// client.set_buffer_size(1024 * 1024); // 1MB buffer
/// ```
pub struct RpcClient {
    next_id: u32,
    buffer_size: usize,
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::with_buffer_size(4096) // Default 4KB buffer
    }
}

impl RpcClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            next_id: 1,
            buffer_size,
        }
    }

    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn call<P: Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: Option<P>,
    ) -> Result<JsonRpcResponse<R>, RpcError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.next_id,
        };

        self.next_id += 1;
        let request_bytes = serde_json::to_vec(&request).map_err(|_| RpcError::InvalidJson)?;
        let mut response_buffer = vec![0u8; self.buffer_size];
        let mut bytes_written = 0u32;
        let result = unsafe {
            rpc_call(
                request_bytes.as_ptr(),
                request_bytes.len() as u32,
                response_buffer.as_mut_ptr(),
                response_buffer.len() as u32,
                &mut bytes_written as *mut u32,
            )
        };
        if result != 0 {
            return match result {
                1 => Err(RpcError::InvalidJson),
                2 => Err(RpcError::MethodNotFound),
                3 => Err(RpcError::InvalidParams),
                4 => Err(RpcError::InternalError),
                5 => Err(RpcError::BufferTooSmall),
                _ => Err(RpcError::InternalError),
            };
        }
        response_buffer.truncate(bytes_written as usize);
        serde_json::from_slice(&response_buffer).map_err(|_| RpcError::InvalidJson)
    }

    /// Convenience method for ping
    pub fn ping(&mut self) -> Result<String, RpcError> {
        let response: JsonRpcResponse<String> = self.call("ping", None::<()>)?;
        response.result.ok_or(RpcError::InternalError)
    }

    /// Convenience method for echo
    pub fn echo<T: Serialize + serde::de::DeserializeOwned>(
        &mut self,
        data: T,
    ) -> Result<T, RpcError> {
        let response: JsonRpcResponse<T> = self.call("echo", Some(data))?;
        response.result.ok_or(RpcError::InternalError)
    }

    /// Convenience method for getting version
    pub fn version(&mut self) -> Result<HashMap<String, String>, RpcError> {
        let response: JsonRpcResponse<HashMap<String, String>> =
            self.call("version", None::<()>)?;
        response.result.ok_or(RpcError::InternalError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_serialization() {
        let request: JsonRpcRequest<()> = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ping".to_string(),
            params: None,
            id: 1,
        };

        let json_str = serde_json::to_string(&request).unwrap();
        assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        assert!(json_str.contains("\"method\":\"ping\""));
        assert!(json_str.contains("\"id\":1"));
    }

    #[test]
    fn test_rpc_response_deserialization() {
        let json_str = r#"{"jsonrpc":"2.0","result":"pong","id":1}"#;
        let response: JsonRpcResponse<String> = serde_json::from_str(json_str).unwrap();

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.result, Some("pong".to_string()));
        assert_eq!(response.id, 1);
        assert!(response.error.is_none());
    }
}
