# Privacy

Grumpo is a local-first tool, and the promise it makes is narrow enough to be verified in the source. It records which application was in the foreground and, for supported browsers, which website host was open, and it keeps that data on the Mac where it was collected. Its only network request is a daily check for a new version, which carries none of that data.

## What is stored

Usage history lives in a SQLite database named `focus.db` and consists of sessions. Each session holds only the following fields.

- The application identifier (the bundle id, for example `com.apple.Safari`)
- The application name as shown to the user
- The domain, only when the app is a supported browser, a host could be read (for example `youtube.com`) and the window is not a private window that Grumpo can recognize
- The start and end timestamps of the session

Beside the sessions, the same database keeps a small table of app settings, namely the retention period and a flag that records that the launch at login default was applied. Rules and unlock state, which are configuration rather than history, are stored by the daemon and contain the blocked sites and apps, the weekly schedule, the daily limits, the list of exhausted limits for the current day with the allowance each had when it ran out, and the current unlock phase with the number of unlocks used today.

## What is never stored

Grumpo never stores a URL, a path, a query string, a fragment, a page title, a window title, the contents of any document or page, keystrokes, screenshots or clipboard contents. When a browser reports the address of the active tab, the app reduces it to a normalized host immediately inside the platform layer (`https://www.YouTube.com/watch?v=abc` becomes `youtube.com`) and discards the rest in memory, before anything reaches the tracker, the disk or a log line. Only `http` and `https` addresses become a site, so internal pages such as `about:blank`, `chrome://settings` or `view-source:` never do. The reason for this design is that a history of hosts is enough to answer "where did my time go", whereas full URLs and titles would turn a focus tool into a surveillance log of everything the person read.

## Private browsing

When the front window of Chrome, Brave or Edge is an incognito window, Grumpo never writes the domain, or any other trace of that time, to the database. If a session was running when the incognito window came to the front, it ends at that moment, and nothing is stored while the window stays in front. The window is still read, because blocks and daily limits would otherwise be trivial to avoid by browsing privately. A blocked site is therefore closed in an incognito window like in any other, and the time counts toward a daily limit for that site or for the browser app.

That counting is the only trace the time leaves, and it is held in memory. For each limit target the app keeps a running total of engaged private seconds for the current day, adds it to the stored usage when it checks a limit, and never writes it anywhere. Quitting the app or restarting the Mac forgets it, and so does the next local midnight, which is when limits reset anyway. The consequence is that a restart can hand back private time that was already spent, which is the price of keeping the promise that private browsing leaves no record on disk, and [ADR 0009](adr/0009-private-browsing-counts-toward-limits-in-memory-only.md) records that choice. Once a limit is reached the daemon remembers the target as exhausted for the day, so a block that private time caused survives a restart, and only the progress toward a limit that has not been reached yet is lost. Private time never appears in the reports, the timeline or an export, and it is not removed by deleting history, because history never held it.

Safari does not tell scripts whether a window is private, so Grumpo cannot tell a Safari private window from a normal one, and time spent in one is recorded at the host level like any other browsing. Firefox is read through the Accessibility API, which does not expose private windows either, so the same limit applies there. Refusing the Automation and Accessibility permissions below hides which site was open in these browsers, because Grumpo then never reads the address, so no domain is stored for them. It does not hide the time itself. The browser is still the frontmost app, so its time is recorded and counted toward limits under the browser's own name, and a private window of Safari or Firefox shows up in the totals like any other use of that browser.

## Tabs and windows

Grumpo reads and closes tabs only in the frontmost window of a browser. The scripts it sends address the front window and nothing else, so tabs in other windows are never read, never counted and never closed, and a window that is not in front is not looked at until it comes forward. Within the front window only the active tab is read, and it is read at most every 4 seconds. Firefox tabs are read through the Accessibility API and are never closed.

## What counts as time

Time counts while the frontmost application is in use. Reading without touching the keyboard or mouse is credited for up to two minutes. After that the session ends at the moment of the last input, so a long pause is never credited, unless the frontmost application, or a helper process working for it, holds a display-sleep assertion because a video or a call is playing. Media playing in a different application does not keep another application's session alive. Nothing counts while the screen is locked or while the Mac sleeps.

## Web apps

A site installed as a web app, such as a Chrome or Edge app or a Safari web app in the Dock, is an application of its own with its own bundle id. Grumpo stores it like any other app, under that app's identifier and name and with no domain, so its time does not appear under the site and a limit on the site does not count it. Blocking the wrapper as an app in the Rules screen covers it.

## Where it lives

The usage database sits in the per-user app data directory that Tauri derives from the bundle identifier, which is `~/Library/Application Support/io.github.brian-rey-development.grumpo`. The directory has mode `0700` and the database mode `0600`, so other local accounts cannot read it. The daemon state file sits in a root-owned directory with mode `0600`. The hosts block that enforcement writes lives in `/etc/hosts` between marker lines, and it contains blocked domains only, never usage data. The `grumpod` socket is a local Unix socket restricted to your user and root, and it is not reachable from the network. `/etc/hosts` is readable by every account on the Mac, as it always is, so another account can read the list of blocked sites in the marked block, although it holds no usage data. The block also applies to those accounts, while only the owner of the installation can change the rules.

## Retention and deletion

Sessions older than the retention period are deleted automatically, and the default is 90 days. The period can be set between 30 and 365 days in Settings, which offers 30, 90, 180 and 365 days. Choosing a shorter period asks for confirmation, because the older history is deleted right away when the setting is saved and not at the next daily run. The database enables SQLite's `secure_delete`, which overwrites deleted rows with zeros instead of leaving their text in free pages, and "Delete history" also compacts the file and truncates the write-ahead log.

Two actions in Settings give the person control over the data. "Export data" writes all stored sessions and settings to a JSON file in the Downloads folder, readable only by the owner, and shows its name. "Delete history" removes every session before the current local day, and it removes today's sessions as well unless today's usage is still needed. Today's sessions are kept until local midnight in two cases. The first is when at least one daily limit is configured, because today's sessions are the usage the limits count, and erasing them would hand back time that was already spent. The second is when the Grumpo service cannot be reached, because the limits are then unknown and keeping today is the safe answer. The dialog says which of the three situations applies before anything is deleted. A session that crosses midnight is cut so that only the part after midnight remains. Rules, limits and settings such as the retention period are not usage history, so they are kept. If Grumpo is tracking when the data is deleted, it carries on recording the current session from that moment into a new row.

Deleting `focus.db` by hand removes the history too, and it also resets the progress toward every daily limit that has not been reached yet, because that usage is stored only there. The threat model lists this among the accepted bypasses.

If Grumpo finds its database damaged, it moves `focus.db` and its `-wal` and `-shm` files to `focus.db.corrupt-<unix time>` in the same folder, starts a new database and tells the person once. Those files still hold the sessions they held, and neither retention nor Delete history touches them, so delete them by hand when the old history should go. The daemon does the same with a damaged `state.json` and leaves `state.json.corrupt-<unix time>` in its state directory, which `--purge` removes.

Uninstalling with `scripts/uninstall.sh` removes the daemon, the hosts block and the Grumpo login item, and `--purge` also deletes the daemon state and logs. The script does not touch the app or the per-user data, so the database and the app logs are removed by deleting the folders named in this document.

## Network access

The daemon makes no network requests, and the app makes exactly one, a check for a new version two minutes after it starts and then once a day. It is a plain HTTPS download of `latest.json` from the latest GitHub release of the project, followed by the update archive when there is a newer version, and it carries no identifier, no usage data and no settings, although GitHub sees the request the way it sees any download. Settings has a switch that turns it off. There is no telemetry, no analytics, no crash reporting service and no remote asset loading, which includes favicons, so site icons are never fetched. App icons are read from the local application bundles. The webview's content security policy sets `default-src 'none'` and permits only the local Tauri IPC bridge, so the check runs in the Rust side of the app and a bug in the UI cannot reach the network.

## Logs

Both binaries log through `tracing`. At `info` and above, logs contain event names, counts, durations, error codes and phase names, but never domains and never application names. Those values may appear only at `debug` level, which is off by default and which a person enables deliberately when diagnosing a problem. Failures of the platform layer are logged with a fixed word such as `permission_denied` or `timeout` instead of the underlying message, because that message can name a browser or an address. The daemon log is at `/Library/Logs/Grumpo/grumpod.log` and the app logs are in `~/Library/Logs/Grumpo/`, with seven daily files kept. Logs contain no URLs, titles or paths at any level. Notifications are the one place where an app or a site is named, because they exist to tell the person what was closed, and they are shown on the Mac and never written to a log.

## Permissions requested

macOS gates the signals Grumpo needs behind two permissions, and the Settings screen shows the state of each. macOS stores a grant against the code signature of the app, so a build signed ad hoc is a new app to macOS after every rebuild and asks again, whereas one signed with a stable certificate keeps its grants. [development.md](development.md) explains the two signing modes and how to reset a stuck grant.

| Permission | Why it is needed | When it is requested |
|---|---|---|
| Accessibility | Firefox does not expose its active tab to scripting, so Grumpo reads the URL field of the focused window through the Accessibility API as a best effort. Asking Firefox for its role switches on its accessibility tree, as any assistive app does. | When the person presses the grant button for it in Settings |
| Automation (per browser) | Chrome, Brave, Edge and Safari report the active tab address through AppleScript, and macOS asks for consent once per browser. | When the person presses the grant button for that browser in Settings |

Grumpo never lets macOS surprise the person with a consent dialog from a background poll. Before it runs a script against a browser it asks the system for the Automation state, which does not prompt, and it caches the answer for 30 seconds. While consent has not been given the script is skipped and the browser is reported as needing consent, and if consent was refused it is reported as denied. The dialog only appears when the person asks for it from Settings.

Both permissions can be refused. Without them Grumpo still tracks time per application, so the browser's time counts as before, and only the website breakdown for the affected browser is missing. Daily limits on a site cannot count the time spent in that browser, and blocked sites there rest on the hosts block alone, because Grumpo cannot see which site is open. The `NSAppleEventsUsageDescription` text shown by macOS states the purpose of the Automation request in plain language. Showing notifications is governed by the macOS notification settings for Grumpo and is not something Grumpo can grant itself.
