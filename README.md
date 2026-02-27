# Ferrox

> [!WARNING]
> **RESEARCH AND EDUCATIONAL PURPOSE ONLY**
>
> This project was developed exclusively for security research and academic study of modern stealer techniques. All testing was conducted in isolated virtual machine environments against Windows Defender only. **Critical components have been intentionally removed from this codebase** - this is NOT a functional stealer out of the box.
>
> - Written in **Rust** (not Go as commonly seen in malware)
> - Successfully evaded Windows Defender in controlled VM testing
> - Demonstrates modern evasion techniques: Hell's Gate syscalls, polymorphic builds, compile-time encryption
> - **Intended for learning purposes only** - understand how modern threats operate to build better defenses
>
> **Unauthorized use is illegal and unethical.** This code is provided for security researchers, malware analysts, and defensive security professionals to study modern attack techniques and improve detection capabilities.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows%2011-blue.svg)
![Stars](https://img.shields.io/github/stars/vibheksoni/ferrox?style=social)
![Forks](https://img.shields.io/github/forks/vibheksoni/ferrox?style=social)

> Windows infostealer written in Rust with polymorphic builds, compile-time string encryption, and direct syscall execution. Every build produces a unique binary.

**Keywords:** rust malware, windows stealer, polymorphic code, hell's gate syscalls, edr evasion, av bypass, compile-time encryption, anti-analysis, red team tools, offensive security

Ferrox is a Windows-targeted stealer that harvests browser credentials, cryptocurrency wallets, messaging app sessions, gaming platform tokens, documents, and system information. It exfiltrates stolen data over Telegram using encrypted blobs. Built with extensive anti-analysis and evasion techniques to avoid detection by AV/EDR products.

## What It Does

Ferrox runs silently (no console window) and executes the following pipeline:

1. Ensures single instance via global mutex
2. Runs anti-VM, anti-debugger, and sandbox detection - silently exits if analysis environment detected
3. Generates legitimate-looking network traffic to blend in
4. Fingerprints the device (CPU, motherboard, disk serials hashed with SHA-256)
5. Collects system recon (external IP, network adapters, saved WiFi passwords, Windows product key, installed AV, domain info)
6. Harvests data from all targets (browsers, wallets, apps, games, documents)
7. Compresses and encrypts all stolen data (ChaCha20)
8. Uploads encrypted blob to Telegram via bot API
9. Cleans up and exits

## Harvest Targets

### Browsers
All Chromium-based browsers (Chrome, Edge, Brave, Opera, Vivaldi, and more) and Gecko-based browsers (Firefox, Waterfox, Pale Moon, etc.). Extracts:
- Saved passwords (Login Data) - includes Chrome App-Bound Encryption decryption (credit: [xaitax/Chrome-App-Bound-Encryption-Decryption](https://github.com/xaitax/Chrome-App-Bound-Encryption-Decryption))
- Cookies
- Autofill data (Web Data)
- Browsing history
- Credit card data
- Browser extensions (crypto wallet extensions targeted specifically)

### Cryptocurrency Wallets
**Desktop apps:** Exodus, Electrum, Atomic, Coinomi, Guarda, Bitcoin Core, Litecoin Core, Ethereum (Geth), Monero, Zcash, Dash, Dogecoin, Wasabi Wallet, Jaxx, MultiBit, Armory, and more.

**Browser extensions:** MetaMask, Phantom, Trust Wallet, Coinbase Wallet, and others.

### Messaging Apps
Discord (+ Canary, PTB, Lightcord), Telegram Desktop (tdata, session files, key files), WhatsApp, Signal, Viber, Element (Matrix), Skype, ICQ, Pidgin, Tox, TeamSpeak, Slack, Zoom.

### Gaming Platforms
Steam (SSFN auth files, config, userdata), Discord tokens, Minecraft sessions, Epic Games Store, Riot Games (League/Valorant), Roblox, Battle.net, Ubisoft Connect, Origin/EA, NationsGlory.

### Documents & Keys
SSH keys, SSL certificates, PGP keys (.gnupg), VPN configs (OpenVPN, NordVPN, ProtonVPN, ExpressVPN), crypto wallet files, keystore/seed/mnemonic/recovery files from Documents/Desktop/Downloads.

### System Info
External IP, network adapters, saved WiFi passwords (plaintext), Windows product key, installed antivirus, domain membership.

## Evasion & Anti-Analysis

### Anti-VM Detection
- CPUID hypervisor bit check + brand string matching (VMware, KVM, VirtualBox, Hyper-V, QEMU, Xen)
- MAC address prefix matching against known VM vendors
- WMI hardware queries (BIOS manufacturer, computer model, disk model)
- Registry checks for VM artifacts
- Process scanning for VM tools (vmtoolsd, vboxservice, qemu-ga, etc.)
- All detection strings are XOR-encrypted with a runtime random key

### Anti-Debugger
- PEB `IsDebuggerPresent` check via hashed API resolution
- RDTSC timing check - detects single-stepping by measuring CPU cycles for trivial operations

### Sandbox Evasion
- Checks for suspicious parent processes (python, wireshark, fiddler, procmon, x64dbg, ollydbg, ida)
- Low resource detection (RAM < 4GB, CPU < 2 cores)
- Recent boot detection (uptime < 10 minutes)
- Time acceleration detection - catches sandbox fast-forwarding by comparing expected vs actual sleep duration
- Execution inflation - performs 15-30 rounds of legitimate-looking operations (reading system DLLs, registry queries, prime calculations) to waste sandbox analysis time

### Direct Syscalls (Hell's Gate)
Bypasses userland API hooks installed by EDR/AV products:
- Locates NTDLL base address from PEB via `gs:[0x60]` - no API calls
- Walks InMemoryOrderModuleList to find ntdll.dll
- Parses PE export table, reads `mov eax, <SSN>` from function prologues to extract System Service Numbers
- Executes raw `syscall` instructions via inline assembly
- Resolved syscalls: NtAllocateVirtualMemory, NtWriteVirtualMemory, NtProtectVirtualMemory, NtCreateThreadEx, NtOpenProcess, NtClose, NtQuerySystemInformation, NtCreateFile, NtReadFile, NtWriteFile
- All SSNs cached in a global `OnceLock` singleton

### API Hashing
Zero import table entries for sensitive APIs:
- DJB2 hash with randomized key per build (1000-10000 range)
- Walks PEB -> Ldr -> InMemoryOrderModuleList to find loaded modules
- Parses PE export tables to resolve functions by hash
- `api_hash!()` proc macro generates hashes at compile time
- `api_resolve!()` macro resolves at runtime

### String Encryption
Every sensitive string in the binary is encrypted at compile time:
- `sprotect!()` proc macro encrypts with AES-256-GCM during compilation
- Key derived from 32 constants disguised as innocent names (NETWORK_TIMEOUT_MS, BUFFER_ALLOCATION_SIZE, etc.)
- Unique random nonce per string
- Decrypted at runtime on first access
- `VolatileString` wrapper zeros memory on drop using `write_volatile` + random overwrite + zero again

### Process Injection (Defender Killer)
Injects an encrypted DLL into SYSTEM-level processes to disable Windows Defender:
- Embedded AES-256-GCM encrypted DLL payload at compile time
- Enables SeDebugPrivilege to access privileged processes
- Targets: winlogon.exe, services.exe, lsass.exe, csrss.exe
- Opens target with PROCESS_ALL_ACCESS via NtOpenProcess syscall
- Allocates RWX memory via NtAllocateVirtualMemory syscall
- Writes decrypted DLL via NtWriteVirtualMemory syscall
- Creates remote thread via NtCreateThreadEx syscall
- All injection steps use direct syscalls to bypass EDR hooks

### Anti-Forensics
- Self-deletion via temp .bat file that loops killing the process and deleting the exe
- File overwriting with null bytes before deletion to prevent recovery
- Self-corruption on VM detection - spawns hidden PowerShell to overwrite exe with random bytes
- Handle manipulation via NtQuerySystemInformation to unlock own executable while running
- Volatile strings zero memory on drop

### Polymorphic Builds
Every compilation produces a unique binary. The `polymorph.py` script mutates source code before building:
- Injects random dead functions into source files (70% probability per file)
- Randomizes timing constants - sleep/jitter durations (80% probability)
- Mutates numeric constants (50% probability)
- Generates unique AES-256-GCM key for string encryption
- Generates unique DJB2 hash key for API hashing
- Restores original source files after build completes

**Marker system** - source files contain comment markers that the builder transforms:

```rust
//#ultraprotect(name="API_KEY", value="default_value")
// Builder generates: lazy_static! { pub static ref API_KEY: String = sprotect!("randomized"); }
//#endultra()

//#junk(name="my_junk")
// Builder generates random dead code functions here
//#endjunk()

//#jcall(name="my_junk")
// Builder inserts call to generated junk function

//#polymorphnop(intensity="light|heavy")
// Builder inserts NOP sled

//#stackjunk(vars=N)
// Builder inserts N unused stack variables

//#opaqueif()
// Builder wraps next block in opaque predicate
//#endopaque()
```

**Runtime polymorphism** (`polymorph.rs`):
- NOP sleds using various x86 NOP-equivalent instructions (xor eax eax, lea rax [rax], push/pop, xchg, pause)
- Opaque predicates - always-true conditions using volatile reads and `black_box` to prevent optimization
- Stack pollution with random values
- Polymorphic sleep using randomly chosen methods (thread::sleep, spin loop, fragmented sleep)
- Control flow confusion combining all techniques

## Exfiltration

All stolen data is exfiltrated over Telegram:
- Data is collected to `C:\temp\extract\`
- ZIP compressed, then encrypted with ChaCha20
- Wrapped in a custom binary blob format with embedded key/nonce/password
- Random filenames like `update_Xk3mP9qR.dat`
- Uploaded via Telegram Bot API `sendDocument` endpoint
- Supports chunked uploads (45MB chunks) for large datasets
- Randomized User-Agent strings
- Retry logic with exponential backoff and rate limit handling
- Startup notification disguised as "Windows Update Service" / "Telemetry service"
- Harvest summary sent with password/cookie/card counts

## Project Structure

```
ferrox/
├── src/
│   ├── main.rs              # Execution pipeline
│   ├── lib.rs               # sprotect! proc macro (AES-256-GCM)
│   ├── api_hash.rs          # DJB2 API hashing macros
│   ├── api_resolve.rs       # Runtime API resolution via PEB
│   ├── ntcall.rs            # Direct syscall execution
│   ├── ntbridge.rs          # Hell's Gate syscall cache
│   ├── detection.rs         # VM/debugger/sandbox detection
│   ├── stealth.rs           # Jitter macros, legit traffic generation
│   ├── polymorph.rs         # Runtime NOP sleds, opaque predicates
│   ├── sprotect.rs          # VolatileString (auto-zeroing)
│   ├── win_internals.rs     # PEB walking, module/export parsing
│   ├── system_health.rs     # Defender killer DLL injection
│   ├── fingerprint.rs       # Device ID generation + registry persistence
│   ├── recon.rs             # System recon (IP, WiFi passwords, AV, etc.)
│   ├── cleanup.rs           # Handle manipulation, self-corruption
│   ├── dissolve.rs          # Self-deletion
│   ├── padding.rs           # Sandbox time wasting
│   ├── proc/
│   │   ├── browser/         # Chromium browser theft
│   │   ├── lbrowser/        # Firefox/Gecko browser theft
│   │   └── system/          # System resource theft
│   ├── wallet/              # Crypto wallet theft (desktop + extensions)
│   ├── app/                 # Messaging app session theft
│   ├── fun/                 # Gaming platform token theft
│   ├── docs/                # Document/key/cert theft
│   └── communications/      # Telegram C2 + encrypted blob format
├── scripts/
│   ├── polymorph.py         # Polymorphic build engine
│   ├── build.py             # Build pipeline orchestrator
│   └── gen_cargo_config.py  # Cargo config with path remapping
├── api-hash-macro/          # Proc macro crate for API hashing
└── .cargo/
    └── config.toml          # Aggressive optimization + symbol stripping
```

## Requirements

- Rust nightly toolchain (REQUIRED - stable will not work)
- Windows 11 (tested on Build 26100)
- Python 3.8+ (for build scripts)
- MSVC linker (Visual Studio Build Tools)

## Building

**IMPORTANT:** You must use `build.py` to build this project. Direct `cargo build` will fail due to polymorphic markers and missing encryption keys.

### Setup
```bash
# Install Rust nightly toolchain (REQUIRED)
rustup override set nightly

# Verify nightly is active
rustc --version
# Should show: rustc 1.xx.0-nightly

# Generate cargo config with your paths
python scripts/gen_cargo_config.py

# Or generate with placeholder paths (for sharing)
python scripts/gen_cargo_config.py --home "C:\\Users\\YourUsername" --project "C:\\path\\to\\ferrox"
```

### Build with build.py
```bash
# Standard polymorphic build (RECOMMENDED)
python scripts/build.py
```

This runs the full build pipeline:
1. Runs `pycrypt.py` to encrypt strings in lib.rs
2. Mutates source code (junk injection, constant randomization)
3. Generates unique AES-256-GCM key for string encryption
4. Generates unique DJB2 hash key for API hashing
5. Compiles with `cargo +nightly build --release`
6. Applies PE obfuscation via `pe_obfuscate.py`
7. Restores original source files
8. Archives output with random filename in `builds/` directory

Every build produces a unique binary with different hash signatures.

### Build Options
```bash
# Build for 32-bit (i686)
python scripts/build.py --32bit

# Run full pycrypt before building
python scripts/build.py --pycrypt

# Clean build (removes all polymorphic markers)
python scripts/build.py --clean
```

The `--clean` flag strips all marker comments (`#ultraprotect`, `#junk`, `#polymorphnop`, etc.) and their generated code blocks, leaving clean compilable Rust.

### Cargo Config

The `.cargo/config.toml` file contains aggressive optimization and anti-forensics settings:

**Optimization:**
- `opt-level=z` - Optimize for size
- `lto=fat` - Full link-time optimization
- `codegen-units=1` - Single codegen unit for maximum optimization
- `strip=symbols` - Strip all symbols
- `debuginfo=0` - No debug info
- `panic=abort` - Abort on panic (smaller binary)

**Anti-Forensics:**
- `--remap-path-prefix` - Removes all filesystem paths from the binary
- Maps cargo registry, rustup toolchains, project paths to `/dev/null`
- Prevents forensic analysis from discovering build environment details
- `/PDBALTPATH:/dev/null` - Removes PDB path from PE header
- `/DEBUG:NONE` - No debug info in linker

**Stealth:**
- `/SUBSYSTEM:WINDOWS` - No console window
- `/ENTRY:mainCRTStartup` - Custom entry point
- `/MANIFEST:NO` - No embedded manifest

## Configuration

Before building, you need to configure the Telegram bot token for exfiltration:

1. Create a Telegram bot via [@BotFather](https://t.me/botfather)
2. Get your chat ID via [@userinfobot](https://t.me/userinfobot)
3. Update the configuration in your source (markers will encrypt these at build time)

The polymorphic builder will encrypt all sensitive strings with unique keys per build.

### Testing in VMs

**IMPORTANT:** Ferrox has extensive anti-VM detection that will cause it to exit silently if run in a virtual machine.

If you want to test in a VM for development/research purposes, you need to disable the detection checks:

1. Open `src/detection.rs`
2. Comment out or modify the `exit_if_detected()` function to skip VM checks
3. Rebuild with `python scripts/build.py`

**Detected VM environments:**
- VMware (Workstation, ESXi, Fusion)
- VirtualBox
- QEMU/KVM
- Hyper-V
- Xen
- Parallels

The detection uses CPUID checks, MAC address prefixes, WMI queries, registry artifacts, and process scanning. For legitimate testing, modify the source to disable these checks.

## Disclaimer

This project is for educational and research purposes only. It demonstrates advanced malware techniques including:
- Polymorphic code generation
- Direct syscall execution (Hell's Gate)
- Compile-time encryption
- Anti-analysis evasion
- Process injection

**Use responsibly and legally.** Unauthorized access to computer systems is illegal. The author is not responsible for misuse of this software.

## License

[MIT](LICENSE)

## Author

[vibheksoni](https://github.com/vibheksoni)

Currently open to work. If you're looking for someone with security research, malware analysis, reverse engineering, or full-stack development experience - hit me up.

- X/Twitter: [@ImVibhek](https://x.com/ImVibhek)
- Website: [vibheksoni.com](https://vibheksoni.com/)
- GitHub: [vibheksoni](https://github.com/vibheksoni)

---

*Remember: With great power comes great responsibility. Use this knowledge to make the internet safer, not more dangerous.*
