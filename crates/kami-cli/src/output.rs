#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Print `value` as pretty JSON, or run `human` for a human-readable
/// rendering — the same split Kakashi uses so scripting against `--format
/// json` never has to parse prose.
pub fn emit(
    format: OutputFormat,
    value: &serde_json::Value,
    human: impl FnOnce(&serde_json::Value) -> String,
) {
    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(value).expect("value is valid JSON")
        ),
        OutputFormat::Human => println!("{}", human(value)),
    }
}
