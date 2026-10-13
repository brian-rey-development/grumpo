# 0009. Private browsing counts toward limits in memory only

Accepted on 2026-10-12.

## Context

Two promises pull in opposite directions. Grumpo tells the person that private browsing leaves no record, which is the reason someone opens an incognito window, so nothing about that time may reach the database, an export or a log. Grumpo also enforces daily limits, and if private windows were invisible to them, opening an incognito window would defeat every limit and every blocked site, which makes the limits pointless for the person most likely to need them.

## Decision

The platform layer reports whether the front window of Chrome, Brave or Edge is private, and the observer carries that in the `Activity` as `private`. The session builder refuses to turn a private activity into a session, so storage never sees it. Enforcement still sees the domain, so blocked sites are closed in a private window like in any other. For limits, the usage tracker keeps a tally of engaged private seconds per limit target for the current local day, adds it to the stored usage whenever it checks a limit, and never persists it. A restart of the app forgets the tally, and so does the next local midnight, when limits reset anyway. A limit that is reached is reported to the daemon like any other, and the daemon then remembers the target as exhausted for the day, so a block that private time caused survives a restart.

Safari and Firefox do not reveal whether a window is private, so their private windows are indistinguishable from normal ones and are recorded like any other browsing.

## Consequences

The privacy promise holds in the strongest form, since nothing about a private window exists on disk, and limits and blocks still apply to it. The weakness is that the tally is volatile, so quitting Grumpo resets the progress that private browsing made toward a limit that was not yet reached. This is accepted because closing that gap would mean writing private time to disk, which contradicts the reason for the feature, and because the cost of the bypass is high, as it also stops tracking and takes a deliberate act. The tally also means the progress shown for a limit can include time that appears nowhere in the reports. Browsers that hide private windows leave the person who wants no trace with one option, which is not to grant the Automation or Accessibility permission for them, as [privacy.md](../privacy.md) explains.
