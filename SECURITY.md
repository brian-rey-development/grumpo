# Security policy

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub Security Advisories at https://github.com/brian-rey-development/grumpo/security/advisories/new, and do not open a public issue, because the daemon runs as root and a public report would reach attackers before it reaches a fix. Include the version, the steps to reproduce and what an attacker gains. You will get a first reply within a week, and the fix and its disclosure are coordinated with you in the advisory.

Only the latest release receives security fixes while the project is at version 0.x.

## In scope

Grumpo is built around a root LaunchDaemon, so the reports that matter most are the ones that cross the boundary between an ordinary user and root.

- The root daemon `grumpod`, including how it parses and validates every request.
- Its Unix socket and the peer-credential check that decides who may talk to it. Any way for a process other than root and the owning user to issue commands, or to confuse which user is asking, is in scope.
- The installer, which runs as root after the administrator prompt, together with the uninstaller and the files they place.
- Writing the managed block of `/etc/hosts`, including anything that lets a request corrupt the file or write outside that block.

## Trust model of the socket

The daemon decides who may talk to it by the uid of the process on the other end of its Unix socket, and it accepts only the user who owns the installation and root. Any process running as that user can therefore do what the user can do in the app, including the unlock flow with its cooldown and typed sentence. This is by design. Grumpo is a self-control tool, and the adversary it guards against is the impulse of the person using it, not malware running as that person. Pinning the app's code signature would not change what the user can do, because the user can run any program they choose, which is why the check identifies a user and not a program.

The friction sits in the daemon rather than in the client. The daemon computes every loosening itself and enforces the unlock cooldown whatever the client sends, so a script running as the user gains no shortcut that the person lacks.

The check also runs the other way. The socket sits in `/var/run`, which members of the `daemon` group can write, so the app and `grumpod ping` verify that the process serving the socket is root, or their own user in a development setup, before sending anything, which keeps a look-alike socket from answering in the daemon's place.

## Out of scope

The bypasses already documented in [docs/threat-model.md](docs/threat-model.md) are accepted limits of a user-space focus tool and are not vulnerabilities. Examples are an administrator stopping the daemon or editing `/etc/hosts`, changing the system clock, DNS over HTTPS in a browser, and the ad hoc signature of the app bundle that the threat model discusses under the installer. A report that shows one of these is worse than documented, for example by working without the administrator password, is still welcome.

Weaknesses in third-party dependencies are best reported upstream, although a note here is appreciated when Grumpo is affected.
