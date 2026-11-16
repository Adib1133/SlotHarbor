# Backup exclusions
Backup exclusions let you set paths and registry keys to completely ignore
from all games. They will not be shown at all during backup scans.

Configure path and registry exclusions on the **Settings** screen.

For excluded file paths, you can use glob syntax.
For example, to exclude all files named `remotecache.vdf`, you would specify `**/remotecache.vdf`.

## Game settings files

SlotHarbor reports INI and manifest-tagged configuration files separately from saves.
Explicit save-file tags take priority for games that store progress in a configuration-like
file. Configuration files are included by default. Expand a game or click its
**Configuration file** badge, then use each file’s checkbox to choose what to back up.
Under **Settings → Scan**, you can also label common non-INI settings filenames.

If a preview finds only configuration, open **Scan details** to see the checked locations.
Use **Browse for save file** or **Browse for save folder** to extend the game’s paths and
run another preview.
