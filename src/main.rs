use dotagent::agent::run_agent;
use dotagent::env::load_env_from_current_working_directory;
use dotagent::tools::built_in_tool_specs;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{err}");
        eprintln!();
        std::process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    let _ = load_env_from_current_working_directory()?;

    if std::env::var("OPENAI_API_KEY").is_err() {
        eprintln!(
            "OPENAI_API_KEY is not set.\n\nRun:\n  export OPENAI_API_KEY=sk-...\n  cargo run\n\nOr create a .env file in the current directory.\n"
        );
        std::process::exit(1);
    }

    run_agent(built_in_tool_specs())
}
