//! # BlessCrawl - Distributed Web Scraping SDK
//!
//! Provides distributed web scraping across the BLESS network's browser nodes.
//!
//! ## Features
//!
//! - **scrape()**: Extract content from a URL as markdown
//! - **map()**: Discover and extract all links from a webpage
//! - **crawl()**: Recursively crawl websites with depth controls
//!
//! ## Limits
//!
//! - Timeout: 15s default, 120s max
//! - Wait time: 3s default, 20s max
//! - Buffer sizes: 2MB (scrape), 128KB (map), 8MB (crawl)

mod html_to_markdown;
mod html_transform;

use html_to_markdown::parse_markdown;
pub use html_transform::{transform_html, HtmlTransformError, TransformHtmlOptions};
use std::collections::HashMap;

type Handle = u32;
type ExitCode = u8;

#[cfg(not(feature = "mock-ffi"))]
#[link(wasm_import_module = "bless_crawl")]
extern "C" {
    /// Scrape webpage content and return as markdown
    #[allow(clippy::too_many_arguments)]
    fn scrape(
        h: *mut Handle,
        url_ptr: *const u8,
        url_len: usize,
        options_ptr: *const u8,
        options_len: usize,
        result_ptr: *mut u8,
        result_len: usize,
        bytes_written: *mut usize,
    ) -> ExitCode;

    /// Close and cleanup a web scraper instance
    fn close(h: Handle) -> ExitCode;
}

#[cfg(feature = "mock-ffi")]
#[allow(unused_variables)]
mod mock_ffi {
    use super::{ExitCode, Handle};

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn scrape(
        h: *mut Handle,
        _url_ptr: *const u8,
        _url_len: usize,
        _options_ptr: *const u8,
        _options_len: usize,
        result_ptr: *mut u8,
        result_len: usize,
        bytes_written: *mut usize,
    ) -> ExitCode {
        1
    }

    pub unsafe fn close(_h: Handle) -> ExitCode {
        1
    }
}

#[cfg(feature = "mock-ffi")]
use mock_ffi::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ScrapeOptions {
    pub timeout: u32,
    pub wait_time: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tags: Option<Vec<String>>,
    pub only_main_content: bool,
    pub format: Format,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

impl Default for ScrapeOptions {
    fn default() -> Self {
        Self {
            timeout: BlessCrawl::DEFAULT_TIMEOUT_MS,
            wait_time: BlessCrawl::DEFAULT_WAIT_TIME_MS,
            include_tags: None,
            exclude_tags: None,
            only_main_content: false,
            format: Format::Markdown,
            viewport: None,
            user_agent: None,
            headers: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Format {
    #[default]
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "json")]
    Json,
}

impl std::str::FromStr for Format {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "markdown" => Ok(Format::Markdown),
            "html" => Ok(Format::Html),
            "json" => Ok(Format::Json),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Viewport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct MapOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_extensions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct CrawlOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_external: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_between_requests: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_requests: Option<u32>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub url: String,
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robots: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_card: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_site: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scrape_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_used: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeData {
    pub success: bool,
    pub timestamp: u64,
    pub format: Format,
    pub content: String,
    pub metadata: PageMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Response<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub data: T,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkInfo {
    pub url: String,
    // TODO: use enum instead of string
    pub link_type: String, // "internal", "external", "anchor"
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MapData {
    pub url: String,
    pub links: Vec<LinkInfo>,
    pub total_links: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlError {
    pub url: String,
    pub error: String,
    pub depth: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlData<T> {
    pub root_url: String,
    pub pages: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_map: Option<MapData>,
    pub depth_reached: u8,
    pub total_pages: usize,
    pub errors: Vec<CrawlError>,
}

impl ScrapeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_include_tags(mut self, tags: Vec<String>) -> Self {
        self.include_tags = Some(tags);
        self
    }

    pub fn with_exclude_tags(mut self, tags: Vec<String>) -> Self {
        self.exclude_tags = Some(tags);
        self
    }

    pub fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.viewport = Some(Viewport {
            width: Some(width),
            height: Some(height),
        });
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }
}

impl MapOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_link_types(mut self, link_types: Vec<String>) -> Self {
        self.link_types = Some(link_types);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn with_filter_extensions(mut self, extensions: Vec<String>) -> Self {
        self.filter_extensions = Some(extensions);
        self
    }
}

impl CrawlOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_max_depth(mut self, max_depth: u8) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn with_exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.exclude_paths = Some(paths);
        self
    }

    pub fn with_include_paths(mut self, paths: Vec<String>) -> Self {
        self.include_paths = Some(paths);
        self
    }

    pub fn with_follow_external(mut self, follow: bool) -> Self {
        self.follow_external = Some(follow);
        self
    }

    pub fn with_delay_between_requests(mut self, delay: u32) -> Self {
        self.delay_between_requests = Some(delay);
        self
    }

    pub fn with_parallel_requests(mut self, parallel: u32) -> Self {
        self.parallel_requests = Some(parallel);
        self
    }
}

/// BlessCrawl client for distributed web scraping operations.
#[derive(Debug, Clone, Default)]
pub struct BlessCrawl {
    inner: Handle,
    config: ScrapeOptions,
}

impl BlessCrawl {
    /// Default timeout in milliseconds (15 seconds)
    pub const DEFAULT_TIMEOUT_MS: u32 = 15000;
    /// Default wait time in milliseconds (3 seconds)
    pub const DEFAULT_WAIT_TIME_MS: u32 = 3000;

    /// Maximum timeout in milliseconds (2 minutes)
    pub const MAX_TIMEOUT_MS: u32 = 120000;
    /// Maximum wait time in milliseconds (20 seconds)
    pub const MAX_WAIT_TIME_MS: u32 = 20000;

    /// Maximum result buffer size in bytes (2MB)
    pub const MAX_SCRAPE_BUFFER_SIZE: usize = 2 * 1024 * 1024;

    /// Maximum result buffer size in bytes (1MB)
    pub const MAX_MAP_BUFFER_SIZE: usize = 1024 * 1024;

    /// Maximum result buffer size in bytes (8MB)
    pub const MAX_CRAWL_BUFFER_SIZE: usize = 8 * 1024 * 1024;

    /// Creates a new BlessCrawl instance with the given configuration.
    pub fn with_config(config: ScrapeOptions) -> Result<Self, WebScrapeErrorKind> {
        let instance = Self { inner: 0, config };
        instance.validate_config(&instance.config)?;
        Ok(instance)
    }

    fn validate_config(&self, config: &ScrapeOptions) -> Result<(), WebScrapeErrorKind> {
        if config.timeout > Self::MAX_TIMEOUT_MS {
            return Err(WebScrapeErrorKind::InvalidTimeout);
        }
        if config.wait_time > Self::MAX_WAIT_TIME_MS {
            return Err(WebScrapeErrorKind::InvalidWaitTime);
        }
        Ok(())
    }

    /// Returns a reference to the current configuration.
    pub fn get_config(&self) -> &ScrapeOptions {
        &self.config
    }

    pub fn handle(&self) -> Handle {
        self.inner
    }

    /// Scrapes webpage content and returns it as markdown with metadata.
    pub fn scrape(
        &self,
        url: &str,
        options: Option<ScrapeOptions>,
    ) -> Result<Response<ScrapeData>, WebScrapeErrorKind> {
        // Use provided options or fall back to instance config
        let config = if let Some(opts) = options {
            self.validate_config(&opts)?;
            opts
        } else {
            self.config.clone()
        };

        let options_json = serde_json::to_vec(&config).unwrap();

        let mut handle = self.inner;
        let mut result_buf = vec![0u8; Self::MAX_SCRAPE_BUFFER_SIZE];
        let mut bytes_written: usize = 0;

        let code = unsafe {
            scrape(
                &mut handle,
                url.as_ptr(),
                url.len(),
                options_json.as_ptr(),
                options_json.len(),
                result_buf.as_mut_ptr(),
                result_buf.len(),
                &mut bytes_written,
            )
        };

        if code != 0 {
            return Err(code.into());
        }
        if bytes_written == 0 {
            return Err(WebScrapeErrorKind::EmptyResponse);
        }
        if bytes_written > result_buf.len() {
            return Err(WebScrapeErrorKind::MemoryError);
        }

        let result_bytes =
            unsafe { std::slice::from_raw_parts(result_buf.as_ptr(), bytes_written) };

        // deserialize the result to host ScrapeResponse
        let mut scrape_response = serde_json::from_slice::<Response<ScrapeData>>(result_bytes)
            .map_err(|e| {
                eprintln!("error: {:?}", e);
                WebScrapeErrorKind::ParseError
            })?;

        if let Some(error) = scrape_response.error {
            return Err(WebScrapeErrorKind::RuntimeError(error));
        }

        // post-process html
        scrape_response.data.content = transform_html(TransformHtmlOptions {
            html: scrape_response.data.content,
            url: scrape_response.data.metadata.url.clone(),
            include_tags: config.include_tags.unwrap_or_default(),
            exclude_tags: config.exclude_tags.unwrap_or_default(),
            only_main_content: config.only_main_content,
        })
        .map_err(|e| {
            eprintln!("error: {:?}", e);
            WebScrapeErrorKind::TransformError
        })?;

        // if the format is markdown, set the data to the markdown of the html
        match config.format {
            Format::Markdown => {
                scrape_response.data.content = parse_markdown(&scrape_response.data.content);
            }
            Format::Html => (), // no need to do anything
            Format::Json => unimplemented!(),
        }

        // convert the host ScrapeResponse to the user ScrapeResponse
        Ok(scrape_response)
    }

    /// Extracts all links from a webpage, categorized by type.
    pub fn map(
        &self,
        url: &str,
        options: Option<MapOptions>,
    ) -> Result<Response<MapData>, WebScrapeErrorKind> {
        let _map_options = options.unwrap_or_default();

        // let scrape_response = self.scrape(url, None)?;
        // TODO: implement map by post-processing the scrape response or using fetch

        Ok(Response {
            success: true,
            error: None,
            data: MapData {
                url: url.to_string(),
                links: vec![],
                total_links: 0,
                timestamp: 0,
            },
        })
    }

    /// Recursively crawls a website with configurable depth and filtering.
    pub fn crawl(
        &self,
        url: &str,
        options: Option<CrawlOptions>,
    ) -> Result<Response<CrawlData<ScrapeData>>, WebScrapeErrorKind> {
        let _crawl_options = options.unwrap_or_default();

        // TODO: implement crawl by post-processing the scrape response or using fetch

        Ok(Response {
            success: true,
            error: None,
            data: CrawlData {
                root_url: url.to_string(),
                pages: vec![],
                link_map: None,
                depth_reached: 0,
                total_pages: 0,
                errors: vec![],
            },
        })
    }
}

impl Drop for BlessCrawl {
    fn drop(&mut self) {
        // if the handle is 0, it means the instance was never initialized on the host
        if self.inner == 0 {
            return;
        }
        let code = unsafe { close(self.inner) };
        if code != 0 {
            eprintln!("Error closing web scraper: {}", code);
        }
    }
}

#[derive(Debug)]
pub enum WebScrapeErrorKind {
    InvalidUrl,
    Timeout,
    NetworkError,
    RenderingError,
    MemoryError,
    DepthExceeded,
    RateLimited,
    TransformError,
    Utf8Error,
    ParseError,
    ScrapeFailed,
    MapFailed,
    CrawlFailed,
    EmptyResponse,
    InvalidTimeout,
    InvalidWaitTime,
    RuntimeError(String),
}

impl From<u8> for WebScrapeErrorKind {
    fn from(code: u8) -> Self {
        match code {
            1 => WebScrapeErrorKind::InvalidUrl,
            2 => WebScrapeErrorKind::Timeout,
            3 => WebScrapeErrorKind::NetworkError,
            4 => WebScrapeErrorKind::RenderingError,
            5 => WebScrapeErrorKind::MemoryError,
            6 => WebScrapeErrorKind::DepthExceeded,
            7 => WebScrapeErrorKind::RateLimited,
            8 => WebScrapeErrorKind::TransformError,
            9 => WebScrapeErrorKind::RuntimeError(String::from("Invalid timeout")),
            10 => WebScrapeErrorKind::RuntimeError(String::from("Invalid wait time")),
            _ => WebScrapeErrorKind::RuntimeError(String::from("Unknown error")),
        }
    }
}

impl std::fmt::Display for WebScrapeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebScrapeErrorKind::InvalidUrl => write!(f, "Invalid URL provided"),
            WebScrapeErrorKind::Timeout => write!(f, "Request timeout"),
            WebScrapeErrorKind::NetworkError => write!(f, "Network error"),
            WebScrapeErrorKind::RenderingError => write!(f, "Page rendering error"),
            WebScrapeErrorKind::MemoryError => write!(f, "Memory allocation error"),
            WebScrapeErrorKind::DepthExceeded => write!(f, "Maximum crawl depth exceeded"),
            WebScrapeErrorKind::RateLimited => write!(f, "Rate limited"),
            WebScrapeErrorKind::TransformError => write!(f, "Transform error"),
            WebScrapeErrorKind::Utf8Error => write!(f, "UTF-8 conversion error"),
            WebScrapeErrorKind::ParseError => write!(f, "JSON parse error"),
            WebScrapeErrorKind::ScrapeFailed => write!(f, "Scrape operation failed"),
            WebScrapeErrorKind::MapFailed => write!(f, "Map operation failed"),
            WebScrapeErrorKind::CrawlFailed => write!(f, "Crawl operation failed"),
            WebScrapeErrorKind::EmptyResponse => write!(f, "Empty response from host"),
            WebScrapeErrorKind::InvalidTimeout => {
                write!(f, "Timeout exceeds maximum allowed (120s)")
            }
            WebScrapeErrorKind::InvalidWaitTime => {
                write!(f, "Wait time exceeds maximum allowed (20s)")
            }
            WebScrapeErrorKind::RuntimeError(error) => write!(f, "Runtime error: {}", error),
        }
    }
}

impl std::error::Error for WebScrapeErrorKind {}
