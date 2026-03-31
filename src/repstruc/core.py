from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


START_MARKER = "<!-- repstruc:start -->"
END_MARKER = "<!-- repstruc:end -->"
README_NAME = "README.md"
DEFAULT_MAX_DEPTH = 4
DEFAULT_IGNORED = {
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
}


@dataclass(frozen=True)
class UpdateResult:
    created: int
    updated: int
    skipped: int


def should_ignore(path: Path, ignored_names: set[str]) -> bool:
    return path.name in ignored_names


def iter_directories(root: Path, ignored_names: set[str]) -> list[Path]:
    directories = [root]
    for path in sorted(root.rglob("*")):
        if not path.is_dir():
            continue
        if any(part in ignored_names for part in path.parts):
            continue
        directories.append(path)
    return directories


def iter_children(path: Path, ignored_names: set[str]) -> list[Path]:
    children: list[Path] = []
    for child in path.iterdir():
        if should_ignore(child, ignored_names):
            continue
        children.append(child)
    return sorted(children, key=lambda child: (not child.is_dir(), child.name.lower()))


def build_tree(path: Path, ignored_names: set[str], prefix: str = "", depth: int = 0, max_depth: int = DEFAULT_MAX_DEPTH) -> list[str]:
    if depth >= max_depth:
        return []

    lines: list[str] = []
    children = iter_children(path, ignored_names)

    for index, child in enumerate(children):
        is_last = index == len(children) - 1
        branch = "└── " if is_last else "├── "
        label = f"{child.name}/" if child.is_dir() else child.name
        lines.append(f"{prefix}{branch}{label}")

        if child.is_dir():
            extension = "    " if is_last else "│   "
            lines.extend(build_tree(child, ignored_names, prefix + extension, depth + 1, max_depth))

    return lines


def render_structure_block(root: Path, ignored_names: set[str], max_depth: int) -> str:
    lines = [f"{root.name}/", *build_tree(root, ignored_names, max_depth=max_depth)]
    tree = "\n".join(lines)
    return (
        "## Repository Structure\n"
        f"{START_MARKER}\n"
        "```text\n"
        f"{tree}\n"
        "```\n"
        f"{END_MARKER}\n"
    )


def merge_readme_content(existing: str, block: str) -> tuple[str, bool]:
    normalized_existing = existing.rstrip()
    block_with_spacing = f"\n\n{block}".rstrip() + "\n"

    if START_MARKER in normalized_existing and END_MARKER in normalized_existing:
        start = normalized_existing.index(START_MARKER)
        section_start = normalized_existing.rfind("## Repository Structure", 0, start)
        if section_start == -1:
            section_start = start
        end = normalized_existing.index(END_MARKER) + len(END_MARKER)
        merged = f"{normalized_existing[:section_start].rstrip()}\n\n{block}".strip() + "\n"
    elif normalized_existing:
        merged = normalized_existing + block_with_spacing
    else:
        merged = block

    return merged, merged != existing


def update_readme_for_directory(directory: Path, ignored_names: set[str], max_depth: int) -> tuple[bool, bool]:
    readme_path = directory / README_NAME
    block = render_structure_block(directory, ignored_names, max_depth)

    if readme_path.exists():
        existing = readme_path.read_text(encoding="utf-8")
        merged, changed = merge_readme_content(existing, block)
        if changed:
            readme_path.write_text(merged, encoding="utf-8")
            return False, True
        return False, False

    readme_path.write_text(block, encoding="utf-8")
    return True, False


def update_repository(root: Path, ignored_names: set[str] | None = None, max_depth: int = DEFAULT_MAX_DEPTH) -> UpdateResult:
    ignored = set(DEFAULT_IGNORED if ignored_names is None else ignored_names)
    created = 0
    updated = 0
    skipped = 0

    for directory in iter_directories(root, ignored):
        was_created, was_updated = update_readme_for_directory(directory, ignored, max_depth)
        if was_created:
            created += 1
        elif was_updated:
            updated += 1
        else:
            skipped += 1

    return UpdateResult(created=created, updated=updated, skipped=skipped)
