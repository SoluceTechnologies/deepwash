# apt + Homebrew Distribution — Design

**Date:** 2026-07-05
**Status:** Approved (design), pending implementation plan
**Author:** Charles Gauthereau

## Goal

Distribute `deepwash` through two additional channels alongside the existing
crates.io publish:

1. **Homebrew** — `brew install soluce-technologies/deepwash/deepwash` (macOS only)
2. **apt** — a true signed APT repository so users can
   `apt update && apt install deepwash`, plus raw `.deb` files attached to each
   GitHub Release.

Both channels are driven automatically from the existing release flow
(`release.yml`, triggered on PR merge to `main` via `release-it`).

## Non-goals

- homebrew-core submission (own tap only; may apply later once notable).
- Launchpad PPA.
- brew-on-Linux (apt covers Linux; formula is macOS-only).
- Windows distribution.

## Decisions

| Topic | Decision |
|-------|----------|
| apt UX | True signed apt repo **and** `.deb` assets on the GitHub Release |
| apt hosting | GitHub Pages, on the **main `deepwash` repo's `gh-pages` branch** |
| brew | New tap repo `SoluceTechnologies/homebrew-deepwash`, macOS-only formula |
| Build targets | `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` |
| Cross-compile | `cargo-zigbuild` for both Linux gnu targets (no Docker); native `rustup target add` for macOS on a `macos-14` runner |
| apt metadata | `apt-ftparchive` (from `apt-utils`) |
| GPG signing | Dedicated signing key; private key + passphrase in GitHub Secrets; public key committed to the apt repo |

## Architecture

Five components, wired into `release.yml` after the existing crate publish.

### §1 — Reusable build workflow: `build-binaries.yml`

`workflow_call` reusable workflow. Inputs: `version`, `ref` (the release tag).

Matrix over 4 targets. Each job:
1. Checkout `ref`.
2. Install Rust + target.
3. Build stripped release binary:
   - macOS (`macos-14` runner): `rustup target add <triple>` then
     `cargo build --release --locked --target <triple>`.
   - Linux (`ubuntu-latest`): `cargo-zigbuild` build for the gnu target.
4. Package `deepwash-<version>-<target>.tar.gz` containing the `deepwash`
   binary (+ `README.md`, `LICENSE`).
5. Emit `<archive>.sha256`.
6. Upload both as assets to the draft GitHub Release for `ref`.

Outputs the per-target sha256 values (used by the brew job) — either via job
outputs or by having the brew job re-download and hash the published assets.
**Chosen:** brew job re-hashes the published `.tar.gz` assets (single source of
truth, avoids output plumbing across matrix legs).

**Interface:** given a tag, produces 4 signed-by-sha tarballs + 2 `.deb` files as
release assets. Depends on: `Cargo.toml`, source, `cargo-zigbuild`.

### §2 — `.deb` build

Add `[package.metadata.deb]` to `Cargo.toml`:

```toml
[package.metadata.deb]
maintainer = "Charles Gauthereau <charles.gauthereau@soluce-technologies.com>"
copyright = "2026, Soluce Technologies"
license-file = ["LICENSE", "0"]
section = "utils"
priority = "optional"
extended-description = "A Command Line Interface to clean your machine (docker...)."
```

Build step (part of `build-binaries.yml`, Linux legs only):
`cargo deb --no-build --target <triple>` reusing the already-built binary, for
`amd64` (x86_64) and `arm64` (aarch64). Resulting `.deb` files are uploaded as
release assets and passed to §3.

> A `LICENSE` file must be added to the repo root (currently only declared as
> `license = "MIT"` in `Cargo.toml`). This is required by `cargo-deb` and by the
> brew formula/tarball. Adding it is part of this work.

### §3 — apt repo publish (`gh-pages` of main repo)

New job in the release flow. Steps:
1. `apt-get install apt-utils gnupg`.
2. Checkout `gh-pages` branch (create if missing).
3. Import GPG private key from `secrets.APT_GPG_PRIVATE_KEY`
   (base64-encoded), passphrase from `secrets.APT_GPG_PASSPHRASE`.
4. Copy new `.deb` files into `pool/main/`.
5. Regenerate metadata:
   - `apt-ftparchive packages pool/ > dists/stable/main/binary-amd64/Packages`
     (and `binary-arm64`), gzip them.
   - `apt-ftparchive release dists/stable > dists/stable/Release`.
   - Sign: `gpg --clearsign -o dists/stable/InRelease dists/stable/Release`
     and `gpg -abs -o dists/stable/Release.gpg dists/stable/Release`.
6. Ensure public key present at repo root:
   `deepwash-archive-keyring.gpg`.
7. Add `.nojekyll` (Pages must not process the tree).
8. Commit + push `gh-pages`.

Resulting layout served at
`https://soluce-technologies.github.io/deepwash/`:

```
/deepwash-archive-keyring.gpg
/.nojekyll
/pool/main/deepwash_<version>_amd64.deb
/pool/main/deepwash_<version>_arm64.deb
/dists/stable/Release
/dists/stable/InRelease
/dists/stable/Release.gpg
/dists/stable/main/binary-amd64/Packages{,.gz}
/dists/stable/main/binary-arm64/Packages{,.gz}
```

**User install:**
```bash
curl -fsSL https://soluce-technologies.github.io/deepwash/deepwash-archive-keyring.gpg \
  | sudo tee /usr/share/keyrings/deepwash.gpg >/dev/null
echo "deb [signed-by=/usr/share/keyrings/deepwash.gpg] https://soluce-technologies.github.io/deepwash stable main" \
  | sudo tee /etc/apt/sources.list.d/deepwash.list
sudo apt update && sudo apt install deepwash
```

**Security notes:**
- Signing key is a dedicated key, not a personal key. Generated locally by the
  maintainer; only the private key + passphrase enter GitHub Secrets. Claude
  never handles the private key material.
- `gh-pages` accumulates the `pool/` — old versions stay installable. Pool grows
  over time; acceptable.
- The main repo Pages site is now the apt repo. If a project website is wanted
  later, it must coexist under the same Pages root or move to a subpath.

### §4 — Homebrew tap update

New repo `SoluceTechnologies/homebrew-deepwash` with `Formula/deepwash.rb`.
macOS-only, arch-split on the two darwin tarballs:

```ruby
class Deepwash < Formula
  desc "A Command Line Interface to clean your machine (docker...)"
  homepage "https://github.com/Soluce-Technologies/deepwash"
  version "X.Y.Z"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Soluce-Technologies/deepwash/releases/download/#{version}/deepwash-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    end
    on_intel do
      url "https://github.com/Soluce-Technologies/deepwash/releases/download/#{version}/deepwash-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  def install
    bin.install "deepwash"
  end

  test do
    system "#{bin}/deepwash", "--version"
  end
end
```

CI job (in release flow): re-download the two published darwin tarballs, compute
sha256, render the formula, commit + push to the tap repo using the existing
GitHub App token mechanism (`APP_ID` / `APP_PRIVATE_KEY`). The App must be
installed on `homebrew-deepwash` with contents:write.

**User install:** `brew install soluce-technologies/deepwash/deepwash`

### §5 — Wiring into `release.yml`

Current chain:
`check-skip → create-release → publish-crate-registry → finalize-release`
(`finalize` un-drafts the release).

New chain (binaries must attach **before** un-draft):

```
check-skip
  → create-release
      → publish-crate-registry ─┐
      → build-binaries ─────────┤   (needs create-release: version + draft tag)
build-binaries
  → publish-apt      ─┐
  → update-brew      ─┤          (parallel; both need the built assets)
finalize-release  (needs: publish-crate-registry, publish-apt, update-brew)
```

`build-binaries` uploads to the **draft** release created by `release-it`, so it
needs the draft tag. `finalize-release` un-drafts only after all assets are
attached and channels published.

## Testing / verification

- `build-binaries.yml`: verify each `.tar.gz` extracts and the binary runs
  `--version` (the two Linux binaries can be smoke-tested in CI; macOS binaries
  run on the macOS runner).
- `.deb`: `dpkg-deb --info` + `dpkg -c` sanity check in CI.
- apt repo: after publish, a CI verification step (or manual first-run) that adds
  the repo in a Docker `debian:stable` container and runs `apt install deepwash`.
- brew: `brew audit --strict deepwash` + `brew install` on the macOS runner in a
  tap CI (optional, can be manual first time).
- The first release after this lands should be validated manually end-to-end on
  both channels before announcing.

## Prerequisites / one-time setup (maintainer, outside CI)

1. Generate dedicated GPG signing key locally; add
   `APT_GPG_PRIVATE_KEY` (base64) + `APT_GPG_PASSPHRASE` to repo secrets; export
   public key to commit as `deepwash-archive-keyring.gpg`.
2. Create `SoluceTechnologies/homebrew-deepwash` repo; install the release
   GitHub App on it with contents:write.
3. Enable GitHub Pages on the `deepwash` repo, source = `gh-pages` branch.
4. Add `LICENSE` file to repo root.

## Rollout order

1. Add `LICENSE` + `[package.metadata.deb]` (no behavior change).
2. Add `build-binaries.yml`; test via a pre-release/manual dispatch.
3. Stand up apt publish against `gh-pages`; validate in a Debian container.
4. Create tap repo + `update-brew` job; validate `brew install`.
5. Wire all into `release.yml`; do one full guarded release.
