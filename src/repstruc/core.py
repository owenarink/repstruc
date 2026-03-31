from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re


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
TRAILING_SECTION_TITLES = {
    "author",
    "authors",
    "notes",
    "note",
    "citations",
    "citation",
    "references",
    "reference",
    "bibliography",
    "works cited",
    "sources",
    "source",
    "acknowledgments",
    "acknowledgements",
    "credits",
    "credit",
    "contributors",
    "contributor",
    "contributing",
    "contribution",
    "license",
    "licence",
    "copyright",
    "disclaimer",
    "legal",
    "support",
    "contact",
    "contacts",
    "security",
    "changelog",
    "change log",
    "history",
    "appendix",
    "appendices",
    "faq",
    "footnotes",
    "further reading",
    "resources",
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


def normalize_heading_title(title: str) -> str:
    return re.sub(r"\s+", " ", title.strip().strip("#").strip()).lower()


def find_trailing_section_start(content: str) -> int | None:
    headings = list(re.finditer(r"(?m)^#{1,6}\s+(.+?)\s*$", content))
    if not headings:
        return None

    trailing_start: int | None = None
    for match in reversed(headings):
        title = normalize_heading_title(match.group(1))
        if title in TRAILING_SECTION_TITLES:
            trailing_start = match.start()
            continue
        break

    return trailing_start


def remove_existing_block(content: str) -> str:
    if START_MARKER not in content or END_MARKER not in content:
        return content

    start = content.index(START_MARKER)
    section_start = content.rfind("## Repository Structure", 0, start)
    if section_start == -1:
        section_start = start
    end = content.index(END_MARKER) + len(END_MARKER)
    return (content[:section_start].rstrip() + "\n\n" + content[end:].lstrip()).strip()


def merge_readme_content(existing: str, block: str) -> tuple[str, bool]:
    normalized_existing = existing.rstrip()
    if not normalized_existing:
        merged = block
        return merged, merged != existing

    base_content = remove_existing_block(normalized_existing)
    trailing_start = find_trailing_section_start(base_content)

    if trailing_start is None:
        merged = f"{base_content}\n\n{block}".strip() + "\n"
    else:
        leading = base_content[:trailing_start].rstrip()
        trailing = base_content[trailing_start:].lstrip()
        merged = f"{leading}\n\n{block}\n{trailing}".strip() + "\n"

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
