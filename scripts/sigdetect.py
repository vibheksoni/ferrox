#!/usr/bin/env python3
"""
Ferrox Signature Detector
Static analysis tool that scans Rust source and compiled binaries
for patterns commonly flagged by AV engines.
"""

import os
import re
import sys
from pathlib import Path
from collections import defaultdict

SUSPICIOUS_PATTERNS = {
    "CREDENTIAL_THEFT": [
        r'"Login Data"',
        r'"Cookies"',
        r'"Web Data"',
        r'Chrome.*LocalState',
        r'Edge.*LocalState',
        r'discord.*Local Storage',
        r'CryptUnprotectData',
        r'sqlite.*passwords',
    ],
    "BROWSER_PATHS": [
        r'\\Google\\Chrome\\User Data',
        r'\\Microsoft\\Edge\\User Data',
        r'\\BraveSoftware\\Brave-Browser',
        r'\\Discord\\Local Storage',
        r'\\Telegram Desktop',
    ],
    "WALLET_TARGETS": [
        r'Exodus',
        r'Electrum',
        r'Atomic',
        r'\\Ethereum\\keystore',
        r'MetaMask',
        r'phantom\.wallet',
    ],
    "PERSISTENCE_METHODS": [
        r'CurrentVersion\\Run',
        r'schtasks.*create',
        r'bitsadmin',
        r'WMI.*EventFilter',
        r'wmic.*process call create',
        r'Startup.*\.lnk',
    ],
    "EVASION_TECHNIQUES": [
        r'VirtualBox',
        r'VMware',
        r'IsDebuggerPresent',
        r'CheckRemoteDebuggerPresent',
        r'CPUID.*hypervisor',
        r'sandbox',
    ],
    "NETWORK_EXFILTRATION": [
        r'api\.telegram\.org',
        r'sendDocument',
        r'discord\.com/api/webhooks',
        r'pastebin\.com',
        r'curl.*-F.*document',
    ],
    "SUSPICIOUS_APIS": [
        r'CreateRemoteThread',
        r'WriteProcessMemory',
        r'VirtualAllocEx',
        r'NtWriteVirtualMemory',
        r'NtCreateThreadEx',
        r'Process32First',
        r'CreateToolhelp32Snapshot',
    ],
    "ANTI_DEBUG": [
        r'NtQueryInformationProcess',
        r'debugger',
        r'ollydbg',
        r'x64dbg',
        r'ida.*pro',
    ],
    "SUSPICIOUS_STRINGS": [
        r'password',
        r'token',
        r'wallet',
        r'seed.*phrase',
        r'private.*key',
        r'credit.*card',
    ],
    "SELF_MODIFICATION": [
        r'attrib.*\+h',
        r'self.*delete',
        r'del /f /q',
        r'copy.*%APPDATA%',
    ],
    "CRYPTO_OPERATIONS": [
        r'AES',
        r'ChaCha20',
        r'encrypt',
        r'decrypt',
        r'XOR.*payload',
    ],
    "TELEGRAM_SPECIFIC": [
        r'bot[0-9]+:[A-Za-z0-9_-]+',
        r'chat_id.*=.*[0-9]+',
        r'/sendDocument',
        r'/sendMessage',
    ],
    "REGISTRY_OPERATIONS": [
        r'RegSetValue',
        r'RegCreateKey',
        r'SOFTWARE\\\\Microsoft\\\\Windows',
        r'HKEY_CURRENT_USER',
    ],
    "FILE_OPERATIONS": [
        r'C:\\\\temp\\\\extract',
        r'%APPDATA%',
        r'%TEMP%',
        r'\.db$',
        r'\.dat$',
    ],
}

STEALER_FUNCTIONS = [
    'extract.*password',
    'steal.*cookie',
    'grab.*token',
    'collect.*wallet',
    'harvest.*data',
    'upload.*harvest',
    'exfiltrate',
]

YARA_RULES = {
    "STEALER_COMBO_1": {
        "description": "Browser + Network Upload Pattern",
        "requires_all": [r'"Login Data"', r'api\.telegram\.org', r'sendDocument'],
    },
    "STEALER_COMBO_2": {
        "description": "Credential Theft + Persistence",
        "requires_all": [r'CryptUnprotectData', r'CurrentVersion\\Run'],
    },
    "STEALER_COMBO_3": {
        "description": "Multi-Browser Targeting",
        "requires_count": 3,
        "patterns": [r'Chrome', r'Edge', r'Brave', r'Firefox', r'Opera'],
    },
    "RANSOMWARE_PATTERN": {
        "description": "File Encryption + Network Contact",
        "requires_all": [r'encrypt', r'AES|ChaCha', r'http.*://|api\.'],
    },
}


class SignatureDetector:
    """
    Scans Rust source files and binaries for AV-flagged patterns.

    Attributes:
        source_dir: Path - Directory containing .rs files.
        findings: dict - Pattern matches grouped by category.
        file_scores: dict - Risk score per file.
        total_score: int - Aggregate risk score.
    """

    def __init__(self, source_dir):
        """
        Args:
            source_dir: Path - Root source directory to scan.
        """
        self.source_dir = Path(source_dir)
        self.findings = defaultdict(list)
        self.file_scores = defaultdict(int)
        self.total_score = 0

    def scan_file(self, file_path):
        """
        Scan a single Rust file against all pattern categories.

        Args:
            file_path: Path - The .rs file to scan.
        """
        try:
            content = file_path.read_text(encoding="utf-8")
            rel = file_path.relative_to(self.source_dir)
        except Exception:
            return

        for category, patterns in SUSPICIOUS_PATTERNS.items():
            for pattern in patterns:
                matches = re.findall(pattern, content, re.IGNORECASE)
                if matches:
                    self.findings[category].append({
                        "file": str(rel),
                        "pattern": pattern,
                        "count": len(matches),
                    })
                    self.file_scores[str(rel)] += len(matches)
                    self.total_score += len(matches)

        for func_pat in STEALER_FUNCTIONS:
            matches = re.findall(rf"fn {func_pat}", content, re.IGNORECASE)
            if matches:
                self.findings["SUSPICIOUS_FUNCTIONS"].append({
                    "file": str(rel),
                    "functions": matches,
                })
                self.file_scores[str(rel)] += len(matches) * 2
                self.total_score += len(matches) * 2

    def check_yara_rules(self):
        """
        Evaluate YARA-like combo rules across the entire source tree.

        Returns:
            list[dict] - Matched rules with severity.
        """
        combined = ""
        for rs in self.source_dir.rglob("*.rs"):
            try:
                combined += rs.read_text(encoding="utf-8") + "\n"
            except Exception:
                pass

        matches = []
        for name, rule in YARA_RULES.items():
            if "requires_all" in rule:
                if all(re.search(p, combined, re.IGNORECASE) for p in rule["requires_all"]):
                    matches.append({"rule": name, "description": rule["description"], "severity": "HIGH"})
            elif "requires_count" in rule:
                hit = sum(1 for p in rule["patterns"] if re.search(p, combined, re.IGNORECASE))
                if hit >= rule["requires_count"]:
                    matches.append({"rule": name, "description": rule["description"], "severity": "MEDIUM", "matches": hit})
        return matches

    def _risk_level(self, score):
        """
        Map a numeric score to a risk label.

        Args:
            score: int - Aggregate signature score.

        Returns:
            str - Risk level string.
        """
        if score > 200:
            return "CRITICAL"
        if score > 100:
            return "HIGH"
        if score > 50:
            return "MEDIUM"
        if score > 20:
            return "LOW"
        return "CLEAN"

    def scan_all(self):
        """
        Scan every .rs file under source_dir.
        """
        for rs in self.source_dir.rglob("*.rs"):
            self.scan_file(rs)

    def report(self):
        """
        Print a summary of findings, YARA matches, and top risky files.
        """
        print(f"Score: {self.total_score} | Risk: {self._risk_level(self.total_score)}")
        print()

        for cat, items in sorted(self.findings.items(), key=lambda x: len(x[1]), reverse=True):
            if not items:
                continue
            print(f"[{cat}] {len(items)} match(es)")
            by_file = defaultdict(list)
            for item in items:
                by_file[item["file"]].append(item)
            for f, fi in sorted(by_file.items(), key=lambda x: len(x[1]), reverse=True):
                print(f"  {f} ({len(fi)} patterns)")

        yara = self.check_yara_rules()
        if yara:
            print()
            for m in yara:
                print(f"[{m['severity']}] {m['rule']}: {m['description']}")

        top = sorted(self.file_scores.items(), key=lambda x: x[1], reverse=True)[:10]
        if top:
            print()
            print("Top risky files:")
            for i, (f, s) in enumerate(top, 1):
                print(f"  {i}. {f} (score: {s})")


def scan_binary(binary_path):
    """
    Scan a compiled binary for suspicious plaintext strings.

    Args:
        binary_path: str - Path to the PE executable.
    """
    try:
        data = Path(binary_path).read_bytes()
    except Exception as e:
        print(f"Could not read binary: {e}")
        return

    checks = {
        b"api.telegram.org": "Telegram API",
        b"sendDocument": "Telegram upload",
        b"Login Data": "Browser credentials",
        b"Chrome\\User Data": "Chrome path",
        b"CurrentVersion\\Run": "Registry persistence",
        b"schtasks": "Scheduled tasks",
        b"C:\\temp\\extract": "Hardcoded temp path",
    }

    found = [(label, tag) for tag, label in checks.items() if tag in data]
    if found:
        print(f"Binary flagged strings ({len(found)}):")
        for label, _ in found:
            print(f"  - {label}")
    else:
        print("Binary clean - no obvious strings found.")


def main():
    """
    Entry point. Scans src/ for patterns and optionally a compiled binary.
    """
    source_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("./src")

    if not source_dir.exists():
        print(f"Source directory not found: {source_dir}")
        sys.exit(1)

    detector = SignatureDetector(source_dir)
    detector.scan_all()
    detector.report()

    if len(sys.argv) > 2:
        print()
        scan_binary(sys.argv[2])


if __name__ == "__main__":
    main()
