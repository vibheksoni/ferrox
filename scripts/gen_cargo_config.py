#!/usr/bin/env python3
"""
Ferrox Cargo Config Generator
Generates .cargo/config.toml with path remapping resolved to the current user's environment.

Usage:
    python gen_cargo_config.py [--home HOME] [--project PROJECT]
"""

import argparse
import os
import sys
from pathlib import Path


def get_user_home():
    """
    Resolve the current user's home directory.

    Returns:
        str - Absolute path to user home (e.g. C:\\Users\\john).
    """
    return os.path.expanduser("~")


def get_project_root():
    """
    Resolve the ferrox project root (parent of scripts/).

    Returns:
        Path - Project root path.
    """
    return Path(__file__).resolve().parent.parent


def generate_config(home_override=None, project_override=None):
    """
    Build the .cargo/config.toml content with paths resolved for the current user.

    Args:
        home_override: str or None - Custom home directory path. Uses system home if None.
        project_override: str or None - Custom project root path. Uses detected root if None.

    Returns:
        str - Full TOML config content.
    """
    home = (home_override or get_user_home()).replace("/", "\\")
    project = (project_override or str(get_project_root())).replace("/", "\\")
    project_unix = (project_override or str(get_project_root())).replace("\\", "/")
    home_unix = home.replace("\\", "/")

    if home_unix.startswith("C:"):
        home_unix = "/c" + home_unix[2:]
    if project_unix.startswith("C:"):
        project_unix = "/c" + project_unix[2:]

    src_modules = [
        "wallet", "app", "docs", "communications",
        "fun/games", "fun", "proc/browser", "proc/lbrowser",
        "proc/system", "proc", "bin",
    ]

    src_files = [
        "fingerprint.rs", "detection.rs", "stealth.rs",
        "api_resolve.rs", "ntbridge.rs", "padding.rs",
        "polymorph.rs", "recon.rs", "cleanup.rs",
        "dissolve.rs", "ntcall.rs", "system_health.rs",
        "sprotect.rs", "win_internals.rs", "api_hash.rs",
        "main.rs", "lib.rs",
    ]

    lines = []
    lines.append("[build]")
    lines.append("jobs = 8")
    lines.append("rustflags = [")

    core_flags = [
        '    "-C", "strip=symbols",',
        '    "-C", "debuginfo=0",',
        '    "-C", "panic=abort",',
        '    "-Z", "location-detail=none",',
        '    "-C", "opt-level=z",',
        '    "-C", "codegen-units=1",',
        '    "-C", "lto=fat",',
        '    "-C", "force-frame-pointers=no",',
    ]
    lines.extend(core_flags)

    cargo_remaps = [
        f"{home}\\.cargo\\registry\\src\\index.crates.io-6f17d22bba15001f",
        f"{home}\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f",
        f"{home}\\.cargo\\registry\\src",
        f"{home}\\.cargo\\registry",
        f"{home}\\.cargo\\git",
        f"{home}\\.cargo",
        f"{home}\\.rustup\\toolchains",
        f"{home}\\.rustup",
    ]
    for r in cargo_remaps:
        escaped = r.replace("\\", "\\\\")
        lines.append(f'    "--remap-path-prefix", "{escaped}=.",')

    project_remaps = [
        project,
        str(Path(project).parent),
        str(Path(project).parent.parent),
        f"{home}\\AppData",
        home,
    ]
    seen = set()
    for r in project_remaps:
        if r not in seen:
            seen.add(r)
            escaped = r.replace("\\", "\\\\")
            lines.append(f'    "--remap-path-prefix", "{escaped}=.",')

    for mod in src_modules:
        fwd = mod
        bck = mod.replace("/", "\\\\")
        lines.append(f'    "--remap-path-prefix", "src/{fwd}=/dev/null",')
        lines.append(f'    "--remap-path-prefix", "src\\\\{bck}=/dev/null",')

    for f in src_files:
        lines.append(f'    "--remap-path-prefix", "src\\\\{f}=/dev/null",')
        lines.append(f'    "--remap-path-prefix", "src/{f}=/dev/null",')

    lines.append('    "--remap-path-prefix", "src/=/dev/null",')
    lines.append('    "--remap-path-prefix", "src\\\\=/dev/null",')

    project_parent_unix = str(Path(project_unix).parent).replace("\\", "/")
    unix_remaps = [
        f"{home_unix}/.cargo",
        f"{home_unix}/.rustup",
        project_unix,
        project_parent_unix,
        home_unix,
    ]
    for r in unix_remaps:
        lines.append(f'    "--remap-path-prefix", "{r}=/dev/null",')

    linker_flags = [
        '    "-C", "link-arg=/PDBALTPATH:/dev/null",',
        '    "-C", "link-arg=/DEBUG:NONE",',
        '    "-C", "link-arg=/PDBSTRIPPED",',
        '    "-C", "link-arg=/SUBSYSTEM:WINDOWS",',
        '    "-C", "link-arg=/ENTRY:mainCRTStartup",',
        '    "-C", "link-arg=/MANIFEST:NO"',
    ]
    lines.extend(linker_flags)
    lines.append("]")

    return "\n".join(lines) + "\n"


def main():
    """
    Generate and write .cargo/config.toml with resolved paths for the current user.
    Accepts optional --home and --project overrides via CLI.
    """
    parser = argparse.ArgumentParser(description="Generate .cargo/config.toml")
    parser.add_argument("--home", default=None, help="Override user home directory path.")
    parser.add_argument("--project", default=None, help="Override project root path.")
    args = parser.parse_args()

    real_project = get_project_root()
    config_dir = real_project / ".cargo"
    config_dir.mkdir(exist_ok=True)

    config_path = config_dir / "config.toml"
    content = generate_config(home_override=args.home, project_override=args.project)
    config_path.write_text(content, encoding="utf-8")
    print(f"Generated: {config_path}")


if __name__ == "__main__":
    main()
