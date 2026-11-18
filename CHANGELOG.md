# Changelog

## [0.2.0](https://github.com/fulldecent/zellij-compact-bar-ng/compare/v0.1.0...v0.2.0) (2026-11-16)


### Features

* paint tab ribbons from the theme ([454e7fd](https://github.com/fulldecent/zellij-compact-bar-ng/commit/454e7fd74449a66ca6557c66319f57370a1f810a))
* paint tab ribbons from the theme ([b5a5353](https://github.com/fulldecent/zellij-compact-bar-ng/commit/b5a5353465b887b2f434aea2a47d332f7e3e9697))

## 0.1.0 (2026-11-14)


### Features

* add first implementation ([0818d05](https://github.com/fulldecent/zellij-compact-bar-ng/commit/0818d05ed4d8a3769e84f34aa195da711db36c81))

## Changelog

## Unreleased

- A labeled chip reverses the key and the name together on hover, including the space on each side. A key with no name is reversed, and it turns normal on hover. Letter combos such as `HJKL` still click one glyph.
- The hint block does not repeat the mode name. `PANE`, `TAB`, `RESIZE`, `MOVE`, `SCROLL`, and `SESSION` stay on the left. Scroll's word is `SCROLL`. Session's word is `SESSION`.
- One row: Zellij, the session name, the compact-bar mode, ribbon tabs, two spaces, and the status hints.
- The hint block has no `Ctrl +` label and no Alt group. It is as wide as the hints it shows.
- Tab names stay whole. The hint block shortens first.
