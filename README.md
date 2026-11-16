# Zellij Compact Bar Next-Generation

[![Lint](https://github.com/fulldecent/zellij-compact-bar-ng/actions/workflows/lint.yml/badge.svg)](https://github.com/fulldecent/zellij-compact-bar-ng/actions/workflows/lint.yml) [![Build and test](https://github.com/fulldecent/zellij-compact-bar-ng/actions/workflows/build-test.yml/badge.svg)](https://github.com/fulldecent/zellij-compact-bar-ng/actions/workflows/build-test.yml)

A one-row replacement for Zellij’s `compact-bar`. Zellij, the session name, and the mode sit on the left. Ribbon tabs fill the middle. The key hints sit on the right, only as wide as they need to be.

Zellij Compact Bar Next-Generation offers:

* Zellij, then the session name, then the mode in the compact-bar colors
* Ribbon tabs from Zellij Tab Bar Next-Generation, including `+` and `← +N` / `+N →`
* Key hints from Zellij Status Bar Next-Generation, with no `Ctrl +` label and no Alt group
* A row that stays inside the terminal width

![Zellij, the session, the mode, ribbon tabs, and the key hints](screenshot.svg)

This is a [Zellij](https://zellij.dev/) plugin and supports Zellij 0.45 or newer.

A [Nerd Font](https://www.nerdfonts.com/) is required for the slants `` (U+E0BA) and `` (U+E0BC).

## Try it out

1. Select the latest wasm release and install to your plugins folder:

   ```sh
   name="com.github.fulldecent.zellij-compact-bar-ng.wasm"
   base="https://github.com/fulldecent/zellij-compact-bar-ng/releases/latest/download"
   mkdir -p ~/.config/zellij/plugins
   curl -fL -o ~/.config/zellij/plugins/"$name" "$base/$name"
   curl -fL -o ~/.config/zellij/plugins/"${name}.sigstore.jsonl" "$base/${name}.sigstore.jsonl"
   ```

   If you have the `gh` command, verify the file build attestation:

   ```sh
   gh attestation verify ~/.config/zellij/plugins/"$name" \
     -R fulldecent/zellij-compact-bar-ng \
     --bundle ~/.config/zellij/plugins/"${name}.sigstore.jsonl"
   ```

2. Start a new session (or tab if already in Zellij):

   ```sh
   zellij --layout-string 'layout {
       pane
       pane size=1 borderless=true {
           plugin location="file:~/.config/zellij/plugins/com.github.fulldecent.zellij-compact-bar-ng.wasm"
       }
   }'
   ```

   Grant `ReadApplicationState`, `ChangeApplicationState`, and `RunActionsAsUser` when Zellij asks. Click that row and press `y`.

3. Click a tab. Click `+`. Click a key hint. Narrow the terminal and watch the hints shorten. The tab names stay whole until there is no hint left to shorten.

## Installation

Use [Try it out](#try-it-out) to download the plugin (this also confirms permissions).

Open your `~/.config/zellij/config.kdl` and replace the existing `compact-bar` line with:

```kdl
compact-bar location="file:~/.config/zellij/plugins/com.github.fulldecent.zellij-compact-bar-ng.wasm"
```

Quit and reopen Zellij to load your new compact bar.

If you already have `layouts/default.kdl`, keep `plugin location="compact-bar"` on that row.

To upgrade: repeat [Try it out](#try-it-out) > step 1, then start a new session.

## Usage

While using this session, try creating new tabs, renaming them and changing the width of your terminal emulator to experience the different available rendering.

### Uninstall

Edit your `~/.config/zellij/config.kdl` again and replace the `compact-bar` line with:

```kdl
compact-bar location="zellij:compact-bar"
```

## Development

Thank you for taking an interest in improving Zellij Compact Bar Next-Generation.

You will need [rustup](https://rust-lang.github.io/rustup/), Git and your platform's native build tools. This project pins the toolchain in [rust-toolchain.toml](rust-toolchain.toml); rustup reads that file and installs matching `rustc`, Cargo, rustfmt, Clippy and the `wasm32-wasip1` target.

A packaged `rustc` or `cargo` from apt, dnf or Homebrew does not apply `rust-toolchain.toml`. Install rustup, then invoke the toolchain rustup selected:

```sh
"$(rustup which rustc)" --version
"$(rustup which cargo)" --version
```

`rustup which` prints the binary for the active toolchain (from `rust-toolchain.toml` in this directory). Quoting the command substitution runs that binary even when another `rustc` or `cargo` is earlier on `PATH`. That Cargo still searches `PATH` for `rustc`. A packaged `rustc` has no `wasm32-wasip1` standard library, so the build stops with E0463, `can't find crate for core`. [.cargo/config.toml](.cargo/config.toml) sets Cargo's [`rustc-wrapper`](https://doc.rust-lang.org/cargo/reference/config.html#buildrustc-wrapper) to [tools/rustc-wrapper](tools/rustc-wrapper), which runs the channel in `rust-toolchain.toml` when that happens. Doctests use [tools/rustdoc](tools/rustdoc) ([`build.rustdoc`](https://doc.rust-lang.org/cargo/reference/config.html#buildrustdoc)). A packaged `rustdoc` reports E0514 for rlibs that toolchain built.

### Linux

On Ubuntu 22.04+ or Debian 12+:

```sh
sudo apt update
sudo apt install git build-essential rustup
```

On Fedora:

```sh
sudo dnf install git gcc rustup
```

If your distribution has no `rustup` package, follow [other rustup installation methods](https://rust-lang.github.io/rustup/installation/other.html).

### macOS

```sh
xcode-select --install
brew install git rustup
```

Homebrew's `rust` formula is a standalone compiler. It does not honor `rust-toolchain.toml`. Use `rustup` instead.

### Windows

```powershell
winget install --exact --id Git.Git
winget install --exact --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --exact --id Rustlang.Rustup
```

Clone the project and build:

```sh
git clone https://github.com/fulldecent/zellij-compact-bar-ng.git
cd zellij-compact-bar-ng
"$(rustup which rustc)" --version
"$(rustup which cargo)" --version
"$(rustup which cargo)" check --locked
"$(rustup which cargo)" build --locked
```

> [!NOTE]
> Run those lines from this repository. `rust-toolchain.toml` is how the version is selected. If a check fails with “compiled by an incompatible version of rustc”, another compiler already wrote `target/`. That directory is gitignored. Remove it (`rm -rf target`) and run the same four lines again.

Run this directly with:

```sh
zellij --layout layout/dev.kdl
```

### Testing

```sh
"$(rustup which cargo)" test --locked --target "$("$(rustup which rustc)" -vV | sed -n 's/^host: //p')"
"$(rustup which cargo)" fmt --all -- --check
"$(rustup which cargo)" clippy --all-targets --locked -- -D warnings
"$(rustup which cargo)" build --release --locked
```

`cargo test` runs the Rust tests in `src/` and `tests/widths.rs`. Pass `--target` with the host triple from `rustup which rustc`. The target in [.cargo/config.toml](.cargo/config.toml) is `wasm32-wasip1`, and that wasm cannot run these tests. [tests/snapshot.sh](tests/snapshot.sh) builds the release wasm, runs every script in [shots/](shots/), and exits non-zero unless those bytes match the committed `shots/*.ansi.txt` files. SVG is traced. `diff` does not compare it. The picture is committed as [screenshot.svg](screenshot.svg) at the repository root.

The host is [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot) 0.2.2. Install that published crate once. `cargo install` compiles it and puts `zellij-plugin-snapshot` on `PATH`. `[dependencies]` and `[dev-dependencies]` link a library into the plugin wasm or into `cargo test`. This host is a command, so those fields leave it uninstalled.

```sh
rustup run "$(rustup show active-toolchain | awk '{print $1}')" cargo install zellij-plugin-snapshot --version 0.2.2 --locked
sh tests/snapshot.sh
```

```sh
npx prettier@latest --check . --write
npx markdownlint-cli@latest "**/*.md" --fix
```

### Releases

Use `fix:`, `feat:` or `BREAKING CHANGE:` in your commit messages. When you push commits to `main`, the release workflow will create a pull request which bumps the version and kicks off a release when you merge.

The [release workflow](.github/workflows/release.yml) uses Release Please's `rust` release type. That pull request bumps the version in [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock). The package name is `plugin`. `include-component-in-tag: false` keeps the tag `vMAJOR.MINOR.PATCH`. A tag shaped `plugin-v1.2.3` does not pass the publish step.

The released binary is named `com.github.fulldecent.zellij-compact-bar-ng.wasm`.

### Maintenance

The project administrator completes these maintenance tasks each month.

1. Identify external Actions in [.github/workflows](./.github/workflows) and update them when it is safe.
1. Review the Rust toolchain in `rust-toolchain.toml`.
1. Review [zellij-plugin-snapshot](https://github.com/fulldecent/zellij-plugin-snapshot) releases. Update every `cargo install zellij-plugin-snapshot --version` line. Regenerate each `shots/*.ansi.txt` and its SVG when the host output changes. Commit [screenshot.svg](screenshot.svg) at the repository root. Commit the other SVG files beside their YAML scripts.

## Project scope

This compact bar is intended to replace the built-in Zellij compact bar ([`default-plugins/compact-bar` at v0.45.1](https://github.com/zellij-org/zellij/tree/v0.45.1/default-plugins/compact-bar)). In this interest, we limit this project only to the following features which we consider to be the highest priority.

* One row: Zellij, the session name, and the mode, then ribbon tabs, then the key hints
* The mode word uses the compact-bar text, spacing, and emphasis colors
* Ribbon tabs that keep their names while the hints can still shorten
* Key hints that take only the columns they need, and give those columns back before a tab name is shortened
* Theme colors from the active Zellij session

Out of scope:

* No `Ctrl +` label and no Alt group on the hint block
* No swap-layout indicator, tooltip, or nested-session hint from the built-in compact bar
* No separate color scheme file
* No format-string language or extra widgets
* Layouts do not point at an HTTPS plugin URL

## References

1. We use "Title Case" only for proper nouns, this includes the name of our project.
1. This project is built based on [best practices documented in zellij-plugin-template](https://github.com/fulldecent/zellij-plugin-template), release v0.5.0.
1. This project is built based on [best practices documented in rust-template](https://github.com/fulldecent/rust-template/), release v1.0.0.
1. This project is built based on [best practices documented in project-template](https://github.com/fulldecent/project-template), release v1.3.0.
1. Zellij offers another installation method that points your layout configuration to an HTTPS URL. We consider that feature wrong and deprecated.
1. We speak of the official rustup installation recommendation as ugly. [Reported upstream](https://github.com/rust-lang/rust/issues/163468).
1. Render snapshots follow [Zellij Plugin Snapshot](https://github.com/fulldecent/zellij-plugin-snapshot) 0.2.2. [tests/snapshot.sh](tests/snapshot.sh) compares every `shots/*.yaml` against `shots/{name}.ansi.txt`. [screenshot.svg](screenshot.svg) is the picture in this README.
1. The mode word follows [Zellij’s `default-plugins/compact-bar` `create_mode_part`](https://github.com/zellij-org/zellij/blob/v0.45.1/default-plugins/compact-bar/src/line.rs) at tag `v0.45.1` (MIT): a space, `Debug` of the mode in uppercase, and a space. Normal uses `text_unselected.emphasis_2`, locked uses `emphasis_3`, and every other mode uses `emphasis_0`. The word is bold on `text_unselected.background`. The `Zellij` word and the session name use the same padding and weight as that file’s `Zellij` and `(session)` parts.
1. The ribbon tabs follow [zellij-tab-bar-ng](https://github.com/fulldecent/zellij-tab-bar-ng). That fitting rule follows [Zellij’s `default-plugins/tab-bar`](https://github.com/zellij-org/zellij/tree/v0.45.1/default-plugins/tab-bar) at tag `v0.45.1` (MIT): keep the active tab, add neighbors toward the shorter side, and collapse the rest to `← +N` / `+N →`. This row draws the ribbons only. The session placard stays in the compact-bar prefix.
1. The key hints follow [zellij-status-bar-ng](https://github.com/fulldecent/zellij-status-bar-ng), which follows [Zellij’s `default-plugins/status-bar`](https://github.com/zellij-org/zellij/tree/v0.45.1/default-plugins/status-bar) at tag `v0.45.1` (MIT). This row leaves out the `Ctrl +` label and the Alt group, and it does not pad the hint block out to the edge of the screen.
1. Release Please's [`rust` release type](https://github.com/googleapis/release-please/blob/main/docs/customizing.md) bumps [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock). [`include-component-in-tag: false`](https://github.com/googleapis/release-please/blob/main/docs/manifest-releaser.md) keeps the GitHub tag `vMAJOR.MINOR.PATCH`.
1. [tests/snapshot.sh](tests/snapshot.sh) runs Cargo with `rustup run` and the active toolchain. `rustup which cargo` is the inner toolchain binary. On GitHub Actions that binary was the default stable toolchain, which did not have `wasm32-wasip1`. `rustup run` does not put `rustc` on `PATH`. [tools/rustc-wrapper](tools/rustc-wrapper) is Cargo's `rustc-wrapper`. It runs the channel in [rust-toolchain.toml](rust-toolchain.toml) when `PATH`'s `rustc` has no `wasm32-wasip1` sysroot. [tools/rustdoc](tools/rustdoc) is `build.rustdoc`, so doctests load those rlibs. On macOS that compiler's `rust-lld` does not find `libLLVM.dylib` on its recorded rpath, so the wrapper sets `DYLD_LIBRARY_PATH` for wasm links ([rust-lang/rust#76921](https://github.com/rust-lang/rust/issues/76921)).
1. The Rust ignore rules in [.gitignore](.gitignore) come from [GitHub's Rust gitignore](https://github.com/github/gitignore/blob/main/Rust.gitignore).
1. This project is released under the [MIT license](./LICENSE.md).
