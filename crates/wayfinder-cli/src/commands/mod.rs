//! One module per subcommand. Each takes the shared [`Ctx`].

pub mod cache;
pub mod categories;
pub mod search;
pub mod show;

use wayfinder_core::aon::AonClient;
use wayfinder_core::aon::client::GameSystem;
use wayfinder_core::search::SearchService;

use crate::cli::OutputFormat;

/// What every command needs: the service, the game, and output settings.
pub struct Ctx {
    pub svc: SearchService<AonClient>,
    pub system: GameSystem,
    pub format: OutputFormat,
    pub legacy: bool,
    /// Colored "⚔️  PF2e" / "🚀 SF2e" banner.
    pub sys_label: String,
}
