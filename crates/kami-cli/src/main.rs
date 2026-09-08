mod commands;
mod output;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use output::OutputFormat;

/// Kami — the ELCI recursive potentiality & transformative development
/// engine. This build implements the non-mutating first-loop baseline
/// (§12): evidence-backed project sizing, ELCI sibling detection, and a
/// single persisted `STATE-000`. It does not yet mutate, project, or
/// recurse — see `README.md` for what's directive-only.
#[derive(Parser)]
#[command(name = "kami", version, about)]
struct Cli {
    /// Project directory to operate on.
    #[arg(long, global = true, default_value = ".")]
    path: PathBuf,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the `.kami/` state directory.
    Init,
    /// Measure the project's current size (Bart storage + source LOC)
    /// and record it as evidence, without persisting a state.
    Inspect,
    /// Report which ELCI sibling adapters are available on this machine.
    Status,
    /// Manage persisted project states (§26).
    State {
        #[command(subcommand)]
        action: StateAction,
    },
}

#[derive(Subcommand)]
enum StateAction {
    /// Build and persist the STATE-000 baseline from measured evidence.
    Build {
        /// The project's mission statement (§3.2). Optional — a state
        /// without one simply has no mission recorded yet.
        #[arg(long)]
        mission: Option<String>,
    },
    /// List persisted state ids.
    List,
    /// Show one persisted state.
    Show {
        #[arg(long)]
        id: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init => commands::init(&cli.path),
        Commands::Inspect => commands::inspect(&cli.path),
        Commands::Status => commands::status(&cli.path),
        Commands::State { action } => match action {
            StateAction::Build { mission } => commands::state_build(&cli.path, mission.clone()),
            StateAction::List => commands::state_list(&cli.path),
            StateAction::Show { id } => commands::state_show(&cli.path, *id),
        },
    }?;

    let human = match &cli.command {
        Commands::Init => render_init,
        Commands::Inspect => render_inspect,
        Commands::Status => render_status,
        Commands::State { action } => match action {
            StateAction::Build { .. } => render_state_build,
            StateAction::List => render_state_list,
            StateAction::Show { .. } => render_state_one,
        },
    };

    output::emit(cli.format, &result, human);
    Ok(())
}

fn render_init(v: &serde_json::Value) -> String {
    format!("initialized {}", v["state_dir"].as_str().unwrap_or("?"))
}

fn render_inspect(v: &serde_json::Value) -> String {
    let size = &v["size"];
    let mut out = format!(
        "storage: {} bytes across {} files\nsource LOC: {} (confidence {:.2})",
        size["storage_bytes"], size["file_count"], size["source_loc"], size["confidence"]
    );
    if let Some(langs) = size["language_loc"].as_object() {
        if !langs.is_empty() {
            out.push_str("\nby language:");
            for (lang, loc) in langs {
                out.push_str(&format!("\n  {lang}: {loc}"));
            }
        }
    }
    if let Some(assumptions) = size["assumptions"].as_array() {
        for a in assumptions {
            if let Some(s) = a.as_str() {
                out.push_str(&format!("\nassumption: {s}"));
            }
        }
    }
    out
}

fn render_status(v: &serde_json::Value) -> String {
    let mut out = String::from("ELCI sibling adapters:");
    if let Some(adapters) = v["adapters"].as_array() {
        for a in adapters {
            let mark = if a["available"].as_bool().unwrap_or(false) {
                "AVAILABLE"
            } else {
                "CAPABILITY_UNAVAILABLE"
            };
            out.push_str(&format!(
                "\n  {:<14} [{}] {} — {}",
                a["name"].as_str().unwrap_or("?"),
                mark,
                a["role"].as_str().unwrap_or(""),
                a["detail"].as_str().unwrap_or(""),
            ));
        }
    }
    out
}

fn render_state_list(v: &serde_json::Value) -> String {
    let states = v["states"].as_array().map(|a| a.len()).unwrap_or(0);
    if states == 0 {
        return "no states persisted yet — run `kami state build`".to_string();
    }
    let ids: Vec<&str> = v["states"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    format!("states: {}", ids.join(", "))
}

fn render_state_build(v: &serde_json::Value) -> String {
    let mut out = render_state_one(v);
    if let Some(passes) = v["battery"].as_array() {
        out.push_str("\nfirst-loop battery (§12):");
        for p in passes {
            let outcome = p["outcome"].as_str().unwrap_or("?");
            let mark = match outcome {
                "evidence" => "EVIDENCE",
                "unavailable" => "UNAVAILABLE",
                "not_applicable" => "NOT_APPLICABLE",
                "invocation_failed" => "INVOCATION_FAILED",
                other => other,
            };
            out.push_str(&format!(
                "\n  {:<10} [{}]",
                p["tool"].as_str().unwrap_or("?"),
                mark
            ));
            if let Some(detail) = p["detail"].as_str() {
                out.push_str(&format!(" — {detail}"));
            }
        }
    }
    out
}

fn render_state_one(v: &serde_json::Value) -> String {
    let s = &v["state"];
    let mission = s["mission"]["statement"].as_str().unwrap_or("(none)");
    let mut out = format!(
        "{} ({})\nproject_root: {}\nmission: {}\nconstraints: {}, capabilities: {}, evidence: {}",
        s["id"].as_str().unwrap_or("?"),
        s["kind"].as_str().unwrap_or("?"),
        s["project_root"].as_str().unwrap_or("?"),
        mission,
        s["constraints"].as_array().map(|a| a.len()).unwrap_or(0),
        s["capabilities"].as_array().map(|a| a.len()).unwrap_or(0),
        s["evidence_ids"].as_array().map(|a| a.len()).unwrap_or(0),
    );
    render_findings_list(&mut out, "strengths", &s["strengths"]);
    render_findings_list(&mut out, "weaknesses", &s["weaknesses"]);
    out
}

fn render_findings_list(out: &mut String, label: &str, findings: &serde_json::Value) {
    let items = findings.as_array().map(Vec::as_slice).unwrap_or(&[]);
    out.push_str(&format!("\n{label} ({}):", items.len()));
    for f in items {
        out.push_str(&format!(
            "\n  - {}",
            f["description"].as_str().unwrap_or("?")
        ));
    }
}
