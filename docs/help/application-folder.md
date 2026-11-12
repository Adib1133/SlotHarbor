# Application folder

SlotHarbor uses `%APPDATA%/ludusavi` for settings, logs, the manifest, and restore
safety copies. Using the existing Ludusavi folder lets the fork read existing
settings and backup locations.

For a separate configuration, launch with `--config DIRECTORY`. For portable
mode, place a file named `slotharbor.portable` beside `slotharbor.exe`. The older
`ludusavi.portable` marker also works.

The application can replace `manifest.yaml` when downloading an updated
Ludusavi Manifest. Use custom games or a secondary manifest for your own paths.
