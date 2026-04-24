use clap::ValueEnum;
use websearch::{SearchError, SearchOptions, SearchProvider, providers, web_search};

#[derive(ValueEnum, Clone)]
pub enum SearchEngine {
    DuckDuckGo,
}

fn get_provider(provider: Option<SearchEngine>) -> Box<dyn SearchProvider> {
    match provider {
        Some(SearchEngine::DuckDuckGo) => Box::new(providers::DuckDuckGoProvider::new()),
        None => {
            println!("No provider supplied, defaulting to DuckDuckGo.");
            Box::new(providers::DuckDuckGoProvider::new())
        }
    }
}

fn create_search_options(
    query: String,
    engine: Option<SearchEngine>,
    max_values: Option<u32>,
) -> SearchOptions {
    SearchOptions {
        query: query.to_string(),
        provider: get_provider(engine),
        max_results: max_values,
        id_list: None,
        language: None,
        region: None,
        safe_search: None,
        page: None,
        start: None,
        sort_by: None,
        sort_order: None,
        timeout: None,
        debug: None,
    }
}

async fn search(
    search_query: SearchOptions,
) -> Result<Vec<websearch::SearchResult>, websearch::SearchError> {
    let results = web_search(search_query).await;
    results
}

pub async fn websearch(query: String, engine: Option<SearchEngine>, max_values: Option<u32>) {
    let search_query = create_search_options(query.to_string(), engine, max_values);
    match search(search_query).await {
        Ok(results) => {
            println!("Found {} results: {:?}", results.len(), results);
        }
        Err(SearchError::AuthenticationError(msg)) => {
            eprintln!("Auth failed: {}", msg);
        }
        Err(SearchError::RateLimit(msg)) => {
            eprintln!("Rate limited: {}", msg);
        }
        Err(SearchError::HttpError {
            message,
            status_code,
            ..
        }) => {
            eprintln!("HTTP error {}: {}", status_code.unwrap_or(0), message);
        }
        Err(e) => {
            eprintln!("Search failed: {}", e);
        }
    }
}
