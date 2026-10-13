# Security policy

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub Security Advisories at https://github.com/brian-rey-development/grumpo/security/advisories/new, and do not open a public issue, because the daemon runs as root and a public report would reach attackers before it reaches a fix. Include the version, the steps to reproduce and what an attacker gains. You will get a first reply within a week, and the fix and its disclosure are coordinated with you in the advisory.

Only the latest release receives security fixes while the project is at version 0.x.

## In scope

Grumpo is built around a root LaunchDaemon, so the reports that matter most are the ones that cross the boundary between an ordinary user and root.

- The root daemon `focusd`, including how it parses and validates every request.
- Its Unix socket and the peer-credential check that decides who may talk to it. Any way for a process other than root and the owning user to issue commands, or to confuse which user is asking, is in scope.
- The installer, which runs as root after the administrator prompt, together with the uninstaller and the files they place.
- Writing the managed block of `/etc/hosts`, including anything that lets a request corrupt the file or write outside that block.

## Out of scope

The bypasses already documented in [docs/threat-model.md](docs/threat-model.md) are accepted limits of a user-space focus tool and are not vulnerabilities. Examples are an administrator stopping the daemon or editing `/etc/hosts`, changing the system clock, DNS over HTTPS in a browser, and the ad hoc signature of the app bundle that the threat model discusses under the installer. A report that shows one of these is worse than documented, for example by working without the administrator password, is still welcome.

Weaknesses in third-party dependencies are best reported upstream, although a note here is appreciated when Grumpo is affected.
