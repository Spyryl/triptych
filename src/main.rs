use std::path::PathBuf;

use triptych::sentinel::input::read_evidence_list;
use triptych::sentinel::{CapsuleBuildResult, SentinelBuildRequest, build_sentinel_capsules};

fn main() {
    if let Err(error) = run() {
        eprintln!(
            "{{\"code\":\"{}\",\"message\":\"{}\"}}",
            error.code,
            json_escape(&error.message)
        );
        std::process::exit(1);
    }
}

fn run() -> triptych::sentinel::Result<()> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next().as_deref()) {
        (Some("sentinel"), Some("build")) => run_sentinel_build(args.collect()),
        (Some("sentinel"), Some("--help")) | (Some("sentinel"), Some("help")) => {
            print_sentinel_help();
            Ok(())
        }
        (Some("--help"), _) | (Some("help"), _) | (None, _) => {
            print_help();
            Ok(())
        }
        _ => {
            print_help();
            Err(triptych::sentinel::SentinelError::new(
                "UNKNOWN_COMMAND",
                "unknown command",
            ))
        }
    }
}

fn run_sentinel_build(args: Vec<String>) -> triptych::sentinel::Result<()> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_sentinel_build_help();
        return Ok(());
    }

    let mut project_root = None;
    let mut cache_root = None;
    let mut evidence_files = Vec::new();
    let mut idx = 0;

    while idx < args.len() {
        match args[idx].as_str() {
            "--project-root" => {
                idx += 1;
                project_root = args.get(idx).map(PathBuf::from);
            }
            "--cache-root" => {
                idx += 1;
                cache_root = args.get(idx).map(PathBuf::from);
            }
            "--evidence" => {
                idx += 1;
                if let Some(path) = args.get(idx) {
                    evidence_files.push(PathBuf::from(path));
                }
            }
            "--evidence-list" => {
                idx += 1;
                let Some(path) = args.get(idx) else {
                    return Err(triptych::sentinel::SentinelError::new(
                        "ARGUMENT_REQUIRED",
                        "--evidence-list requires a path",
                    ));
                };
                evidence_files.extend(read_evidence_list(&PathBuf::from(path))?);
            }
            other => {
                return Err(triptych::sentinel::SentinelError::new(
                    "UNKNOWN_ARGUMENT",
                    format!("unknown sentinel build argument: {}", other),
                ));
            }
        }
        idx += 1;
    }

    let project_root = project_root.ok_or_else(|| {
        triptych::sentinel::SentinelError::new("ARGUMENT_REQUIRED", "--project-root is required")
    })?;
    let cache_root = cache_root.ok_or_else(|| {
        triptych::sentinel::SentinelError::new("ARGUMENT_REQUIRED", "--cache-root is required")
    })?;

    let request = SentinelBuildRequest::new(project_root, cache_root, evidence_files);
    let report = build_sentinel_capsules(&request)?;
    print_capsule_report(&report.capsules);
    Ok(())
}

fn print_help() {
    println!("triptych commands:");
    println!("  sentinel build --project-root <path> --cache-root <path> --evidence <file>");
}

fn print_sentinel_help() {
    println!("triptych sentinel commands:");
    println!("  sentinel build");
}

fn print_sentinel_build_help() {
    println!("triptych sentinel build");
    println!("  --project-root <path>      Analysed project root");
    println!("  --cache-root <path>        Sentinel evidence cache root");
    println!("  --evidence <file>          Markdown evidence file; repeatable");
    println!("  --evidence-list <file>     Newline-delimited markdown evidence paths");
    println!();
    println!("stdout:");
    println!("  JSON report with ok=true and capsules[] entries");
    println!("  capsule status is created, reused, or updated");
    println!();
    println!("stderr:");
    println!("  JSON error object on failure");
}

fn print_capsule_report(capsules: &[CapsuleBuildResult]) {
    println!("{{");
    println!("  \"ok\": true,");
    println!("  \"capsules\": [");
    for (idx, result) in capsules.iter().enumerate() {
        let comma = if idx + 1 == capsules.len() { "" } else { "," };
        println!("    {{");
        println!(
            "      \"source\": \"{}\",",
            json_escape(&result.source.to_string_lossy())
        );
        println!(
            "      \"capsule\": \"{}\",",
            json_escape(&result.capsule.to_string_lossy())
        );
        println!("      \"status\": \"{}\"", result.status.as_str());
        println!("    }}{}", comma);
    }
    println!("  ]");
    println!("}}");
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}
