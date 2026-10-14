# 0015. Only a registrable domain is written with aliases, and only with the aliases it serves

Accepted on 2026-10-13.

## Context

A hosts file has no wildcards, so since version 0.1.0 the daemon wrote every blocked site together with the prefixes `www.`, `m.`, `mobile.`, `old.`, `new.`, `music.`, `i.` and `amp.`. It applied all eight to every site, including sites that are already a subdomain. The rules imported from `focus-hosts` list subdomains such as `m.youtube.com`, `old.reddit.com` and `store.epicgames.com` as sites of their own, so the owner's hosts file held 500 names for about 60 sites, among them `m.m.facebook.com`, `old.old.reddit.com`, `music.9gag.com` and `new.store.playstation.com`. None of those hosts exist. They blocked nothing extra and made the block look like junk in the one file a person reads when a site will not load.

## Decision

A site gets aliases only when it is a registrable domain, one label above a public suffix, such as `youtube.com` or `espn.com.ar`, which `Domain::is_registrable` decides with the same suffix heuristic that rejects `com.ar` as a site. A subdomain is written exactly as given, because `m.youtube.com` and `store.steampowered.com` have no `www.` or `m.` of their own. Every registrable site gets `www.` and `m.`, which nearly every site serves its pages from. The other prefixes go only to the sites that use them, listed in `SITE_ALIASES` in `focus-core::hosts`, which are `old.`, `new.` and `i.` for Reddit and `mobile.` for Facebook, Twitter and X. `amp.` is gone, since no blocked site serves pages from it. `music.` is gone too, because YouTube Music is a product of its own that many people keep playing while they work, so blocking YouTube no longer takes it down unannounced, and a person who wants it blocked adds `music.youtube.com` as a site.

The app's loosening preview mirrors the rule in `app/src/features/rules/hosts.ts`, and both sides are tested against `crates/focus-core/tests/vectors/hosts.json`, which has a case for every site in `SITE_ALIASES`. The generated `HOST_ALIAS_PREFIXES` constant is removed, because one flat list can no longer describe the rule.

## Consequences

The owner's block shrank from 500 names to 139, every one of them a host that exists. Hosts such as `old.youtube.com` or `mobile.reddit.com` are no longer written, which unblocks nothing a browser can reach. A site that serves its pages from an unusual alias needs an entry in `SITE_ALIASES`, or the person adds that alias as a site of its own. Because the rule looks at one site at a time, adding a site never removes a name from the block, so the strictness check keeps comparing what the hosts file blocks before and after a change.
