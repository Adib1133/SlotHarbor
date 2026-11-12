# Package a Windows release

On Windows, build and package the locked source from the repository root:

```powershell
cargo build --release --locked --target x86_64-pc-windows-msvc
./scripts/package-windows.ps1 -Executable target/x86_64-pc-windows-msvc/release/slotharbor.exe -Architecture x64
```

The `dist/` directory contains the executable, ZIP, and SHA-256 checksum.
The ZIP includes the executable and license notices. Keep `dist/` out of Git and
attach the ZIP and checksum to the corresponding GitHub release.

The **Windows** workflow verifies pushes and pull requests. Run it manually or
push a version tag beginning with `v` to also produce x64 and x86 ZIP artifacts.
Download those artifacts from the Actions run and attach the enclosed ZIPs and
checksums when you create a GitHub release.
