from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

from repstruc.core import DEFAULT_IGNORED, update_repository


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="repstruc")
    subparsers = parser.add_subparsers(dest="command", required=True)

    update_parser = subparsers.add_parser("update", help="Update README files in a repository.")
    update_parser.add_argument("path", nargs="?", default=".", help="Target repository path.")
    update_parser.add_argument("--max-depth", type=int, default=4, help="Maximum tree depth to render.")
    update_parser.add_argument(
        "--ignore",
        action="append",
        default=[],
        help="Directory or file name to ignore. Can be provided multiple times.",
    )

    hook_parser = subparsers.add_parser("install-hook", help="Install a pre-commit hook in a repository.")
    hook_parser.add_argument("path", nargs="?", default=".", help="Target repository path.")
    hook_parser.add_argument("--hook-command", default="repstruc update .", help="Command executed by the hook.")

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

    if args.command == "update":
        run_update(target, args.max_depth, args.ignore)
        return

    if args.command == "install-hook":
        install_hook(target, args.hook_command)
        return

    parser.print_help()
    raise SystemExit(1)


if __name__ == "__main__":
    main()
