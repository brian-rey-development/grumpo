# 0008. Protected apps can never be blocked

Accepted on 2026-10-12.

## Context

Blocking an app means quitting it again every time it appears, and Grumpo lets a person pick any application for that. Most picks are harmless, but a few would turn the tool against the person who configured it. If Grumpo were blocked, nobody could open the window that requests an unlock, so the escape hatch that makes strict blocking tolerable would be gone. Blocking Finder, the Dock or the login window would repeatedly tear down the desktop session, and blocking System Settings would prevent granting the permissions that Grumpo needs or removing its login item. A rule like that is rarely typed on purpose, and it is easy to create by accident from a long list of running apps, for instance when an app is selected next to the one that was meant.

## Decision

Five bundle ids are protected, and they can be neither blocked nor the target of a daily limit. They are Grumpo itself, Finder (`com.apple.finder`), the login window (`com.apple.loginwindow`), System Settings (`com.apple.systempreferences`) and the Dock (`com.apple.dock`), held in `PROTECTED_BUNDLE_IDS` in `focus-core` and tested with `is_protected`, which ignores case like the system does. Terminal is deliberately not on the list, because people do want to block it.

The rule is enforced in four places so that no layer has to be perfect. The daemon rejects a rule set that names a protected app, in `Rules::validate`, with the error `RulesError::ProtectedApp`, which reaches clients as `rules_invalid` with a sentence that names the app. The first state migration drops protected apps from saved rules, since one old entry would otherwise make the whole saved state fail validation. The app removes protected apps and itself from the list it offers for blocking, and the enforcement policy checks `is_protected` before it quits anything, so a status that somehow listed one would still not make Grumpo act on it.

## Consequences

A person can no longer lock themselves out of Grumpo or damage the desktop with a rule, and the daemon does not have to trust the app to uphold that, because validation lives where the rules are accepted. The cost is that a few apps are exempt from the product's promise, which is acceptable because they are infrastructure and not distractions, and the exempt list is short and visible in one constant. Adding an app to the list is a protocol-visible change, since rules the daemon accepted before may fail validation afterwards, so it needs a state migration that removes the new entry, as that first migration did. Protecting an app also does not stop someone from blocking a substitute for it, which the [threat model](../threat-model.md) lists among the accepted bypasses.
