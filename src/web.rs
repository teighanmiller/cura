use clap::Parser;
use clap::ValueEnum;
use websearch::{SearchOptions, SearchProvider, providers, web_search};

#[derive(ValueEnum, Clone)]
pub enum SearchEngine {
    DuckDuckGo,
}

#[derive(Parser, Clone)]
pub struct WebArgs {
    query: String,
    #[arg(short, long)]
    engine: Option<SearchEngine>,
    #[arg(long)]
    max_value: Option<u32>,
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
    web_search(search_query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_search_options_sets_query() {
        let opts = create_search_options("rust testing".to_string(), None, None);
        assert_eq!(opts.query, "rust testing");
        assert!(opts.max_results.is_none());
    }

    #[test]
    fn create_search_options_sets_max_results() {
        let opts = create_search_options("hello".to_string(), None, Some(5));
        assert_eq!(opts.max_results, Some(5));
    }

    #[test]
    fn create_search_options_optional_fields_none() {
        let opts = create_search_options("q".to_string(), None, None);
        assert!(opts.id_list.is_none());
        assert!(opts.language.is_none());
        assert!(opts.region.is_none());
    }
}

pub async fn websearch(args: WebArgs) -> Result<String, anyhow::Error> {
    let search_query = create_search_options(args.query.to_string(), args.engine, args.max_value);
    let results = search(search_query).await?;
    Ok(format!("Search Results: {:?}", results))
}
