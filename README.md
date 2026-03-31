# repstruc

`repstruc` keeps `README.md` files in sync with the real folder structure of a repository.

If a folder already has a `README.md`, `repstruc` preserves existing content and updates only the managed repository structure block.

If a folder does not have a `README.md`, `repstruc` creates one containing only the managed structure block.

## Features

- Recursively updates every folder in a repository
- Preserves existing README content outside the managed block
- Creates missing `README.md` files automatically
- Places the repository structure block before footer-style sections such as `Authors`, `Notes`, `Citations`, `References`, `License`, `Credits`, and similar end sections
- Can install a Git `pre-commit` hook so the structure updates automatically
- No external dependencies

## Install

### Homebrew

No separate tap repository is required. Install directly from the formula stored in this repository:

```bash
brew install https://raw.githubusercontent.com/owenarink/repstruc/v0.2.1/Formula/repstruc.rb
```

### pip

```bash
python3 -m pip install .
```

```bash
python3 -m pip install -e .
```

## Homebrew Formula

This repository includes a Homebrew formula at `Formula/repstruc.rb`.

To publish a new Homebrew-installable release from this same repository:

1. Tag and push the new version.
2. Compute the release tarball checksum.
3. Update `Formula/repstruc.rb` with the new release URL and `sha256`.
4. Users can then install directly from the raw formula URL for that release tag.

## Release Notes

### v0.2.1

- Removes the requirement for a separate Homebrew tap repository
- Documents direct Homebrew installation from `Formula/repstruc.rb` in this repository
- Keeps footer-aware placement of the repository structure block
- Cleans up the homepage README examples and publishing instructions

### v0.2.0

- Detects footer-style sections that belong at the end of an existing `README.md`
- Places `Repository Structure` before those trailing sections instead of after them
- Preserves the rest of the README content outside the managed block
- Adds clearer before-and-after examples

## TODO

- [ ] AI integration: train a model on heading types in GitHub `README.md` files
- [ ] AI integration: train a model to predict whether a heading or block of text belongs to the footer, so it should remain below the repository structure section
- [ ] Use that footer prediction system in the next version to improve structure placement beyond rule-based heading matching

Recognized trailing sections include:

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

Example folder before update:

```text
my-project/
├── src/
│   └── README.md
├── data.csv
└── README.md
```

Root `README.md` before update:

~~~~md
# My Project

Intro text.

## Authors
Jane Doe

## Citations
Paper A
~~~~

Root `README.md` after `repstruc update .`:

~~~~md
# My Project

Intro text.

## Authors
Jane Doe

## Citations
Paper A
~~~~

`src/README.md` after `repstruc update .`:

~~~~md
## Repository Structure
<!-- repstruc:start -->
```text
src/
└── README.md
```
<!-- repstruc:end -->
~~~~

If a folder has no `README.md`, `repstruc` creates one containing only the managed structure block for that folder.

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
