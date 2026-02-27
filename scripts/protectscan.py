#!/usr/bin/env python3
"""
Ferrox String Protection Scanner
Scans Rust source files for unprotected string literals that should use sprotect!().

Usage:
    python protectscan.py [src_directory]
"""

import os
import re
import sys
from typing import List, NamedTuple


class Finding(NamedTuple):
    """
    An unprotected string literal found during scanning.

    Attributes:
        file_path: str - Path to the source file.
        line_number: int - Line where the string was found.
        char_pos: int - Character offset in the line.
        content: str - The raw string literal.
        context: str - Full line content for reference.
    """
    file_path: str
    line_number: int
    char_pos: int
    content: str
    context: str


IGNORE_PATTERNS = [
    re.compile(r'^[0-9a-fA-F]+$'),
    re.compile(r'^[\\/:.* ]+$'),
    re.compile(r'^[a-zA-Z]$'),
    re.compile(r'^\s*$'),
    re.compile(r'^[\[\]{}()]+$'),
    re.compile(r'^[,;:.]+$'),
    re.compile(r'^[\\\s]+$'),
    re.compile(r'^[._\-/\\]+$'),
    re.compile(r'^\\n$'),
    re.compile(r'^\\0$'),
]

FORMAT_ONLY_PATTERNS = [
    re.compile(r'^(\{\})+$'),
    re.compile(r'^[\{\}\s\\\\n\\\\t\\\\r]+$'),
    re.compile(r'^[\{\}=\\\\n]+$'),
    re.compile(r'^[\{\}:\s\\\\n]+$'),
    re.compile(r'^[\{\}\.\d:]+$'),
    re.compile(r'^[\{\}_\.\d:]+$'),
    re.compile(r'^[\{\}\\\\t]+$'),
    re.compile(r'^[\{\}\s,]+$'),
    re.compile(r'^\{:[a-z]\}$'),
    re.compile(r'^\s*-\s*[\{\}\\\\n]+$'),
]


class ProtectionScanner:
    """
    Scans Rust source files for string literals not wrapped in sprotect!().

    Attributes:
        src_dir: str - Directory to scan for .rs files.
        findings: List[Finding] - Collected unprotected strings.
    """

    RE_STRING = re.compile(r'"([^"\\]|\\.)*"')
    RE_PROTECTED = re.compile(r'sprotect!\s*\(\s*"([^"\\]|\\.)*"\s*\)')
    RE_COMMENT = re.compile(r'//.*$')
    RE_FORMAT = re.compile(r'format!\s*\(\s*("[^"]*?")')
    RE_INCLUDE = re.compile(r'include_bytes!\s*\(\s*("[^"]*")\s*\)')

    def __init__(self, src_dir=""):
        """
        Args:
            src_dir: str - Root directory to scan. Defaults to current directory.
        """
        self.src_dir = src_dir or "."
        self.findings: List[Finding] = []

    def _ranges(self, pattern, line):
        """
        Collect all match ranges for a pattern in a line.

        Args:
            pattern: re.Pattern - Compiled regex.
            line: str - Source line to search.

        Returns:
            List[tuple] - List of (start, end) tuples.
        """
        return [(m.start(), m.end()) for m in pattern.finditer(line)]

    def _in_range(self, pos, ranges):
        """
        Check if a position falls within any range.

        Args:
            pos: int - Character position.
            ranges: List[tuple] - (start, end) range tuples.

        Returns:
            bool - True if pos is inside any range.
        """
        return any(s <= pos < e for s, e in ranges)

    def _is_format_only(self, content):
        """
        Check if string is purely format placeholders with no meaningful text.

        Args:
            content: str - Inner string content without quotes.

        Returns:
            bool - True if only format template syntax.
        """
        if not content:
            return False
        for p in FORMAT_ONLY_PATTERNS:
            if p.match(content):
                return True
        if re.match(r'^[*`\{\}\\\\n\s]+$', content):
            return True
        stripped = re.sub(r'\{[^}]*\}', '', content)
        stripped = re.sub(r'\\\\[ntr]', '', stripped)
        if stripped and not any(c.isalpha() for c in stripped):
            return True
        return False

    def _should_ignore(self, content):
        """
        Check if a string is trivial and doesn't need protection.

        Args:
            content: str - Inner string content without quotes.

        Returns:
            bool - True if the string should be skipped.
        """
        if content in ['\\n', '\\0']:
            return True
        if not any(c.isalnum() for c in content):
            return True
        return any(p.match(content) for p in IGNORE_PATTERNS)

    def _scan_line(self, line, line_number, file_path):
        """
        Find unprotected string literals in a single line.

        Args:
            line: str - Source line to scan.
            line_number: int - Line number in file.
            file_path: str - Path to the file being scanned.
        """
        clean = self.RE_COMMENT.sub('', line)
        protected = self._ranges(self.RE_PROTECTED, clean)
        includes = [(m.start(1), m.end(1)) for m in self.RE_INCLUDE.finditer(clean)]

        for match in self.RE_STRING.finditer(clean):
            start = match.start()
            raw = match.group()

            if self._in_range(start, protected):
                continue
            if self._in_range(start, includes):
                continue
            if len(raw) <= 2:
                continue

            inner = raw[1:-1]
            if self._is_format_only(inner):
                continue
            if not inner.strip():
                continue
            if self._should_ignore(inner):
                continue

            self.findings.append(Finding(
                file_path=file_path,
                line_number=line_number,
                char_pos=start,
                content=raw,
                context=line.rstrip(),
            ))

    def scan_file(self, file_path):
        """
        Scan a single Rust file for unprotected strings.

        Args:
            file_path: str - Path to the .rs file.
        """
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                for num, line in enumerate(f, 1):
                    self._scan_line(line, num, file_path)
        except Exception as e:
            print(f"Error scanning {file_path}: {e}")

    def scan_directory(self):
        """
        Walk the src directory and scan all .rs files.
        """
        src_dir = os.path.join(self.src_dir, "src")
        if not os.path.exists(src_dir):
            print(f"Source directory not found: {src_dir}")
            return

        for root, _, files in os.walk(src_dir):
            for f in files:
                if f.endswith('.rs'):
                    self.scan_file(os.path.join(root, f))

    def report(self):
        """
        Print scan results grouped by file.

        Returns:
            int - Number of unprotected strings found.
        """
        if not self.findings:
            print("All strings protected.")
            return 0

        print(f"Found {len(self.findings)} unprotected strings")
        print("=" * 80)

        current_file = ""
        for r in sorted(self.findings, key=lambda x: (x.file_path, x.line_number)):
            if r.file_path != current_file:
                current_file = r.file_path
                print(f"\n{current_file}")
                print("-" * 60)

            safe = r.content.encode('ascii', 'replace').decode('ascii')
            ctx = r.context.strip().encode('ascii', 'replace').decode('ascii')
            print(f"  L{r.line_number:4d} | {safe}")
            print(f"         {ctx}")

        return len(self.findings)


def main():
    """
    Entry point. Accepts optional src directory as CLI argument.
    """
    src_dir = sys.argv[1] if len(sys.argv) > 1 else "."
    scanner = ProtectionScanner(src_dir)
    scanner.scan_directory()
    count = scanner.report()
    sys.exit(1 if count > 0 else 0)


if __name__ == "__main__":
    main()
