use anyhow::{Context, Result, bail};
use nostra_extraction::contribution_graph::{
    assess_path, diff_editions, doctor, explain_path, ingest_and_write, publish_edition,
    query_graph, simulate, validate_research_portfolio,
};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_help();
        bail!("missing command");
    }

    let command = args[0].as_str();
    let root = parse_opt_value(&args, "--root")
        .map(PathBuf::from)
        .unwrap_or_else(detect_root);

    match command {
        "validate" => {
            validate_research_portfolio(&root)?;
            println!("validate: ok");
        }
        "ingest" => {
            let graph = ingest_and_write(&root)?;
            println!(
                "ingest: ok nodes={} edges={} hash={}",
                graph.nodes.len(),
                graph.edges.len(),
                graph.graph_root_hash
            );
        }
        "query" => {
            let kind = required_opt_value(&args, "--kind")?;
            let id = required_opt_value(&args, "--id")?;
            let result = query_graph(&root, kind, id)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "path" => {
            let goal = parse_opt_value(&args, "--goal").unwrap_or("stable-cortex-domain");
            let result = assess_path(&root, goal)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "explain-path" => {
            let goal = parse_opt_value(&args, "--goal").unwrap_or("stable-cortex-domain");
            let result = explain_path(&root, goal)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "doctor" => {
            let result = doctor(&root)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "simulate" => {
            let scenario = required_opt_value(&args, "--scenario")?;
            let session = simulate(&root, &PathBuf::from(scenario))?;
            println!("{}", serde_json::to_string_pretty(&session)?);
        }
        "publish-edition" => {
            let version = parse_opt_value(&args, "--version").unwrap_or("v0.2.0");
            let result = publish_edition(&root, version)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "diff-edition" => {
            let from = required_opt_value(&args, "--from")?;
            let to = required_opt_value(&args, "--to")?;
            let result = diff_editions(&root, from, to)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            print_help();
            bail!("unknown command `{command}`");
        }
    }

    Ok(())
}

fn detect_root() -> PathBuf {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("research").exists() && cwd.join("nostra").exists() {
        return cwd;
    }
    if cwd.ends_with("nostra") && cwd.parent().is_some() {
        return cwd.parent().unwrap_or(&cwd).to_path_buf();
    }
    cwd
}

fn parse_opt_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    for idx in 0..args.len() {
        if args[idx] == flag {
            return args.get(idx + 1).map(String::as_str);
        }
    }
    None
}

fn required_opt_value<'a>(args: &'a [String], flag: &str) -> Result<&'a str> {
    parse_opt_value(args, flag).with_context(|| format!("missing required option `{flag}`"))
}

fn print_help() {
    println!(
        "nostra-contribution-cli\n\
         \n\
         Commands:\n\
         - validate [--root <repo_root>]\n\
         - ingest [--root <repo_root>]\n\
         - query --kind <edge_kind> --id <contribution_id> [--root <repo_root>]\n\
         - path [--goal <goal>] [--root <repo_root>]\n\
         - explain-path [--goal <goal>] [--root <repo_root>]\n\
         - doctor [--root <repo_root>]\n\
         - simulate --scenario <path.yaml> [--root <repo_root>]\n\
         - publish-edition --version <vX.Y.Z> [--root <repo_root>]\n\
         - diff-edition --from <vX.Y.Z> --to <vX.Y.Z> [--root <repo_root>]\n\
         \n\
         Examples:\n\
         - cargo run -p nostra-extraction --bin nostra-contribution-cli -- validate --root /Users/xaoj/ICP\n\
         - cargo run -p nostra-extraction --bin nostra-contribution-cli -- ingest --root /Users/xaoj/ICP\n\
         - cargo run -p nostra-extraction --bin nostra-contribution-cli -- doctor --root /Users/xaoj/ICP\n\
         - cargo run -p nostra-extraction --bin nostra-contribution-cli -- path --goal stable-cortex-domain --root /Users/xaoj/ICP"
    );
}
