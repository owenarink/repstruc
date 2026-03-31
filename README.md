# repstruc

`repstruc` is a small Python CLI that keeps `README.md` files in sync with the real folder structure of a repository.

If a folder already has a `README.md`, `repstruc` preserves the existing content and updates only the managed repository structure block at the end of the file.

If a folder does not have a `README.md`, `repstruc` creates one containing only the managed repository structure block.

## Features

- Recursively updates every folder in a repository
- Preserves existing README content outside the managed block
- Creates missing `README.md` files automatically
- Keeps the repository structure block ahead of footer-style sections such as Authors, Notes, Citations, References, License, Credits, and similar end sections
- Can install a Git `pre-commit` hook so the structure updates automatically
- No external dependencies

## Install

### Homebrew

After publishing a tagged release and formula, users should be able to install with:

```bash
brew install owenarink/tap/repstruc
```

### pip

```bash
python3 -m pip install .
```

Or install in editable mode while developing:

```bash
python3 -m pip install -e .
```

## Homebrew Formula

This repository includes a Homebrew formula at `Formula/repstruc.rb`.

To publish it properly:

1. Create a separate tap repository such as `homebrew-tap`.
2. Copy `Formula/repstruc.rb` into that tap.
3. Replace the release URL and `sha256` with the archive and checksum for your tagged GitHub release.
4. Users can then run `brew install owenarink/tap/repstruc`.

## Release Notes

### v0.2.0

- Detects footer-style sections that belong at the end of an existing `README.md`
- Places `## Repository Structure` before those trailing sections instead of after them
- Preserves the rest of the README content outside the managed block

Recognized trailing sections now include:

- `Authors`
- `Author`
- `Notes`
- `Note`
- `Citations`
- `Citation`
- `References`
- `Reference`
- `Bibliography`
- `Works Cited`
- `Sources`
- `Source`
- `Acknowledgments`
- `Acknowledgements`
- `Credits`
- `Credit`
- `Contributors`
- `Contributor`
- `Contributing`
- `Contribution`
- `License`
- `Licence`
- `Copyright`
- `Disclaimer`
- `Legal`
- `Support`
- `Contact`
- `Contacts`
- `Security`
- `Changelog`
- `Change Log`
- `History`
- `Appendix`
- `Appendices`
- `FAQ`
- `Footnotes`
- `Further Reading`
- `Resources`

## Examples

Existing README before update:

```md
# My Project

Intro text.

## Authors
Jane Doe

## Citations
Paper A
```

README after `repstruc .`:

```md
# My Project

Intro text.

## Authors
Jane Doe

## Citations
Paper A
```

If a folder has no `README.md`, `repstruc` creates one containing only the managed structure block.

## Managed Block

`repstruc` manages this block:

~~~~md
## Repository Structure
<!-- repstruc:start -->
```text
my-project/
├── src/
└── README.md
```
<!-- repstruc:end -->
~~~~

## Repository Structure
<!-- repstruc:start -->
```text
repstruc/
├── Formula/
│   ├── README.md
│   └── repstruc.rb
├── src/
│   ├── repstruc/
│   │   ├── __init__.py
│   │   ├── cli.py
│   │   ├── core.py
│   │   └── README.md
│   └── README.md
├── tests/
│   ├── README.md
│   └── test_core.py
├── .gitignore
├── LICENSE
├── pyproject.toml
└── README.md
```
<!-- repstruc:end -->
