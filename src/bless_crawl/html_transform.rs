use kuchikiki::{parse_html, traits::TendrilSink};
use serde::{Deserialize, Serialize};
use url::Url;

const EXCLUDE_NON_MAIN_TAGS: [&str; 41] = [
    "header",
    "footer",
    "nav",
    "aside",
    ".header",
    ".top",
    ".navbar",
    "#header",
    ".footer",
    ".bottom",
    "#footer",
    ".sidebar",
    ".side",
    ".aside",
    "#sidebar",
    ".modal",
    ".popup",
    "#modal",
    ".overlay",
    ".ad",
    ".ads",
    ".advert",
    "#ad",
    ".lang-selector",
    ".language",
    "#language-selector",
    ".social",
    ".social-media",
    ".social-links",
    "#social",
    ".menu",
    ".navigation",
    "#nav",
    ".breadcrumbs",
    "#breadcrumbs",
    ".share",
    "#share",
    ".widget",
    "#widget",
    ".cookie",
    "#cookie",
];

const FORCE_INCLUDE_MAIN_TAGS: [&str; 13] = [
    "#main",
    // swoogo event software as .widget in all of their content
    ".swoogo-cols",
    ".swoogo-text",
    ".swoogo-table-div",
    ".swoogo-space",
    ".swoogo-alert",
    ".swoogo-sponsors",
    ".swoogo-title",
    ".swoogo-tabs",
    ".swoogo-logo",
    ".swoogo-image",
    ".swoogo-button",
    ".swoogo-agenda",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransformHtmlOptions {
    pub html: String,
    pub url: String,
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub only_main_content: bool,
}

#[derive(Debug)]
struct ImageSource {
    url: String,
    size: i32,
    is_x: bool,
}

#[derive(Debug)]
pub enum HtmlTransformError {
    ParseError,
    UrlParseError,
    SelectError,
}

impl std::fmt::Display for HtmlTransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HtmlTransformError::ParseError => write!(f, "Failed to parse HTML"),
            HtmlTransformError::UrlParseError => write!(f, "Failed to parse URL"),
            HtmlTransformError::SelectError => write!(f, "Failed to select HTML elements"),
        }
    }
}

impl std::error::Error for HtmlTransformError {}

/// Transforms HTML by removing unwanted elements, filtering tags, and processing URLs
pub fn transform_html(opts: TransformHtmlOptions) -> Result<String, HtmlTransformError> {
    let mut document = parse_html().one(opts.html);

    // If include_tags is specified, only include those tags
    if !opts.include_tags.is_empty() {
        let new_document = parse_html().one("<div></div>");
        let root = new_document
            .select_first("div")
            .map_err(|_| HtmlTransformError::SelectError)?;

        for tag_selector in opts.include_tags.iter() {
            let matching_nodes: Vec<_> = document
                .select(tag_selector)
                .map_err(|_| HtmlTransformError::SelectError)?
                .collect();
            for tag in matching_nodes {
                root.as_node().append(tag.as_node().clone());
            }
        }

        document = new_document;
    }

    // Remove unwanted elements
    let unwanted_selectors = ["head", "meta", "noscript", "style", "script"];
    for selector in &unwanted_selectors {
        while let Ok(element) = document.select_first(selector) {
            element.as_node().detach();
        }
    }

    // Remove excluded tags
    for tag_selector in opts.exclude_tags.iter() {
        while let Ok(element) = document.select_first(tag_selector) {
            element.as_node().detach();
        }
    }

    // Remove non-main content if requested
    if opts.only_main_content {
        for selector in EXCLUDE_NON_MAIN_TAGS.iter() {
            let elements: Vec<_> = document
                .select(selector)
                .map_err(|_| HtmlTransformError::SelectError)?
                .collect();
            for element in elements {
                // Check if this element contains any force-include tags
                let should_keep = FORCE_INCLUDE_MAIN_TAGS.iter().any(|force_selector| {
                    element
                        .as_node()
                        .select(force_selector)
                        .map(|mut iter| iter.next().is_some())
                        .unwrap_or(false)
                });

                if !should_keep {
                    element.as_node().detach();
                }
            }
        }
    }

    // Process images with srcset attributes
    let srcset_images: Vec<_> = document
        .select("img[srcset]")
        .map_err(|_| HtmlTransformError::SelectError)?
        .collect();

    for img in srcset_images {
        let srcset = img.attributes.borrow().get("srcset").map(|s| s.to_string());
        if let Some(srcset) = srcset {
            let mut sizes: Vec<ImageSource> = srcset
                .split(',')
                .filter_map(|entry| {
                    let tokens: Vec<&str> = entry.trim().split(' ').collect();
                    if tokens.is_empty() {
                        return None;
                    }

                    let size_token = if tokens.len() > 1 && !tokens[1].is_empty() {
                        tokens[1]
                    } else {
                        "1x"
                    };

                    if let Ok(parsed_size) = size_token[..size_token.len() - 1].parse() {
                        Some(ImageSource {
                            url: tokens[0].to_string(),
                            size: parsed_size,
                            is_x: size_token.ends_with('x'),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            // Add src attribute as 1x if all sizes are x-based
            if sizes.iter().all(|s| s.is_x) {
                let src = img.attributes.borrow().get("src").map(|s| s.to_string());
                if let Some(src) = src {
                    sizes.push(ImageSource {
                        url: src,
                        size: 1,
                        is_x: true,
                    });
                }
            }

            // Sort by size (largest first) and use the biggest image
            sizes.sort_by(|a, b| b.size.cmp(&a.size));
            if let Some(biggest) = sizes.first() {
                img.attributes
                    .borrow_mut()
                    .insert("src", biggest.url.clone());
            }
        }
    }

    // Convert relative URLs to absolute URLs
    let base_url = Url::parse(&opts.url).map_err(|_| HtmlTransformError::UrlParseError)?;

    // Process image src attributes
    let src_images: Vec<_> = document
        .select("img[src]")
        .map_err(|_| HtmlTransformError::SelectError)?
        .collect();
    for img in src_images {
        let old_src = img.attributes.borrow().get("src").map(|s| s.to_string());
        if let Some(old_src) = old_src {
            if let Ok(new_url) = base_url.join(&old_src) {
                img.attributes
                    .borrow_mut()
                    .insert("src", new_url.to_string());
            }
        }
    }

    // Process anchor href attributes
    let href_anchors: Vec<_> = document
        .select("a[href]")
        .map_err(|_| HtmlTransformError::SelectError)?
        .collect();
    for anchor in href_anchors {
        let old_href = anchor
            .attributes
            .borrow()
            .get("href")
            .map(|s| s.to_string());
        if let Some(old_href) = old_href {
            if let Ok(new_url) = base_url.join(&old_href) {
                anchor
                    .attributes
                    .borrow_mut()
                    .insert("href", new_url.to_string());
            }
        }
    }

    Ok(document.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_html_removes_unwanted_elements() {
        let opts = TransformHtmlOptions {
            html: "<html><head><title>Test</title></head><body><p>Content</p><script>alert('test')</script></body></html>".to_string(),
            url: "https://example.com".to_string(),
            include_tags: vec![],
            exclude_tags: vec![],
            only_main_content: false,
        };

        let result = transform_html(opts).unwrap();
        let expected = "<html><body><p>Content</p></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transform_html_include_tags() {
        let opts = TransformHtmlOptions {
            html: "<html><body><div class=\"content\">Keep this</div><div class=\"sidebar\">Remove this</div></body></html>".to_string(),
            url: "https://example.com".to_string(),
            include_tags: vec![".content".to_string()],
            exclude_tags: vec![],
            only_main_content: false,
        };

        let result = transform_html(opts).unwrap();
        let expected =
            "<html><body><div><div class=\"content\">Keep this</div></div></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transform_html_exclude_tags() {
        let opts = TransformHtmlOptions {
            html: "<html><body><div class=\"content\">Keep this</div><div class=\"ad\">Remove this</div></body></html>".to_string(),
            url: "https://example.com".to_string(),
            include_tags: vec![],
            exclude_tags: vec![".ad".to_string()],
            only_main_content: false,
        };

        let result = transform_html(opts).unwrap();
        let expected = "<html><body><div class=\"content\">Keep this</div></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transform_html_relative_urls() {
        let opts = TransformHtmlOptions {
            html: r#"<html><body><img src="/image.jpg"><a href="/page">Link</a></body></html>"#
                .to_string(),
            url: "https://example.com/subdir/".to_string(),
            include_tags: vec![],
            exclude_tags: vec![],
            only_main_content: false,
        };

        let result = transform_html(opts).unwrap();
        let expected = r#"<html><body><img src="https://example.com/image.jpg"><a href="https://example.com/page">Link</a></body></html>"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transform_html_only_main_content() {
        let opts = TransformHtmlOptions {
            html: "<html><body><header>Header</header><main><p>Main content</p></main><footer>Footer</footer></body></html>".to_string(),
            url: "https://example.com".to_string(),
            include_tags: vec![],
            exclude_tags: vec![],
            only_main_content: true,
        };

        let result = transform_html(opts).unwrap();
        let expected = "<html><body><main><p>Main content</p></main></body></html>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transform_html_srcset_processing() {
        let opts = TransformHtmlOptions {
            html: r#"<html><body><img srcset="/small.jpg 1x, /large.jpg 2x" src="/default.jpg"></body></html>"#.to_string(),
            url: "https://example.com".to_string(),
            include_tags: vec![],
            exclude_tags: vec![],
            only_main_content: false,
        };

        let result = transform_html(opts).unwrap();
        let expected = r#"<html><body><img srcset="/small.jpg 1x, /large.jpg 2x" src="https://example.com/large.jpg"></body></html>"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transform_html_force_include_tags() {
        let opts = TransformHtmlOptions {
            html: r#"<html><body><div class="widget"><div id="main"><p>Important content</p></div></div><div class="sidebar">Sidebar</div></body></html>"#.to_string(),
            url: "https://example.com".to_string(),
            include_tags: vec![],
            exclude_tags: vec![],
            only_main_content: true,
        };

        let result = transform_html(opts).unwrap();
        let expected = r#"<html><body><div class="widget"><div id="main"><p>Important content</p></div></div></body></html>"#;
        assert_eq!(result, expected);
    }
}
