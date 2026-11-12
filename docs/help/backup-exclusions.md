# Backup exclusions
Backup exclusions let you set paths and registry keys to completely ignore
from all games. They will not be shown at all during backup scans.

Configure path and registry exclusions on the **Settings** screen.

For excluded file paths, you can use glob syntax.
For example, to exclude all files named `remotecache.vdf`, you would specify `**/remotecache.vdf`.

## Game settings files

Enable **Exclude game settings files from backups** under **Settings → Scan** to
skip manifest paths tagged as configuration and common settings filenames such as
`GameUserSettings.ini`. Explicit save-file tags take priority. The filter starts
off, so existing configurations keep backing up settings until you enable it.
