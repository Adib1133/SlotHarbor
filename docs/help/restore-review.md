# Restore review and safety copies

The Windows app lists the resolved paths of files it plans to create or replace before
a restore. The review uses your selected backup and configured redirects. It flags
destinations under a different standard Windows user profile and existing files whose
modification time is later than the selected backup. Check these warnings and close
the game before continuing.

Before replacing an existing file, SlotHarbor saves its current contents under
`restore-safety` in the application folder. If that copy fails, SlotHarbor reports an
error and skips the file. This applies to GUI, command-line, and API restores.
Registry entries do not have safety copies.

To recover a file, open the corresponding numbered JSON record. It lists the original
destination and the name of the saved copy in the same folder. With the game closed,
copy the saved file back to that destination using the original filename. Safety copies
are kept until you remove them.
