#[derive(clap::Parser, Debug)]
pub struct Args {
    /// Set logging level for the app.
    #[clap(short, long, default_value = "info")]
    pub log_level: tracing::level_filters::LevelFilter,
}
