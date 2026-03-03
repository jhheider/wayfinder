use tokio::sync::mpsc;
use wayfinder_core::aon::{Document, GameSystem};

use super::service::DataService;

pub enum LoadRequest {
    SearchCategory {
        category: String,
        filters: Vec<(String, String)>,
    },
    #[allow(dead_code)]
    ShowDocument {
        name: String,
        category: Option<String>,
    },
}

pub enum LoadResult {
    Documents(Vec<Document>),
    Error(String),
}

/// Spawn background loader that processes requests and sends results.
pub fn spawn_loader(
    system: GameSystem,
) -> (
    mpsc::UnboundedSender<LoadRequest>,
    mpsc::UnboundedReceiver<LoadResult>,
) {
    let (req_tx, mut req_rx) = mpsc::unbounded_channel::<LoadRequest>();
    let (res_tx, res_rx) = mpsc::unbounded_channel::<LoadResult>();

    tokio::spawn(async move {
        let mut service = match DataService::new(system) {
            Ok(s) => s,
            Err(e) => {
                let _ = res_tx.send(LoadResult::Error(e.to_string()));
                return;
            }
        };

        while let Some(req) = req_rx.recv().await {
            let result = match req {
                LoadRequest::SearchCategory { category, filters } => {
                    service.search_category(&category, &filters).await
                }
                LoadRequest::ShowDocument { name, category } => {
                    service.show(&name, category.as_deref()).await
                }
            };
            let msg = match result {
                Ok(docs) => LoadResult::Documents(docs),
                Err(e) => LoadResult::Error(e.to_string()),
            };
            if res_tx.send(msg).is_err() {
                break;
            }
        }
    });

    (req_tx, res_rx)
}
