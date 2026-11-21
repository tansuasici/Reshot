# Changelog

## 0.3.1

### Changed
- The Mac app is now signed with a Developer ID and notarized by Apple. macOS opens it without the "can't check" warning, and Screen Recording permission now stays in place when Reshot updates.

### Mac: Screen Recording permission, one last time
Because the app's signature changed, macOS asks for Screen Recording permission once more after this update. Allow Reshot in *System Settings → Privacy & Security → Screen Recording*, then quit and reopen Reshot. Later updates keep the permission.

## 0.3.0

### New: Automatic updates
- Reshot checks for a new version shortly after it starts, and from the menu bar / tray icon → **Check for Updates…**. It asks before installing, then restarts.
- Coming from 0.2.1 or older? Install this version by hand once; later versions arrive on their own.

## 0.2.1

### Changed
- Starting Reshot no longer takes a screenshot. It starts quietly in the menu bar / tray; press the shortcut to capture.
- macOS shows the app as "Reshot" (not "reshot") in login-item and background-activity notices. If Open at login was on, it is updated automatically.

## 0.2.0

### New: Settings
Open it from the menu bar / tray icon → **Settings…**. Changes apply right away.

- **Capture shortcut**: click the shortcut and press a new key combination. If another app already uses that combination, your old shortcut keeps working.
- **Save folder**: choose where the save dialog opens.
- **Save as**: PNG or JPEG.
- **Open at login**: start Reshot when you log in. It waits quietly in the menu bar and doesn't take a screenshot on startup.

### Changed
- The default shortcut on Windows and Linux is now <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>9</kbd> (it was <kbd>Print Screen</kbd>), matching <kbd>⌘</kbd> <kbd>⇧</kbd> <kbd>9</kbd> on Mac. You can change it in Settings.
- The tray menu shows your current shortcut.

## 0.1.0

First release: select part of the screen, mark it up with arrows and notes, then copy or save it.
