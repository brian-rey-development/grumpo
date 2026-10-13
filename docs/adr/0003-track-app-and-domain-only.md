# 0003. Track the app and the domain only

Accepted on 2026-10-12.

## Context

A screen-time tool is only useful if it can say where the time went, and a naive implementation could record window titles, full URLs and document names to answer that. Such a record is a detailed log of everything a person read and wrote, it is dangerous if the disk is shared or backed up, and it is unnecessary for the product, whose rules operate on apps and sites. Remote favicons and analytics would add network traffic that also leaks browsing habits.

## Decision

Grumpo stores only the application id, the application name, the domain and timestamps for each session. Browser tab addresses are normalized to a host the moment they are read and the remainder is discarded. Window titles, paths, queries and content are never stored or logged, and the app makes no network calls, including for favicons. Domains and app names do not appear in logs above `debug` level.

## Consequences

The data model is small, easy to export and easy to delete, and the privacy claim in [privacy.md](../privacy.md) can be verified by reading one schema. The trade-off is that reports cannot distinguish two pages on the same site, so a limit on `youtube.com` cannot treat a lecture differently from a feed. Features that depend on page content, such as category detection by title, are deliberately out of reach.
