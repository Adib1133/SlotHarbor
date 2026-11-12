# Windows installation

Follow the [Windows development instructions](../../CONTRIBUTING.md) to install
the Rust MSVC toolchain, the Microsoft C++ compiler, and a Windows SDK.

From the project folder, run `cargo build --release --locked`. The portable
executable is `target/release/slotharbor.exe`.

SlotHarbor uses GPU rendering on supported Windows systems. To select software
rendering, set the `ICED_BACKEND` environment variable to `tiny-skia`.

SlotHarbor uses the existing Ludusavi configuration folder and backup format.
For portable mode, place a file named `slotharbor.portable` beside the executable.
Existing `ludusavi.portable` markers also work.
