# Writing

NativeCompute talks like a capable colleague: direct, plain and calm. It says what happened, then what to do next, and stops. These rules apply to every word in an app or on a site: buttons, labels, notices, empty states, errors, settings and docs.

## Voice

- **Direct.** "Delete this script?" not "Are you sure you would like to delete this script?"
- **Plain.** Everyday words. "Saved", not "Persisted". "Could not open the file", not "File I/O error".
- **Calm.** No exclamation marks. No "Oops", "Uh oh", "Whoops" or apologies for normal things.
- **Specific.** Name the thing: "Snapshot restored", not "Done".
- **Short.** One idea per sentence. One or two sentences per notice.
- **Human.** "You" and "your" for the person. The app does not say "I" or "we", except on the website where NativeCompute speaks as a studio.

## Case

- Sentence case for buttons, labels, menu items, settings, notices, headings and page titles in running text.
- Upper case only for display type (giant titles) and HUD readouts. The components set the case for you, so write the text in sentence case and let `ui::giant` and `ui::hud` raise it.
- Product names keep their own case: NativeCompute, Blackline, YouTrack, GitHub.

## Buttons and menu items

- Start with a verb: **New script**, **Export**, **Open the folder**, **Download it**.
- Name the result, not the mechanism: **Save a copy**, not **Duplicate file**.
- A destructive button says exactly what it destroys: **Delete**, **Remove from team**, **Discard changes**. Never **OK** or **Yes**.
- The safe answer in a confirmation is **Cancel** or **Keep it**.

## Notices

Title: what happened, in two to four words. Body: the next step, or where to find the thing.

| Do | Don't |
|---|---|
| **Snapshot taken.** Find it under More → Snapshots. | **Success!** Your snapshot has been successfully created. |
| **Scene moved.** Ctrl+Z puts it back. | **Scene moved successfully.** |
| **Ledger 0.2.0 is out.** Download it from Settings → About. | **Update available!** Click here to update now! |
| **Restored.** It is back in the library. | **The item was restored.** |

## Errors and warnings

Say what went wrong in plain words, then how to fix it. Never blame the person and never show a raw error code alone.

| Do | Don't |
|---|---|
| **That is not a web address.** Set the YouTrack link in Settings. | **Invalid URL.** |
| **Nothing to export.** Choose a page, or keep the title page. | **Export failed: 0 pages selected.** |
| **Could not save.** The folder is read-only. Pick another one in Settings. | **Error 13: permission denied.** |
| **That is not a Discord ID.** A Discord user ID is a long number, 17 to 20 digits. | **Bad input.** |

When the system's own message helps, put it after the plain sentence: "Could not save. *Permission denied (os error 13).*"

## Confirmations

- The title is the question, naming the thing: **Delete this script?**, **Restore this snapshot?**, **Delete Launch notes?**
- The body says what happens and whether it can be undone: "It goes to the bin. You can bring it back for 30 days." or "This can't be undone."
- The yes button repeats the verb: **Delete**, **Restore**.
- Only ask before something that loses work or is hard to undo. Never ask "Are you sure you want to close Settings?"

## Empty states

A giant title that says the state, one line that says how to change it, and the button that does it.

> **NOTHING YET**
> Make your first item and it shows up here.
> [ New item ]

## Fields and hints

- Hint text says what goes in the field, not "Enter …": **Name this script**, **Filter scripts**, **Discord user ID**.
- Put the shortcut after a middle dot: **Search · Ctrl+F**.
- Placeholders that stand in for a person's own setting say so: **YouTrack workspace link here**.

## Settings

- The label is the setting, the hint is the effect: **Compact cards** / *One line per item.*
- Switches are named for the "on" state: **Check for updates**, not **Disable update checks**.
- Group with `ui::section` labels that read like places: **Where things live**, **Motion**, **Cards**.

## Tooltips

The action name, then the shortcut after a middle dot: **Settings · Ctrl+,**, **Close · Esc**, **Hide · Ctrl+I**. No full stops.

## HUD readouts

Short, factual, separated by a slash with spaces: **V0.1.0 / STABLE**, **LEDGER / 12 ITEMS**, **SETTINGS / DARK**. Use them for versions, counts, breadcrumbs and live values, never for sentences.

## Numbers, time and symbols

- Numerals for all numbers: 3 items, 12 scenes.
- Arrows for paths through the interface: Settings → About, More → Snapshots.
- Middle dot `·` between an action and its shortcut, or between short facts: Saved 2 minutes ago · 14 KB.
- Curly apostrophes are fine in prose; straight ones are fine in code.

## Words we use

| Use | Not |
|---|---|
| Delete | Remove permanently, Erase, Destroy |
| Settings | Preferences, Options, Config |
| Folder | Directory |
| Sign in, Sign out | Log in, Login, Log out |
| Could not, Can't | Unable to, Failed to |
| Download it | Click here |
