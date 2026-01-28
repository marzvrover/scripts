# dsv2md

Convert delimiter-separated values (DSV) to Markdown tables.

## Usage

```bash
# Basic usage (auto-detects CSV/TSV)
cat data.csv | ./dsv2md.py

# With header row
cat data.csv | ./dsv2md.py --header

# Pretty-print with aligned columns
cat data.csv | ./dsv2md.py --header --pretty

# Custom delimiter (supports multiple characters)
cat data.txt | ./dsv2md.py -d '::' --header
```

## Options

| Flag | Description |
|------|-------------|
| `--header` | Treat the first row as a header row. Without this flag, an empty header row is generated (required by Markdown table spec). |
| `-d`, `--delimiter` | Custom delimiter (supports multiple characters). Auto-detects CSV/TSV if not specified. |
| `-p`, `--pretty` | Align columns for readable plain text output. |

## Examples

### CSV with header

```bash
echo -e "Name,Age,City\nAlice,30,NYC\nBob,25,LA" | ./dsv2md.py --header
```

Output:
```
| Name | Age | City |
| --- | --- | --- |
| Alice | 30 | NYC |
| Bob | 25 | LA |
```

### Without header (empty header row)

```bash
echo -e "Alice,30,NYC\nBob,25,LA" | ./dsv2md.py
```

Output:
```
|  |  |  |
| --- | --- | --- |
| Alice | 30 | NYC |
| Bob | 25 | LA |
```

### Pretty-printed output

```bash
echo -e "Name,Age,City\nAlice,30,NYC\nBob,25,Los Angeles" | ./dsv2md.py --header --pretty
```

Output:
```
| Name  | Age | City        |
| ----- | --- | ----------- |
| Alice | 30  | NYC         |
| Bob   | 25  | Los Angeles |
```

### Custom multi-character delimiter

```bash
echo -e "Name::Age::City\nAlice::30::NYC" | ./dsv2md.py -d '::' --header
```

Output:
```
| Name | Age | City |
| --- | --- | --- |
| Alice | 30 | NYC |
```

## Requirements

Python 3.9+ (uses standard library only)

## Installation

```bash
chmod +x dsv2md.py
# Optionally symlink to PATH
ln -s "$(pwd)/dsv2md.py" ~/.local/bin/dsv2md
```
