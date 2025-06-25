use htmd::HtmlToMarkdown;
use regex::Regex;

/// Parses HTML content and converts it to Markdown
///
/// This function replicates the behavior of the JavaScript parseMarkdown function:
/// - Converts HTML to Markdown using htmd
/// - Processes multi-line links by escaping newlines inside link content
/// - Removes "Skip to Content" links
/// - Returns empty string for empty/null input
pub fn parse_markdown(html: &str) -> String {
    if html.is_empty() {
        return String::new();
    }

    // Convert HTML to Markdown using htmd
    let markdown = match HtmlToMarkdown::new().convert(html) {
        Ok(md) => md,
        Err(_) => {
            // Return empty string if conversion fails
            return String::new();
        }
    };

    // Process the markdown content
    let processed_markdown = process_multiline_links(&markdown);
    let final_markdown = remove_skip_to_content_links(&processed_markdown);

    final_markdown
}

/// Processes multi-line links by escaping newlines inside link content
///
/// This function replicates the JavaScript processMultiLineLinks function:
/// - Tracks when we're inside link content (between [ and ])
/// - Escapes newlines with backslash when inside links
fn process_multiline_links(markdown_content: &str) -> String {
    let mut new_markdown_content = String::new();
    let mut link_open_count: usize = 0;

    for ch in markdown_content.chars() {
        match ch {
            '[' => {
                link_open_count += 1;
            }
            ']' => {
                link_open_count = link_open_count.saturating_sub(1);
            }
            _ => {}
        }

        let inside_link_content = link_open_count > 0;

        if inside_link_content && ch == '\n' {
            new_markdown_content.push('\\');
            new_markdown_content.push('\n');
        } else {
            new_markdown_content.push(ch);
        }
    }

    new_markdown_content
}

/// Removes "Skip to Content" links from the markdown content
///
/// This function replicates the JavaScript removeSkipToContentLinks function:
/// - Removes [Skip to Content](#page) and [Skip to content](#skip) patterns
/// - Case-insensitive matching
fn remove_skip_to_content_links(markdown_content: &str) -> String {
    let re = Regex::new(r"(?i)\[Skip to Content\]\(#[^)]*\)").unwrap();
    re.replace_all(markdown_content, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_simple() {
        let html = "<p>Hello, world!</p>";
        let result = parse_markdown(html);
        assert_eq!(result.trim(), "Hello, world!");
    }

    #[test]
    fn test_parse_markdown_complex() {
        let html =
            "<div><p>Hello <strong>bold</strong> world!</p><ul><li>List item</li></ul></div>";
        let result = parse_markdown(html);
        assert_eq!(result.trim(), "Hello **bold** world!\n\n*   List item");
    }

    #[test]
    fn test_parse_markdown_empty() {
        let html = "";
        let result = parse_markdown(html);
        assert_eq!(result, "");
    }

    #[test]
    fn test_process_multiline_links() {
        let markdown = "[Link\nwith newline](http://example.com)";
        let result = process_multiline_links(markdown);
        assert_eq!(result, "[Link\\\nwith newline](http://example.com)");
    }

    #[test]
    fn test_remove_skip_to_content_links() {
        let markdown = "Some content [Skip to Content](#page) more content";
        let result = remove_skip_to_content_links(markdown);
        assert_eq!(result, "Some content  more content");
    }

    #[test]
    fn test_remove_skip_to_content_links_case_insensitive() {
        let markdown = "Some content [Skip to content](#skip) more content";
        let result = remove_skip_to_content_links(markdown);
        assert_eq!(result, "Some content  more content");
    }
}
