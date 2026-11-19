# Changelog

## [0.2.1](https://github.com/fulldecent/zellij-compact-bar-ng/compare/v0.2.0...v0.2.1) (2026-11-19)


### Bug Fixes

* install the toolchain before selecting cargo ([c003fb6](https://github.com/fulldecent/zellij-compact-bar-ng/commit/c003fb6086764eaa5e84c56b192e3c02563779e0))
* keep one space on each side of a labeled chip ([3d14751](https://github.com/fulldecent/zellij-compact-bar-ng/commit/3d147518f9bc89e8dda8276b9490804fefad94df))
* keep rename controls when the title fills the wider budget ([d1129b3](https://github.com/fulldecent/zellij-compact-bar-ng/commit/d1129b3c0a334a770f097cfca6e861f666d239e4))
* keep the space before ESC when the help title is hidden ([4abe802](https://github.com/fulldecent/zellij-compact-bar-ng/commit/4abe802ef067e61e1051fb31b90deedf02606f90))
* reverse labeled chips and drop the repeated mode name ([ff4555f](https://github.com/fulldecent/zellij-compact-bar-ng/commit/ff4555f7107ff5555033920c2fc3ea76be873b42))
* stop the snapshot script when rustup which fails ([72dc688](https://github.com/fulldecent/zellij-compact-bar-ng/commit/72dc68830f132cbd2700df517517f92af844e1aa))

## [0.2.0](https://github.com/fulldecent/zellij-compact-bar-ng/compare/v0.1.0...v0.2.0) (2026-11-16)


### Features

* paint tab ribbons from the theme ([454e7fd](https://github.com/fulldecent/zellij-compact-bar-ng/commit/454e7fd74449a66ca6557c66319f57370a1f810a))
* paint tab ribbons from the theme ([b5a5353](https://github.com/fulldecent/zellij-compact-bar-ng/commit/b5a5353465b887b2f434aea2a47d332f7e3e9697))

## 0.1.0 (2026-11-14)


### Features

* add first implementation ([0818d05](https://github.com/fulldecent/zellij-compact-bar-ng/commit/0818d05ed4d8a3769e84f34aa195da711db36c81))

## Changelog

## Unreleased

- A labeled chip reverses the key and the name together on hover. Exactly one space on each side is part of that target, including the space in front of the first chip. A key with no name is reversed, and it turns normal on hover. Letter combos such as `HJKL` still click one glyph.
- The hint block does not repeat the mode name. `PANE`, `TAB`, `RESIZE`, `MOVE`, `SCROLL`, and `SESSION` stay on the left. Scroll's word is `SCROLL`. Session's word is `SESSION`.
- One row: Zellij, the session name, the compact-bar mode, ribbon tabs, two spaces, and the status hints.
- The hint block has no `Ctrl +` label and no Alt group. It is as wide as the hints it shows.
- Tab names stay whole. The hint block shortens first.
