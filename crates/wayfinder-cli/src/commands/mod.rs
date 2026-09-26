//! One module per subcommand. Each takes the shared [`Ctx`].

pub mod cache;
pub mod categories;
pub mod search;
pub mod show;

use wayfinder_core::Wayfinder;
use wayfinder_core::aon::Edition;

use crate::cli::OutputFormat;

/// What every command needs: the game's service and output settings.
pub struct Ctx {
    pub wf: Wayfinder,
    pub format: OutputFormat,
    pub edition: Edition,
    /// Colored "⚔️  PF2e" / "🚀 SF2e" banner.
    pub sys_label: String,
}
