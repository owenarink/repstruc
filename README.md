# repstruc

`repstruc` is a small Python CLI that keeps `README.md` files in sync with the real folder structure of a repository.

If a folder already has a `README.md`, `repstruc` preserves the existing content and updates only the managed repository structure block at the end of the file.

If a folder does not have a `README.md`, `repstruc` creates one containing only the managed repository structure block.

## Features

- Recursively updates every folder in a repository
- Preserves existing README content outside the managed block
- Creates missing `README.md` files automatically
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

## Usage

Update all folders in the current repository:

```bash
repstruc .
```

Update a specific repository:

```bash
repstruc /path/to/target-repo
```

Update a specific folder name from your current location:

```bash
repstruc folder_name
```

Install the automatic Git hook:

```bash
repstruc . --install-hook
```

Then normal commits will refresh README structure blocks automatically before commit, so one `git push` is enough.

## Homebrew Formula

This repository includes a Homebrew formula at `Formula/repstruc.rb`.

To publish it properly:

1. Create a separate tap repository such as `homebrew-tap`.
2. Copy `Formula/repstruc.rb` into that tap.
3. Replace the release URL and `sha256` with the archive and checksum for your tagged GitHub release.
4. Users can then run `brew install owenarink/tap/repstruc`.

## Managed Block

`repstruc` manages this block:

```md

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
