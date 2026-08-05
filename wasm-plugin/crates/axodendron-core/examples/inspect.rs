use std::env;
use std::fs;
use std::process::ExitCode;

use axodendron_core::{Severity, ValidationProfile, parse_swc};

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: cargo run -p axodendron-core --example inspect -- FILE.swc");
        return ExitCode::from(2);
    };
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let parsed = parse_swc(&source, ValidationProfile::IncfStrict);
    for diagnostic in &parsed.diagnostics {
        eprintln!(
            "{:?} {}{}: {}",
            diagnostic.severity,
            diagnostic.code,
            diagnostic
                .line
                .map_or_else(String::new, |line| format!(" at line {line}")),
            diagnostic.message
        );
    }
    let Some(morphology) = parsed.morphology else {
        return ExitCode::FAILURE;
    };
    let analysis = morphology.analyze();
    println!("nodes\t{}", analysis.summary.node_count);
    println!("edges\t{}", analysis.summary.edge_count);
    println!("roots\t{}", analysis.summary.root_count);
    println!("branch_points\t{}", analysis.summary.branch_point_count);
    println!("terminals\t{}", analysis.summary.terminal_count);
    println!("sections\t{}", analysis.summary.section_count);
    println!("cable_length\t{:.12}", analysis.summary.total_cable_length);
    println!(
        "max_path_length\t{:.12}",
        analysis.summary.max_root_path_length
    );
    println!("soma_class\t{:?}", analysis.summary.soma_class);
    if parsed
        .diagnostics
        .iter()
        .any(|item| item.severity == Severity::Error)
    {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
