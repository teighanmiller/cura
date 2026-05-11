use clap::ValueEnum;
use websearch::{SearchOptions, SearchProvider, providers, web_search};

#[derive(ValueEnum, Clone)]
pub enum SearchEngine {
    DuckDuckGo,
}

fn get_provider(provider: Option<SearchEngine>) -> Box<dyn SearchProvider> {
    match provider {
        Some(SearchEngine::DuckDuckGo) => Box::new(providers::DuckDuckGoProvider::new()),
        None => {
            eprintln!("No provider supplied, defaulting to DuckDuckGo.");
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

pub async fn websearch(
    query: String,
    engine: Option<SearchEngine>,
    max_values: Option<u32>,
) -> Result<String, anyhow::Error> {
    let search_query = create_search_options(query.to_string(), engine, max_values);
    let results = search(search_query).await?;
    Ok(format!("Search Results: {:?}", results))
}
