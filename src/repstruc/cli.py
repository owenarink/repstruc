from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

from repstruc.core import DEFAULT_IGNORED, update_repository


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="repstruc")
    parser.add_argument("path", nargs="?", default=".", help="Target repository path.")
    parser.add_argument("--max-depth", type=int, default=4, help="Maximum tree depth to render.")
    parser.add_argument(
        "--ignore",
        action="append",
        default=[],
        help="Directory or file name to ignore. Can be provided multiple times.",
    )
    parser.add_argument(
        "--install-hook",
        action="store_true",
        help="Install a pre-commit hook in the target repository.",
    )
    parser.add_argument(
        "--hook-command",
        default="repstruc .",
        help="Command executed by the installed hook.",
    )
    return parser


def ensure_git_repository(path: Path) -> None:
    result = subprocess.run(
        ["git", "rev-parse", "--git-dir"],
        cwd=path,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(f"{path} is not a Git repository.")


def install_hook(path: Path, command: str) -> None:
    ensure_git_repository(path)
    hooks_dir = path / ".git" / "hooks"
    hooks_dir.mkdir(parents=True, exist_ok=True)
    hook_path = hooks_dir / "pre-commit"
    hook_body = (
        "#!/bin/sh\n\n"
        "set -eu\n\n"
        f"{command}\n"
        "git add -A .\n"
    )
    hook_path.write_text(hook_body, encoding="utf-8")
    hook_path.chmod(0o755)
    print(f"Installed pre-commit hook at {hook_path}")


def run_update(path: Path, max_depth: int, ignores: list[str]) -> None:
    if not path.exists():
        raise SystemExit(f"Path does not exist: {path}")
    if not path.is_dir():
        raise SystemExit(f"Path is not a directory: {path}")

    ignore_set = set(DEFAULT_IGNORED)
    ignore_set.update(ignores)
    result = update_repository(path, ignored_names=ignore_set, max_depth=max_depth)
    print(
        f"Updated README files in {path}: "
        f"created={result.created} updated={result.updated} skipped={result.skipped}"
    )


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    target = Path(args.path).resolve()
    run_update(target, args.max_depth, args.ignore)
    if args.install_hook:
        install_hook(target, args.hook_command)


if __name__ == "__main__":
    main()
