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

From the project root:

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
repstruc update .
```

Install the automatic Git hook in another repository:

```bash
cd /path/to/target-repo
repstruc install-hook .
```

Then normal commits will refresh README structure blocks automatically before commit.

## Managed Block

`repstruc` manages this block:

```md
## Repository Structure
<!-- repstruc:start -->
```text
...
```
<!-- repstruc:end -->
```

Anything outside that block is preserved.
