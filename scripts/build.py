#!/usr/bin/env python3
"""
Ferrox Build System
Smart build pipeline with error filtering, PE obfuscation, and build archiving.
"""

import subprocess
import sys
import os
import re
import glob
import shutil
import random
import string
import argparse
from datetime import datetime
from pathlib import Path

if sys.platform.startswith("win"):
    import codecs
    sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())
    sys.stderr = codecs.getwriter("utf-8")(sys.stderr.detach())


class FeroxBuilder:
    """
    Manages the Rust build process with error filtering and post-build steps.

    Attributes:
        project_dir: str - Path to the Rust project directory.
        build_logs_dir: str - Path to store build logs.
        use_32bit: bool - Whether to target i686 instead of x86_64.
        target: str - The Rust target triple.
        build_cmd: list - The cargo build command.
    """

    def __init__(self, project_dir=".", use_32bit=False):
        """
        Initialize the builder.

        Args:
            project_dir: str - Path to the Rust project root.
            use_32bit: bool - Build for 32-bit target.
        """
        self.project_dir = project_dir
        self.build_logs_dir = os.path.join(project_dir, "build_logs")
        self.use_32bit = use_32bit
        self.target = "i686-pc-windows-msvc" if use_32bit else "x86_64-pc-windows-msvc"
        self.build_cmd = [
            "cargo", "+nightly", "build",
            "--release",
            "--target", self.target,
        ]

    def setup(self):
        """
        Create build log directory and clean stale logs.
        """
        os.makedirs(self.build_logs_dir, exist_ok=True)
        self._cleanup_old_logs()

    def _cleanup_old_logs(self):
        """
        Remove old build artifacts, keeping only the 3 most recent error logs.
        """
        for pattern in ["build_errors_*.txt", "full_build_output.txt"]:
            for f in glob.glob(os.path.join(self.build_logs_dir, pattern)):
                try:
                    os.remove(f)
                except OSError:
                    pass

        error_files = sorted(
            glob.glob(os.path.join(self.build_logs_dir, "build_errors_*.txt")),
            key=os.path.getmtime,
            reverse=True,
        )
        for old in error_files[3:]:
            try:
                os.remove(old)
            except OSError:
                pass

    def run_build(self):
        """
        Execute the cargo build with RUSTFLAGS set for LTO compatibility.

        Returns:
            bool - True if the build succeeded.
        """
        self.setup()

        original_dir = os.getcwd()
        try:
            os.chdir(self.project_dir)

            pycrypt_path = os.path.join(".", "pycrypt.py")
            if os.path.exists(pycrypt_path):
                result = subprocess.run(
                    ["python", pycrypt_path, "src/lib.rs"],
                    capture_output=True, text=True, shell=True,
                )
                if result.returncode != 0:
                    print(f"pycrypt failed: {result.stderr}")
                    return False

            env = os.environ.copy()
            env["RUSTFLAGS"] = "-C embed-bitcode"

            result = subprocess.run(
                self.build_cmd,
                capture_output=True, text=True, shell=True, env=env,
            )

            self._process_output(result)
            return result.returncode == 0

        except Exception as e:
            print(f"Build failed: {e}")
            return False
        finally:
            os.chdir(original_dir)

    def _process_output(self, result):
        """
        Filter build output, display errors, and save logs.

        Args:
            result: subprocess.CompletedProcess - The build result.
        """
        output = result.stderr or result.stdout

        if result.returncode == 0:
            exe_path = f"./target/{self.target}/release/ferrox.exe"
            if os.path.exists(exe_path):
                size = os.path.getsize(exe_path)
                print(f"Build succeeded: {exe_path} ({size / 1024 / 1024:.2f} MB)")
            else:
                print("Build succeeded.")
            return

        full_path = os.path.join(self.build_logs_dir, "full_build_output.txt")
        with open(full_path, "w", encoding="utf-8") as f:
            f.write(output)

        errors = self._extract_errors(output)
        if errors:
            print(f"Build failed with {len(errors)} error(s):")
            for i, err in enumerate(errors, 1):
                print(f"\n--- Error #{i} ---")
                print(err.strip())
            self._save_error_log(errors)
        else:
            print("Build failed. Check full_build_output.txt for details.")

    def _extract_errors(self, output):
        """
        Parse compiler output and extract only real errors.

        Args:
            output: str - Raw compiler stderr/stdout.

        Returns:
            list[str] - List of error blocks.
        """
        lines = output.split("\n")
        errors = []
        error_patterns = [
            r"error\[E\d+\]:",
            r"error: aborting due to",
            r"error: could not compile",
            r"error: failed to run custom build command",
            r"error: linker.*failed",
        ]

        i = 0
        while i < len(lines):
            line = lines[i]
            is_error = any(re.search(p, line, re.IGNORECASE) for p in error_patterns)

            if not is_error and "error:" in line.lower():
                if not any(s in line.lower() for s in ["warning", "note:", "help:"]):
                    is_error = True

            if is_error:
                block = [line]
                j = i + 1
                while j < len(lines) and j < i + 10:
                    ctx = lines[j]
                    if any(s in ctx.lower() for s in ["error:", "warning:"]):
                        break
                    if any(c in ctx for c in ["-->", "|", "^", "="]):
                        block.append(ctx)
                    elif ctx.strip() and not ctx.startswith(" " * 10):
                        block.append(ctx)
                    elif not ctx.strip():
                        break
                    j += 1
                errors.append("\n".join(block))
                i = j
                continue
            i += 1

        return errors

    def _save_error_log(self, errors):
        """
        Write extracted errors to a timestamped log file.

        Args:
            errors: list[str] - List of error blocks.
        """
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        path = os.path.join(self.build_logs_dir, f"build_errors_{ts}.txt")
        with open(path, "w", encoding="utf-8") as f:
            for i, err in enumerate(errors, 1):
                f.write(f"Error #{i}:\n{err}\n\n")
        print(f"Errors saved to: {path}")


def archive_build(exe_path, builds_dir):
    """
    Copy the built executable into a timestamped archive folder with a randomized name.

    Args:
        exe_path: str - Path to the built executable.
        builds_dir: str - Root builds directory.

    Returns:
        str - Path to the archive folder.
    """
    os.makedirs(builds_dir, exist_ok=True)
    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    folder = os.path.join(builds_dir, ts)
    os.makedirs(folder)

    prefixes = ["Windows", "Microsoft", "System", "Display", "Audio", "Network", "Security", "Update"]
    middles = ["Driver", "Service", "Manager", "Helper", "Host", "Monitor", "Handler", "Component"]

    rand_name = f"{random.choice(prefixes)}{random.choice(middles)}.exe"

    shutil.copy2(exe_path, os.path.join(folder, "ferrox.exe"))
    shutil.copy2(exe_path, os.path.join(folder, rand_name))

    size_mb = os.path.getsize(exe_path) / (1024 * 1024)
    print(f"Archived: {folder} | ferrox.exe + {rand_name} ({size_mb:.2f} MB)")
    return folder


def main():
    """
    Entry point for the Ferrox build system.
    """
    parser = argparse.ArgumentParser(description="Ferrox Build System")
    parser.add_argument("--32bit", action="store_true", help="Build 32-bit (i686)")
    parser.add_argument("--pycrypt", action="store_true", help="Run pycrypt --full before building")
    parser.add_argument("--dir", default=".", help="Project directory")
    args = parser.parse_args()

    project_dir = args.dir
    use_32bit = args.__dict__["32bit"]

    if args.pycrypt:
        pycrypt_path = os.path.join(project_dir, "pycrypt.py")
        if not os.path.exists(pycrypt_path):
            print(f"pycrypt.py not found at {pycrypt_path}")
            sys.exit(1)
        r = subprocess.run(["python", "pycrypt.py", "--full"], capture_output=True, text=True, cwd=project_dir)
        if r.returncode != 0:
            print(f"pycrypt failed: {r.stderr}")
            sys.exit(1)

    builder = FeroxBuilder(project_dir=project_dir, use_32bit=use_32bit)
    success = builder.run_build()

    if not success:
        sys.exit(1)

    target = "i686-pc-windows-msvc" if use_32bit else "x86_64-pc-windows-msvc"
    exe_path = os.path.join(project_dir, "target", target, "release", "ferrox.exe")

    if os.path.exists(exe_path):
        pe_script = os.path.join(project_dir, "pe_obfuscate.py")
        if os.path.exists(pe_script):
            r = subprocess.run(["python", pe_script, exe_path], capture_output=True, text=True)
            if r.returncode == 0:
                print("PE obfuscation applied.")
            else:
                print(f"PE obfuscation failed: {r.stderr}")

        archive_build(exe_path, os.path.join(project_dir, "builds"))


if __name__ == "__main__":
    main()
