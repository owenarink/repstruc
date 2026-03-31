from pathlib import Path

from repstruc.core import update_repository
from repstruc.core import START_MARKER


def test_creates_readme_for_root(tmp_path: Path) -> None:
    result = update_repository(tmp_path, max_depth=2)
    assert result.created == 1
    assert (tmp_path / "README.md").exists()


def test_preserves_existing_content_and_appends_structure(tmp_path: Path) -> None:
    readme = tmp_path / "README.md"
    readme.write_text("# Title\n\nSome text.\n", encoding="utf-8")

    update_repository(tmp_path, max_depth=2)

    updated = readme.read_text(encoding="utf-8")
    assert "# Title" in updated
    assert "Some text." in updated
    assert "## Repository Structure" in updated


def test_replaces_existing_managed_block(tmp_path: Path) -> None:
    readme = tmp_path / "README.md"
    readme.write_text(
        "# Title\n\nKeep this.\n\n## Repository Structure\n"
        f"{START_MARKER}\n```text\nold/\n```\n<!-- repstruc:end -->\n",
        encoding="utf-8",
    )

    update_repository(tmp_path, max_depth=2)

    updated = readme.read_text(encoding="utf-8")
    assert "# Title" in updated
    assert "Keep this." in updated
    assert "old/" not in updated
