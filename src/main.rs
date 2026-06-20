mod actions;
mod app;
mod config;
mod input;
mod render;
mod sim;
mod term;

use clap::Parser;
use config::CliArgs;

fn main() -> anyhow::Result<()> {
    // ── Panic hook: guarantee terminal cleanup even on unwind ─────────
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Teardown BEFORE the panic message prints, so the user sees it
        // in a normal terminal state.
        term::guard::teardown();
        default_hook(info);
    }));

    let args = CliArgs::parse();

    // ── Terminal guard (RAII) — raw mode, alt screen, hidden cursor ──
    let _guard = term::TermGuard::new()?;

    app::run(args)
}
