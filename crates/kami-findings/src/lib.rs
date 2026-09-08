//! Deterministic extraction of typed `Strength`/`Weakness` records from
//! first-loop battery evidence (`kami-battery`), per the directive's
//! doctrine that core accounting must stay deterministic (§24) even
//! though no LLM is wired in yet to interpret findings itself.
//!
//! Each extractor here is built and tested against a real, captured
//! example of that tool's own JSON schema — never a guessed shape. A
//! tool whose output schema hasn't been verified against a real example
//! with actual findings in it (Wilder's `findings` array was empty in
//! every fixture we tried until we found one against a real sibling
//! repo) simply isn't extracted from until it has been.
//!
//! An LLM never touches this path: `extract` is a pure function from a
//! JSON blob to typed records, with no inference step (§24's
//! deterministic/probabilistic boundary).

use kami_core::{Strength, Weakness};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Extraction {
    pub strengths: Vec<Strength>,
    pub weaknesses: Vec<Weakness>,
}

/// Extract strengths/weaknesses from one battery tool's evidence data.
/// `evidence_id` is attached to every record produced so it can be
/// traced back to the exact evidence it came from (§25). An
/// unrecognized tool name yields an empty `Extraction`, never a guess.
pub fn extract(tool: &str, evidence_id: &str, data: &Value) -> Extraction {
    match tool {
        "amber" => amber::extract(evidence_id, data),
        "traci" => traci::extract(evidence_id, data),
        "ferret" => ferret::extract(evidence_id, data),
        "fract" => fract::extract(evidence_id, data),
        "wilder" => wilder::extract(evidence_id, data),
        _ => Extraction::default(),
    }
}

mod traci {
    use super::*;

    /// Real severities observed in this tool's own output: "critical"
    /// and "error". Anything else (e.g. a future "warning"/"info" level)
    /// is left unclassified rather than guessed elevated or not.
    fn is_elevated(severity: &str) -> bool {
        matches!(severity, "critical" | "error")
    }

    pub fn extract(evidence_id: &str, data: &Value) -> Extraction {
        let mut by_rule: BTreeMap<&str, (u64, &str, &str, &str)> = BTreeMap::new();
        if let Some(diagnostics) = data.get("diagnostics").and_then(Value::as_array) {
            for d in diagnostics {
                let severity = d.get("severity").and_then(Value::as_str).unwrap_or("");
                if !is_elevated(severity) {
                    continue;
                }
                let rule = d.get("rule").and_then(Value::as_str).unwrap_or("unknown");
                let message = d.get("message").and_then(Value::as_str).unwrap_or("");
                let path = d.get("path").and_then(Value::as_str).unwrap_or("");
                let entry = by_rule.entry(rule).or_insert((0, message, path, severity));
                entry.0 += 1;
            }
        }

        let weaknesses = by_rule
            .into_iter()
            .map(|(rule, (count, message, path, severity))| Weakness {
                id: format!("traci:{rule}"),
                description: format!(
                    "Traci {rule} [{severity}]: {message} — {count} occurrence(s) (e.g. {path})"
                ),
                evidence_ids: vec![evidence_id.to_string()],
            })
            .collect();

        Extraction {
            strengths: Vec::new(),
            weaknesses,
        }
    }
}

mod ferret {
    use super::*;

    /// Real severities observed: "major" and "minor". "critical" isn't
    /// verified against real output but is included per the tool's own
    /// naming convention alongside "major"/"minor" — the only guess in
    /// this module, and a conservative one (only escalates, never
    /// suppresses a genuinely severe finding).
    fn is_elevated(severity: &str) -> bool {
        matches!(severity, "major" | "critical")
    }

    pub fn extract(evidence_id: &str, data: &Value) -> Extraction {
        let mut by_rule: BTreeMap<&str, (u64, &str, &str, &str, &str)> = BTreeMap::new();
        if let Some(findings) = data
            .get("analysis")
            .and_then(|a| a.get("findings"))
            .and_then(Value::as_array)
        {
            for f in findings {
                let severity = f.get("severity").and_then(Value::as_str).unwrap_or("");
                if !is_elevated(severity) {
                    continue;
                }
                let rule = f
                    .get("rule_id")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let category = f.get("category").and_then(Value::as_str).unwrap_or("");
                let message = f.get("message").and_then(Value::as_str).unwrap_or("");
                let file = f.get("file").and_then(Value::as_str).unwrap_or("");
                let entry = by_rule
                    .entry(rule)
                    .or_insert((0, category, message, file, severity));
                entry.0 += 1;
            }
        }

        let weaknesses = by_rule
            .into_iter()
            .map(|(rule, (count, category, message, file, severity))| Weakness {
                id: format!("ferret:{rule}"),
                description: format!(
                    "Ferret {rule} ({category}, {severity}): {message} — {count} occurrence(s) (e.g. {file})"
                ),
                evidence_ids: vec![evidence_id.to_string()],
            })
            .collect();

        Extraction {
            strengths: Vec::new(),
            weaknesses,
        }
    }
}

mod fract {
    use super::*;

    /// Fract's own top-level rollup (`summary.{warning,critical,score}`)
    /// is the deterministic signal used here, rather than re-deriving
    /// health from individual findings (whose `severity` field reads
    /// "info" even in a fully healthy scan — the summary counts are the
    /// tool's authoritative verdict, not the per-finding severity).
    pub fn extract(evidence_id: &str, data: &Value) -> Extraction {
        let Some(summary) = data.get("summary") else {
            return Extraction::default();
        };
        let total = summary.get("total").and_then(Value::as_u64).unwrap_or(0);
        let warning = summary.get("warning").and_then(Value::as_u64).unwrap_or(0);
        let critical = summary.get("critical").and_then(Value::as_u64).unwrap_or(0);
        let score = summary.get("score").and_then(Value::as_u64).unwrap_or(0);

        if total == 0 {
            return Extraction::default();
        }

        if warning == 0 && critical == 0 {
            Extraction {
                strengths: vec![Strength {
                    id: "fract:architecture-health".to_string(),
                    description: format!(
                        "Fract: all {total} module(s) architecturally healthy (entropy score {score}/100)"
                    ),
                    evidence_ids: vec![evidence_id.to_string()],
                }],
                weaknesses: Vec::new(),
            }
        } else {
            Extraction {
                strengths: Vec::new(),
                weaknesses: vec![Weakness {
                    id: "fract:architecture-health".to_string(),
                    description: format!(
                        "Fract: {warning} module(s) warning, {critical} module(s) critical architectural health (score {score}/100 across {total} module(s))"
                    ),
                    evidence_ids: vec![evidence_id.to_string()],
                }],
            }
        }
    }
}

mod amber {
    use super::*;

    /// Deterministic on `metadata.cve_count`, not on Amber's own
    /// `score.classification` opinion (e.g. "Security Critical") — that
    /// field reflects replaceability scoring philosophy, not a security
    /// verdict Kami should restate as one.
    pub fn extract(evidence_id: &str, data: &Value) -> Extraction {
        let Some(results) = data.get("results").and_then(Value::as_array) else {
            return Extraction::default();
        };
        let total = data
            .get("total_dependencies")
            .and_then(Value::as_u64)
            .unwrap_or(results.len() as u64);

        let vulnerable: Vec<(String, u64)> = results
            .iter()
            .filter_map(|r| {
                let cve_count = r.get("metadata")?.get("cve_count")?.as_u64()?;
                if cve_count == 0 {
                    return None;
                }
                let crate_name = r.get("crate")?.as_str()?.to_string();
                Some((crate_name, cve_count))
            })
            .collect();

        if vulnerable.is_empty() {
            Extraction {
                strengths: vec![Strength {
                    id: "amber:no-known-cves".to_string(),
                    description: format!(
                        "Amber: no known CVEs across {total} dependencies analyzed"
                    ),
                    evidence_ids: vec![evidence_id.to_string()],
                }],
                weaknesses: Vec::new(),
            }
        } else {
            let listing = vulnerable
                .iter()
                .map(|(name, count)| format!("{name} ({count})"))
                .collect::<Vec<_>>()
                .join(", ");
            Extraction {
                strengths: Vec::new(),
                weaknesses: vec![Weakness {
                    id: "amber:known-cves".to_string(),
                    description: format!(
                        "Amber: {} of {total} dependencies carry known CVEs: {listing}",
                        vulnerable.len()
                    ),
                    evidence_ids: vec![evidence_id.to_string()],
                }],
            }
        }
    }
}

mod wilder {
    use super::*;

    /// Real severities observed: "MEDIUM"/"HIGH" (uppercase, unlike
    /// every other tool here). Only "HIGH"/"CRITICAL" are elevated —
    /// Wilder's own finding text explicitly frames lower severities as
    /// "evidence for investigation, not a prediction of defects", so
    /// treating those as a Weakness would overstate what Wilder claims.
    ///
    /// Deliberately does **not** emit a Strength when `findings` is
    /// empty: Wilder's own `limitations` field says its coverage is
    /// partial (§29's evidence-consolidation doctrine — never let
    /// absence of a finding read as a positive claim), unlike Fract and
    /// Amber, which report a completeness count (module/dependency
    /// totals) alongside a clean result.
    fn is_elevated(severity: &str) -> bool {
        matches!(severity, "HIGH" | "CRITICAL")
    }

    pub fn extract(evidence_id: &str, data: &Value) -> Extraction {
        let mut by_category: BTreeMap<&str, (u64, &str, &str)> = BTreeMap::new();
        if let Some(findings) = data.get("findings").and_then(Value::as_array) {
            for f in findings {
                let severity = f.get("severity").and_then(Value::as_str).unwrap_or("");
                if !is_elevated(severity) {
                    continue;
                }
                let category = f
                    .get("category")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let title = f.get("title").and_then(Value::as_str).unwrap_or("");
                let entry = by_category.entry(category).or_insert((0, title, severity));
                entry.0 += 1;
            }
        }

        let weaknesses = by_category
            .into_iter()
            .map(|(category, (count, title, severity))| Weakness {
                id: format!("wilder:{category}"),
                description: format!(
                    "Wilder {category} [{severity}]: {title} — {count} finding(s)"
                ),
                evidence_ids: vec![evidence_id.to_string()],
            })
            .collect();

        Extraction {
            strengths: Vec::new(),
            weaknesses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn unrecognized_tool_extracts_nothing() {
        let e = extract("some-unknown-tool", "ev1", &json!({"anything": true}));
        assert!(e.strengths.is_empty());
        assert!(e.weaknesses.is_empty());
    }

    // --- Traci: shaped from a real `traci check --format json` capture. ---

    #[test]
    fn traci_groups_elevated_diagnostics_by_rule_and_ignores_low_severity() {
        let data = json!({
            "diagnostics": [
                {"rule": "TRC007", "severity": "critical", "path": "src/a.rs", "line": 1, "message": "panic path cannot be reconstructed from telemetry"},
                {"rule": "TRC007", "severity": "critical", "path": "src/b.rs", "line": 2, "message": "panic path cannot be reconstructed from telemetry"},
                {"rule": "TRC099", "severity": "info", "path": "src/c.rs", "line": 3, "message": "informational only, should be ignored"},
            ]
        });
        let e = extract("traci", "ev1", &data);
        assert!(e.strengths.is_empty());
        assert_eq!(e.weaknesses.len(), 1);
        let w = &e.weaknesses[0];
        assert_eq!(w.id, "traci:TRC007");
        assert!(w.description.contains("2 occurrence(s)"));
        assert_eq!(w.evidence_ids, vec!["ev1".to_string()]);
    }

    #[test]
    fn traci_with_no_diagnostics_key_extracts_nothing() {
        let e = extract("traci", "ev1", &json!({}));
        assert!(e.weaknesses.is_empty());
    }

    // --- Ferret: shaped from a real `ferret --json` capture. ---

    #[test]
    fn ferret_groups_major_findings_and_ignores_minor() {
        let data = json!({
            "analysis": {
                "findings": [
                    {"category": "safety", "file": "src/x.rs", "message": "Avoid panic/unwrap in production paths", "rule_id": "safety/no-panic", "severity": "major"},
                    {"category": "style", "file": "src/y.rs", "message": "Debug print left", "rule_id": "style/no-debug-print", "severity": "minor"},
                ]
            }
        });
        let e = extract("ferret", "ev2", &data);
        assert_eq!(e.weaknesses.len(), 1);
        assert_eq!(e.weaknesses[0].id, "ferret:safety/no-panic");
    }

    // --- Fract: shaped from a real `fract index --format json` capture. ---

    #[test]
    fn fract_all_healthy_summary_is_a_strength() {
        let data = json!({"summary": {"total": 15, "excellent": 0, "healthy": 15, "warning": 0, "critical": 0, "score": 100}, "findings": []});
        let e = extract("fract", "ev3", &data);
        assert_eq!(e.strengths.len(), 1);
        assert!(e.weaknesses.is_empty());
        assert!(e.strengths[0].description.contains("15 module(s)"));
    }

    #[test]
    fn fract_unhealthy_summary_is_a_weakness() {
        let data = json!({"summary": {"total": 10, "excellent": 0, "healthy": 7, "warning": 2, "critical": 1, "score": 61}, "findings": []});
        let e = extract("fract", "ev3", &data);
        assert!(e.strengths.is_empty());
        assert_eq!(e.weaknesses.len(), 1);
        assert!(e.weaknesses[0].description.contains("2 module(s) warning"));
        assert!(e.weaknesses[0].description.contains("1 module(s) critical"));
    }

    #[test]
    fn fract_with_zero_total_modules_extracts_nothing() {
        let data = json!({"summary": {"total": 0, "warning": 0, "critical": 0, "score": 100}});
        let e = extract("fract", "ev3", &data);
        assert!(e.strengths.is_empty());
        assert!(e.weaknesses.is_empty());
    }

    // --- Amber: shaped from a real `amber --format json .` capture. ---

    #[test]
    fn amber_flags_dependencies_with_known_cves() {
        let data = json!({
            "total_dependencies": 2,
            "results": [
                {"crate": "anyhow", "metadata": {"cve_count": 1}, "score": {"classification": "Security Critical"}},
                {"crate": "serde", "metadata": {"cve_count": 0}, "score": {"classification": "Low Risk"}},
            ]
        });
        let e = extract("amber", "ev4", &data);
        assert!(e.strengths.is_empty());
        assert_eq!(e.weaknesses.len(), 1);
        assert!(e.weaknesses[0].description.contains("anyhow (1)"));
        assert!(!e.weaknesses[0].description.contains("serde"));
    }

    #[test]
    fn amber_with_no_cves_anywhere_is_a_strength() {
        let data = json!({
            "total_dependencies": 1,
            "results": [{"crate": "serde", "metadata": {"cve_count": 0}, "score": {"classification": "Low Risk"}}]
        });
        let e = extract("amber", "ev4", &data);
        assert_eq!(e.strengths.len(), 1);
        assert!(e.weaknesses.is_empty());
    }

    // --- Wilder: shaped from a real `wilder analyse --format json` capture. ---

    #[test]
    fn wilder_groups_high_severity_findings_and_never_fabricates_a_strength() {
        let data = json!({
            "findings": [
                {"id": "WLD-1", "category": "CHANGE", "severity": "MEDIUM", "title": "Change hotspot"},
                {"id": "WLD-2", "category": "SECURITY", "severity": "HIGH", "title": "Suspicious pattern"},
            ]
        });
        let e = extract("wilder", "ev5", &data);
        assert_eq!(e.weaknesses.len(), 1);
        assert_eq!(e.weaknesses[0].id, "wilder:SECURITY");
        assert!(
            e.strengths.is_empty(),
            "wilder must never synthesize a strength from findings"
        );
    }

    #[test]
    fn wilder_with_empty_findings_extracts_nothing_at_all() {
        let e = extract("wilder", "ev5", &json!({"findings": []}));
        assert!(e.strengths.is_empty());
        assert!(e.weaknesses.is_empty());
    }
}
