"""CLI entry point for berg10-agent."""

import click
from dotenv import load_dotenv

from .config import AgentConfig


@click.command()
@click.option("--model", "-m", default=None, help="LLM model identifier")
@click.option("--host", "-h", default=None, help="Server bind host")
@click.option("--port", "-p", default=None, type=int, help="Server bind port")
@click.option("--work-dir", "-w", default=None, help="Working directory for file operations")
@click.option("--log-level", "-l", default=None, help="Log level (DEBUG/INFO/WARNING/ERROR)")
@click.option("--max-turns", default=None, type=int, help="Maximum agent turns per session")
@click.option("--tool-timeout", default=None, type=int, help="Tool execution timeout in seconds")
@click.option("--no-validators", is_flag=True, help="Disable input validators")
@click.option("--no-memory", is_flag=True, help="Disable .memory.md loading")
@click.option("--no-skills", is_flag=True, help="Disable AGENTS.md skills loading")
@click.option("--env-file", default=None, help="Path to .env file")
def main(
    model: str | None,
    host: str | None,
    port: int | None,
    work_dir: str | None,
    log_level: str | None,
    max_turns: int | None,
    tool_timeout: int | None,
    no_validators: bool,
    no_memory: bool,
    no_skills: bool,
    env_file: str | None,
) -> None:
    """Berg10 Agent - Headless LLM agent with WebSocket transport."""
    load_dotenv(env_file or ".env")

    config = AgentConfig.from_env(
        model=model,
        host=host,
        port=port,
        work_dir=work_dir,
        log_level=log_level,
        max_turns=max_turns,
        tool_timeout=tool_timeout,
        enable_validators=not no_validators if no_validators else None,
        enable_memory=not no_memory if no_memory else None,
        enable_skills=not no_skills if no_skills else None,
    )

    import uvicorn
    from .server import create_app

    app = create_app(config)
    uvicorn.run(app, host=config.host, port=config.port, log_level=config.log_level.lower())


if __name__ == "__main__":
    main()
