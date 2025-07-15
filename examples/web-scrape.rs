use blockless_sdk::bless_crawl::*;

/// This example demonstrates how to use the Blockless SDK to perform web scraping
/// using the BlessCrawl functionality.
///
/// It shows how to:
/// - Create a BlessCrawl instance with default configuration
/// - Scrape content from a single URL with custom configuration overrides
/// - Map links from a webpage to discover available URLs
/// - Handle errors and responses appropriately
fn main() {
    println!("=== Blockless Web Scraping SDK Example ===\n");

    example_scraping();
    example_mapping();
    example_crawling();
}

fn example_scraping() {
    println!("--- Example 1: Basic Web Scraping ---");

    let url = "https://example.com";
    println!("scraping: {}...", url);

    // First scrape with default config
    let response = BlessCrawl::default()
        .scrape(url, None)
        .expect("Failed to scrape");
    println!("response with default config: {:?}", response);
    println!();
    println!(
        "---------- markdown ----------\n{}\n------------------------------",
        response.data.content
    );
}

fn example_mapping() {
    println!("--- Example 2: Link Mapping/Discovery ---");

    let url = "https://example.com";
    println!("Mapping links from: {}", url);

    let options = MapOptions::new()
        .with_link_types(vec!["internal".to_string(), "external".to_string()])
        .with_base_url(url.to_string())
        .with_filter_extensions(vec![".html".to_string(), ".htm".to_string()]);

    let response = BlessCrawl::default()
        .map(url, Some(options))
        .expect("Failed to map");
    println!("response: {:?}", response);
    println!();
    println!(
        "------------ links ------------\n{:?}\n------------------------------",
        response.data.links
    );
    println!();
    println!(
        "------------ total links ------------\n{}\n------------------------------",
        response.data.total_links
    );
}

fn example_crawling() {
    println!("--- Example 3: Recursive Website Crawling ---");

    let url = "https://example.com";
    println!("Crawling website: {}", url);

    let options = CrawlOptions::new()
        .with_max_depth(2)
        .with_limit(10)
        .with_include_paths(vec!["/".to_string()])
        .with_exclude_paths(vec!["/admin/".to_string(), "/api/".to_string()])
        .with_follow_external(false)
        .with_delay_between_requests(1000)
        .with_parallel_requests(3);

    let response = BlessCrawl::default()
        .crawl(url, Some(options))
        .expect("Failed to crawl");
    println!("response: {:?}", response);
    println!();
    println!(
        "------------ pages ------------\n{:?}\n------------------------------",
        response.data.pages
    );
    println!();
    println!(
        "------------ total pages ------------\n{}\n------------------------------",
        response.data.total_pages
    );
}
