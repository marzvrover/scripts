#!/usr/bin/env python3
"""Convert delimiter-separated values to a Markdown table."""

import argparse
import sys


def detect_delimiter(sample: str) -> str:
    """Detect whether input is TSV or CSV based on content."""
    tab_count = sample.count("\t")
    comma_count = sample.count(",")
    return "\t" if tab_count > comma_count else ","


def escape_pipes(cell: str) -> str:
    """Escape pipe characters in cell content."""
    return cell.replace("|", "\\|")


def parse_dsv(text: str, delimiter: str) -> list[list[str]]:
    """Parse delimiter-separated values with support for multi-character delimiters."""
    rows = []
    for line in text.strip().splitlines():
        rows.append(line.split(delimiter))
    return rows


def format_row(row: list[str], col_widths: list[int] | None = None) -> str:
    """Format a row as a Markdown table row."""
    escaped = [escape_pipes(cell.strip()) for cell in row]
    if col_widths:
        padded = [cell.ljust(width) for cell, width in zip(escaped, col_widths)]
        return "| " + " | ".join(padded) + " |"
    return "| " + " | ".join(escaped) + " |"


def format_separator(col_count: int, col_widths: list[int] | None = None) -> str:
    """Generate the Markdown header separator row."""
    if col_widths:
        dashes = ["-" * width for width in col_widths]
        return "| " + " | ".join(dashes) + " |"
    return "| " + " | ".join(["---"] * col_count) + " |"


def calculate_col_widths(rows: list[list[str]], has_header: bool) -> list[int]:
    """Calculate column widths based on content, minimum 3 for separator dashes."""
    col_count = len(rows[0]) if rows else 0
    widths = [3] * col_count

    for row in rows:
        for i, cell in enumerate(row):
            escaped = escape_pipes(cell.strip())
            widths[i] = max(widths[i], len(escaped))

    if not has_header:
        for i in range(col_count):
            widths[i] = max(widths[i], 0)

    return widths


def main():
    parser = argparse.ArgumentParser(
        description="Convert delimiter-separated values to Markdown table",
        epilog="Reads from stdin, writes to stdout",
    )
    parser.add_argument(
        "--header", action="store_true", help="Treat the first row as a header row"
    )
    parser.add_argument(
        "-d",
        "--delimiter",
        default=None,
        help="Custom delimiter (supports multiple characters). Auto-detects CSV/TSV if not specified.",
    )
    parser.add_argument(
        "-p",
        "--pretty",
        action="store_true",
        help="Align columns for readable plain text output",
    )
    args = parser.parse_args()

    input_text = sys.stdin.read()
    if not input_text.strip():
        sys.exit(0)

    delimiter = args.delimiter if args.delimiter else detect_delimiter(input_text)

    rows = parse_dsv(input_text, delimiter)

    if not rows:
        sys.exit(0)

    col_count = max(len(row) for row in rows)

    for row in rows:
        while len(row) < col_count:
            row.append("")

    col_widths = calculate_col_widths(rows, args.header) if args.pretty else None

    if args.header:
        print(format_row(rows[0], col_widths))
        print(format_separator(col_count, col_widths))
        for row in rows[1:]:
            print(format_row(row, col_widths))
    else:
        print(format_row([""] * col_count, col_widths))
        print(format_separator(col_count, col_widths))
        for row in rows:
            print(format_row(row, col_widths))


if __name__ == "__main__":
    main()
