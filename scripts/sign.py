#!/usr/bin/env python3
"""
Ferrox Code Signing
Automates signing executables with signtool.exe and PKCS12 certificates.

Usage:
    python sign_binary.py <exe_path> <cert_path> [password]
"""

import sys
import os
import subprocess
from pathlib import Path

SIGNTOOL_PATHS = [
    r"C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.18362.0\x64\signtool.exe",
]


def find_signtool():
    """
    Locate signtool.exe on the system.

    Returns:
        str or None - Path to signtool.exe if found.
    """
    for path in SIGNTOOL_PATHS:
        if os.path.exists(path):
            return path

    kits = r"C:\Program Files (x86)\Windows Kits"
    if os.path.exists(kits):
        for root, _, files in os.walk(kits):
            if "signtool.exe" in files:
                return os.path.join(root, "signtool.exe")
    return None


def sign(exe_path, cert_path, password=None):
    """
    Sign an executable with a PKCS12 certificate.

    Args:
        exe_path: str - Path to the executable.
        cert_path: str - Path to the .p12/.pfx certificate.
        password: str or None - Certificate password.

    Returns:
        bool - True if signing succeeded.
    """
    if not os.path.exists(exe_path):
        print(f"Executable not found: {exe_path}")
        return False

    if not os.path.exists(cert_path):
        print(f"Certificate not found: {cert_path}")
        return False

    signtool = find_signtool()
    if not signtool:
        print("signtool.exe not found. Install Windows SDK.")
        return False

    cmd = [signtool, "sign", "/fd", "SHA256", "/f", cert_path]
    if password:
        cmd.extend(["/p", password])
    cmd.append(exe_path)

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=False)
        if result.returncode == 0:
            print(f"Signed: {exe_path}")
            return True
        print(f"Signing failed: {result.stderr or result.stdout}")
        return False
    except Exception as e:
        print(f"Signing error: {e}")
        return False


def verify(exe_path):
    """
    Verify the signature on an executable.

    Args:
        exe_path: str - Path to the signed executable.

    Returns:
        bool - True if signature is valid.
    """
    signtool = find_signtool()
    if not signtool:
        return False

    try:
        result = subprocess.run(
            [signtool, "verify", "/pa", exe_path],
            capture_output=True, text=True, check=False,
        )
        if result.returncode == 0:
            print(f"Signature verified: {exe_path}")
            return True
        print("Signature verification failed (expected for untrusted CA).")
        return False
    except Exception:
        return False


def main():
    """
    Entry point. Parse args, resolve password, sign, and verify.
    """
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <exe_path> <cert_path> [password]")
        sys.exit(1)

    exe_path = sys.argv[1]
    cert_path = sys.argv[2]
    password = sys.argv[3] if len(sys.argv) > 3 else None

    if not password:
        pw_file = Path(cert_path).parent / "password.txt"
        if pw_file.exists():
            pw = pw_file.read_text().strip()
            if pw and pw != "NO_PASSWORD_REQUIRED":
                password = pw

    if sign(exe_path, cert_path, password):
        verify(exe_path)
        size_mb = os.path.getsize(exe_path) / (1024 * 1024)
        print(f"Done. Size: {size_mb:.2f} MB")
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()
