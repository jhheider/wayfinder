pub mod results;
pub mod service;

pub use results::{filter_legacy_duplicates, group_broad_results, is_legacy};
pub use service::SearchService;
