## Windows development

Install stable Rust with the MSVC toolchain, rustfmt, and Clippy. Install Microsoft C++ Build Tools with the x64/x86 compiler and a Windows SDK. The resource compiler from the SDK is required for the executable manifest.

From the project directory in PowerShell:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace -- --deny warnings
cargo build --release --locked
```

The native build produces `target/release/slotharbor.exe`. To build for 32-bit Windows, add the `i686-pc-windows-msvc` target with rustup and pass `--target i686-pc-windows-msvc` to Cargo.

The full test suite uses a junction and registry fixtures. Run the registry setup in a disposable Windows test account or CI runner.

```powershell
reg import tests/ludusavi.reg
New-Item -ItemType Junction -Path tests/root3/game5/data-symlink -Target (Resolve-Path tests/root3/game5/data)
cargo test --locked
```

### Environment variables
These are optional:

* `LUDUSAVI_VERSION`:
  * If set, shown in the window title instead of the Cargo.toml version.
  * Intended for CI.
* `LUDUSAVI_VARIANT`:
  * If set, shown in the window title in parentheses.
  * Intended for alternative builds, such as using different Iced renderers.

### Icon
The SlotHarbor logo source is `assets/icon.svg`. The GUI uses `assets/icon.png` and Windows Explorer uses `assets/icon.ico`.
