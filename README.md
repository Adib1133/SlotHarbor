# SlotHarbor

<img src="assets/icon.svg" alt="SlotHarbor logo" width="72" height="72">

SlotHarbor backs up and restores PC game saves on Windows. It is a fork of
[Ludusavi 0.31.0](https://github.com/mtkennerly/ludusavi) by Matthew T. Kennerly
and uses the [Ludusavi Manifest](https://github.com/mtkennerly/ludusavi-manifest)
for known save paths.

## What this fork adds

- **Game detection.** Steam app IDs take priority over folder-name matching.
  Title matching distinguishes similarly named games and numbered sequels.
- **AppData Local.** The scan checks Local counterparts of game-specific paths
  listed under Roaming or LocalLow.
- **Save files and configuration.** INI files and manifest-tagged configuration
  are labeled in the file list and included by default. Expand the game and
  uncheck any file you want to leave out. Explicit save-file tags take priority.
  **Settings → Scan** can also label common non-INI settings filenames.
- **Missing saves.** Preview the game, then open **Scan details** to see
  configuration files and locations checked. If no saves were found, browse
  for the file or folder. SlotHarbor adds the path to the game and previews it again.
- **Restore review.** See the actual file destinations, including redirects,
  before continuing. The dialog flags newer live files and paths under another
  standard Windows user profile.
- **Safety copies.** SlotHarbor preserves existing files before replacing them.
  If it cannot make a copy, it skips that file and reports an error.
  [Recover a file from a safety copy](docs/help/restore-review.md).

## Finding and comparing saves

- **Find game** searches all loaded manifest titles and aliases, even before
  scanning. Select a result to preview its backup.
- **Back** returns to the previous screen or reverses the last navigation step,
  such as opening a game, opening a folder, or filtering to shared files.
- Hover over **Duplicates** to see why entries share a path. In a game's file
  list, hover over the changed-file badge for modification times. On Restore,
  SlotHarbor compares the live file with the version in the selected backup
  when both timestamps are available.
- Backup and restore default to `C:\Slotharbor`. Existing custom paths and
  older backup folders that are present on disk are preserved.

## Run on Windows

Extract a Windows ZIP from the repository's **Releases** page and run
`slotharbor.exe`. A locally packaged copy of this project also includes the
executable and ZIP in `dist/`.

1. On **Backup**, run **Preview** to confirm which games and files were found.
2. Choose **Back up** to create a backup.
3. On **Restore**, select the backup and review the destination list before
   continuing.

SlotHarbor reads existing Ludusavi backups and settings. Use
`slotharbor --config DIRECTORY` for a separate profile or place
`slotharbor.portable` beside the executable for portable mode. The earlier
`ludusavi.portable` marker also works. See the
[application folder documentation](docs/help/application-folder.md).

## Build from source

Install the stable Rust MSVC toolchain and Microsoft C++ Build Tools with a
Windows SDK. In PowerShell, run:

```powershell
cargo build --release --locked
.\target\release\slotharbor.exe --help
```

See [contributing](CONTRIBUTING.md) for test setup and
[packaging a release](docs/releasing.md) for the ZIP and checksum workflow.
The [changelog](CHANGELOG.md) includes the fork changes and upstream history.

## License

SlotHarbor carries forward Ludusavi's [MIT license](LICENSE) and copyright notice.
See [third-party notices](THIRD_PARTY_NOTICES.md) for the bundled fonts and Rust
dependency license files.
