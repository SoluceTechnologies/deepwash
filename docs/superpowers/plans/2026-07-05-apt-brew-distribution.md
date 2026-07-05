# apt + Homebrew Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Distribute `deepwash` via a signed apt repo (GitHub Pages) and a macOS Homebrew tap, driven automatically from the existing `release.yml` flow, alongside the current crates.io publish.

**Architecture:** A reusable `build-binaries.yml` cross-compiles 4 targets (macOS arm64/x86_64 via native runner, Linux x86_64/arm64 via `cargo-zigbuild`) and uploads `.tar.gz` + `.deb` assets to the draft GitHub Release. Two publish jobs then consume those assets: `publish-apt` regenerates a GPG-signed apt repo on the `gh-pages` branch, and `update-brew` renders + pushes the formula to `SoluceTechnologies/homebrew-deepwash`. All jobs slot into `release.yml` before the existing `finalize-release` un-drafts the release.

**Tech Stack:** GitHub Actions, `cargo-zigbuild`, `cargo-deb`, `apt-ftparchive` (apt-utils), GnuPG, Homebrew Ruby formula, GitHub App token (existing), GitHub Pages.

**Reference spec:** `docs/superpowers/specs/2026-07-05-apt-brew-distribution-design.md`

---

## Nature of this plan

Most artifacts are CI workflow YAML and packaging config that cannot be unit-tested locally in the classic TDD sense. Where a step *can* be verified locally (a `.deb` builds, a tarball extracts, a formula passes `brew audit`), the plan installs the tool and runs it. Where verification is only meaningful in CI or a clean container (apt repo install, macOS build), the plan uses a Docker container check or an explicit first-release manual guard. "Expected output" is given for every runnable command.

Assume the engineer has: a working Rust toolchain (`rustc 1.94+`), Docker, `git`, and repo push access. Homebrew + a macOS machine are needed only for the local brew audit step (Task 14) — skip-and-note if unavailable.

## File Structure

**Created:**
- `LICENSE` — MIT license text (required by cargo-deb + brew tarball).
- `.github/workflows/build-binaries.yml` — reusable build/package workflow.
- `.github/workflows/publish-apt.yml` — reusable apt-repo publish workflow.
- `.github/workflows/update-brew.yml` — reusable brew-formula update workflow.
- `packaging/apt/verify-install.sh` — container smoke test for the apt repo.
- `packaging/brew/deepwash.rb.tmpl` — formula template rendered by CI.
- `docs/DISTRIBUTION.md` — user-facing install instructions (apt + brew).

**Modified:**
- `Cargo.toml` — add `[package.metadata.deb]`.
- `.github/workflows/release.yml` — wire in the three new jobs, reorder before finalize.
- `README.md` — add apt/brew install snippets, link to `docs/DISTRIBUTION.md`.

**External (created outside this repo, one-time, see Task 0):**
- `SoluceTechnologies/homebrew-deepwash` repo with `Formula/deepwash.rb`.

---

## Task 0: One-time maintainer prerequisites (manual, do first)

These are performed by the maintainer (Charles), not by an automated worker, because they involve secret material and external repos/settings. **The CI tasks below assume these exist.** Record completion by checking the boxes.

**Files:** none in-repo except the committed public key (Task 5).

- [ ] **Step 1: Generate a dedicated apt signing key**

Run locally (not on CI):
```bash
gpg --batch --gen-key <<'EOF'
%no-protection
Key-Type: eddsa
Key-Curve: ed25519
Subkey-Type: ecdh
Subkey-Curve: cv25519
Name-Real: deepwash apt repository
Name-Email: tech.soluce.technologies@gmail.com
Expire-Date: 0
%commit
EOF
```
Expected: `gpg: key <KEYID> marked as ultimately trusted`. Note the `<KEYID>`.

> A passphrase-protected key is preferred over `%no-protection`. If you use a passphrase, set `APT_GPG_PASSPHRASE` (Step 3); if not, that secret may be an empty string and the import step skips `--passphrase`.

- [ ] **Step 2: Export the public key (armored) for the repo**

```bash
gpg --export <KEYID> > /tmp/deepwash-archive-keyring.gpg
```
Keep `/tmp/deepwash-archive-keyring.gpg` — it gets committed in Task 5.
Expected: a non-empty binary file (`file` reports `PGP/GPG key public ring`).

- [ ] **Step 3: Add GitHub Secrets on `Soluce-Technologies/deepwash`**

Export the private key base64-encoded and store it, plus passphrase:
```bash
gpg --export-secret-keys <KEYID> | base64 | pbcopy   # macOS; use base64 -w0 on Linux
```
In repo Settings → Secrets and variables → Actions, add:
- `APT_GPG_PRIVATE_KEY` = the base64 blob above.
- `APT_GPG_PASSPHRASE` = the key passphrase (empty string if none).

Expected: both secrets listed. **Claude/automated workers never see this material.**

- [ ] **Step 4: Create the Homebrew tap repo**

Create public repo `SoluceTechnologies/homebrew-deepwash` with a `Formula/` directory (a placeholder `Formula/.gitkeep` is fine). Install the existing release GitHub App (the one behind `APP_ID` / `APP_PRIVATE_KEY`) on this new repo with **Contents: read and write** permission.

Expected: `git clone https://github.com/SoluceTechnologies/homebrew-deepwash` succeeds and the App appears in the repo's Installed GitHub Apps.

- [ ] **Step 5: Enable GitHub Pages**

In `Soluce-Technologies/deepwash` Settings → Pages, set Source = **Deploy from a branch**, Branch = `gh-pages`, folder = `/ (root)`. The branch may not exist yet — Task 8 creates it; re-confirm Pages after the first publish.

Expected: Pages settings saved pointing at `gh-pages`.

---

## Task 1: Add LICENSE file

**Files:**
- Create: `LICENSE`

- [ ] **Step 1: Create the MIT license file**

Create `LICENSE` (fill the year and holder to match `Cargo.toml` authors):
```
MIT License

Copyright (c) 2026 Charles Gauthereau / Soluce Technologies

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Verify cargo still packages cleanly**

Run: `cargo package --list --allow-dirty | grep -q '^LICENSE$' && echo OK`
Expected: `OK` (the LICENSE is now included in the crate package).

- [ ] **Step 3: Commit**

```bash
git add LICENSE
git commit -m "docs: add MIT LICENSE file"
```

---

## Task 2: Add cargo-deb metadata to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Install cargo-deb locally**

Run: `cargo install cargo-deb --locked`
Expected: ends with `Installed package \`cargo-deb ...\``. Verify: `cargo deb --version` prints a version.

- [ ] **Step 2: Add the metadata section**

Append to `Cargo.toml`:
```toml
[package.metadata.deb]
maintainer = "Charles Gauthereau <charles.gauthereau@soluce-technologies.com>"
copyright = "2026, Soluce Technologies"
license-file = ["LICENSE", "0"]
section = "utils"
priority = "optional"
extended-description = "A Command Line Interface to clean your machine (docker...)."
```

- [ ] **Step 3: Build a .deb for the host arch to validate metadata**

Run: `cargo deb`
Expected: ends with a path like `target/debian/deepwash_1.2.0-1_<arch>.deb`. No `missing license-file` or metadata errors.

- [ ] **Step 4: Inspect the package**

Run: `dpkg-deb --info target/debian/deepwash_*.deb` (Linux) — on macOS use `ar t target/debian/deepwash_*.deb` to at least confirm structure.
Expected (Linux): control fields show `Package: deepwash`, `Section: utils`, `Priority: optional`, `Maintainer: Charles Gauthereau ...`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml
git commit -m "build: add cargo-deb package metadata"
```

---

## Task 3: Reusable build-binaries workflow — skeleton + macOS legs

**Files:**
- Create: `.github/workflows/build-binaries.yml`

- [ ] **Step 1: Create the workflow with macOS matrix legs**

Create `.github/workflows/build-binaries.yml`:
```yaml
name: Build release binaries

on:
  workflow_call:
    inputs:
      version:
        description: "Version being released (e.g. 1.3.0)"
        required: true
        type: string
      ref:
        description: "Git ref (tag) to build from"
        required: true
        type: string

permissions:
  contents: write

jobs:
  macos:
    name: macOS ${{ matrix.target }}
    runs-on: macos-14
    strategy:
      fail-fast: false
      matrix:
        target:
          - aarch64-apple-darwin
          - x86_64-apple-darwin
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.ref }}

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2

      - name: Build
        run: cargo build --release --locked --target ${{ matrix.target }}

      - name: Package tarball
        run: |
          BIN=target/${{ matrix.target }}/release/deepwash
          strip "$BIN" || true
          NAME=deepwash-${{ inputs.version }}-${{ matrix.target }}
          mkdir -p "dist/$NAME"
          cp "$BIN" "dist/$NAME/deepwash"
          cp README.md LICENSE "dist/$NAME/"
          tar -C dist -czf "dist/$NAME.tar.gz" "$NAME"
          shasum -a 256 "dist/$NAME.tar.gz" | awk '{print $1}' > "dist/$NAME.tar.gz.sha256"

      - name: Upload to release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          gh release upload "${{ inputs.ref }}" \
            dist/deepwash-${{ inputs.version }}-${{ matrix.target }}.tar.gz \
            dist/deepwash-${{ inputs.version }}-${{ matrix.target }}.tar.gz.sha256 \
            --clobber
```

- [ ] **Step 2: Validate YAML syntax**

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/build-binaries.yml')); print('YAML OK')"`
Expected: `YAML OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/build-binaries.yml
git commit -m "ci: add build-binaries workflow (macOS legs)"
```

---

## Task 4: build-binaries — Linux legs (tarball + .deb via cargo-zigbuild)

**Files:**
- Modify: `.github/workflows/build-binaries.yml`

- [ ] **Step 1: Add the Linux job**

Add this job under `jobs:` in `build-binaries.yml` (sibling of `macos`):
```yaml
  linux:
    name: Linux ${{ matrix.target }}
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            deb_arch: amd64
          - target: aarch64-unknown-linux-gnu
            deb_arch: arm64
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.ref }}

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2

      - name: Install zig + cargo tools
        run: |
          pip install ziglang
          cargo install cargo-zigbuild --locked
          cargo install cargo-deb --locked

      - name: Build
        run: cargo zigbuild --release --locked --target ${{ matrix.target }}

      - name: Package tarball
        run: |
          BIN=target/${{ matrix.target }}/release/deepwash
          NAME=deepwash-${{ inputs.version }}-${{ matrix.target }}
          mkdir -p "dist/$NAME"
          cp "$BIN" "dist/$NAME/deepwash"
          cp README.md LICENSE "dist/$NAME/"
          tar -C dist -czf "dist/$NAME.tar.gz" "$NAME"
          sha256sum "dist/$NAME.tar.gz" | awk '{print $1}' > "dist/$NAME.tar.gz.sha256"

      - name: Build .deb
        run: |
          cargo deb --no-build --target ${{ matrix.target }} --output dist/
          ls -1 dist/*.deb

      - name: Smoke test binary
        run: |
          if [ "${{ matrix.deb_arch }}" = "amd64" ]; then
            ./target/${{ matrix.target }}/release/deepwash --version
          fi

      - name: Upload to release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          gh release upload "${{ inputs.ref }}" \
            dist/deepwash-${{ inputs.version }}-${{ matrix.target }}.tar.gz \
            dist/deepwash-${{ inputs.version }}-${{ matrix.target }}.tar.gz.sha256 \
            dist/*_${{ matrix.deb_arch }}.deb \
            --clobber
```

> `cargo deb --no-build` reuses the zigbuild output. If cargo-deb cannot locate the cross-built binary, drop `--no-build` and let cargo-deb build for the target (it will invoke the linker cargo-zigbuild configured). Verify locally in Step 3 before relying on `--no-build`.

- [ ] **Step 2: Validate YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/build-binaries.yml')); print('YAML OK')"`
Expected: `YAML OK`

- [ ] **Step 3: Locally rehearse the cross .deb (x86_64 host → arm64)**

On a Linux host (or `docker run --rm -it -v "$PWD":/w -w /w rust:1.94 bash`):
```bash
pip install ziglang && cargo install cargo-zigbuild cargo-deb --locked
rustup target add aarch64-unknown-linux-gnu
cargo zigbuild --release --locked --target aarch64-unknown-linux-gnu
cargo deb --no-build --target aarch64-unknown-linux-gnu --output dist/
ls dist/*_arm64.deb
```
Expected: `dist/deepwash_1.2.0-1_arm64.deb` exists. If `--no-build` fails, note it and remove the flag in Step 1.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/build-binaries.yml
git commit -m "ci: add Linux build legs with zigbuild + cargo-deb"
```

---

## Task 5: Commit the apt public key + repo scaffolding files

**Files:**
- Create: `deepwash-archive-keyring.gpg` (from Task 0 Step 2)
- Create: `packaging/apt/verify-install.sh`

- [ ] **Step 1: Copy the exported public key into the repo root**

```bash
cp /tmp/deepwash-archive-keyring.gpg ./deepwash-archive-keyring.gpg
```
Expected: `file deepwash-archive-keyring.gpg` → `PGP/GPG key public ring`. This file is served verbatim from Pages and is what users import.

- [ ] **Step 2: Create the apt install verification script**

Create `packaging/apt/verify-install.sh`:
```bash
#!/usr/bin/env bash
# Smoke-tests the published apt repo inside a clean Debian container.
# Usage: packaging/apt/verify-install.sh [BASE_URL]
set -euo pipefail
BASE_URL="${1:-https://soluce-technologies.github.io/deepwash}"

docker run --rm "debian:stable" bash -c "
  set -euo pipefail
  apt-get update -qq && apt-get install -y -qq curl gnupg ca-certificates >/dev/null
  curl -fsSL '${BASE_URL}/deepwash-archive-keyring.gpg' -o /usr/share/keyrings/deepwash.gpg
  echo 'deb [signed-by=/usr/share/keyrings/deepwash.gpg] ${BASE_URL} stable main' \
    > /etc/apt/sources.list.d/deepwash.list
  apt-get update -qq
  apt-get install -y -qq deepwash
  deepwash --version
"
echo 'apt install smoke test PASSED'
```

- [ ] **Step 3: Make it executable + syntax check**

```bash
chmod +x packaging/apt/verify-install.sh
bash -n packaging/apt/verify-install.sh && echo "bash syntax OK"
```
Expected: `bash syntax OK`

- [ ] **Step 4: Commit**

```bash
git add deepwash-archive-keyring.gpg packaging/apt/verify-install.sh
git commit -m "ci: add apt public key and install verification script"
```

---

## Task 6: Reusable publish-apt workflow

**Files:**
- Create: `.github/workflows/publish-apt.yml`

- [ ] **Step 1: Create the workflow**

Create `.github/workflows/publish-apt.yml`:
```yaml
name: Publish apt repo

on:
  workflow_call:
    inputs:
      version:
        required: true
        type: string
      ref:
        description: "Release tag holding the .deb assets"
        required: true
        type: string
    secrets:
      APT_GPG_PRIVATE_KEY:
        required: true
      APT_GPG_PASSPHRASE:
        required: false

permissions:
  contents: write

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout main (for the public keyring source)
        uses: actions/checkout@v4
        with:
          ref: ${{ inputs.ref }}
          path: source

      - name: Install tooling
        run: sudo apt-get update -qq && sudo apt-get install -y -qq apt-utils gnupg

      - name: Import signing key
        run: |
          echo "${{ secrets.APT_GPG_PRIVATE_KEY }}" | base64 -d | gpg --batch --import
          gpg --list-secret-keys --keyid-format=long
          KEYID=$(gpg --list-secret-keys --with-colons | awk -F: '/^sec:/ {print $5; exit}')
          echo "GPG_KEYID=$KEYID" >> "$GITHUB_ENV"

      - name: Download .deb assets from the release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          mkdir -p incoming
          gh release download "${{ inputs.ref }}" \
            --repo "${{ github.repository }}" \
            --pattern '*.deb' --dir incoming
          ls -1 incoming/*.deb

      - name: Checkout gh-pages (create if missing)
        run: |
          git clone --branch gh-pages --single-branch \
            "https://x-access-token:${{ github.token }}@github.com/${{ github.repository }}.git" pages \
            || (git clone "https://x-access-token:${{ github.token }}@github.com/${{ github.repository }}.git" pages \
                && cd pages && git checkout --orphan gh-pages && git rm -rf . >/dev/null 2>&1 || true)

      - name: Assemble pool + metadata
        run: |
          cd pages
          mkdir -p pool/main dists/stable/main/binary-amd64 dists/stable/main/binary-arm64
          cp ../incoming/*.deb pool/main/

          for arch in amd64 arm64; do
            apt-ftparchive --arch "$arch" packages pool/ \
              > "dists/stable/main/binary-$arch/Packages"
            gzip -9c "dists/stable/main/binary-$arch/Packages" \
              > "dists/stable/main/binary-$arch/Packages.gz"
          done

          apt-ftparchive \
            -o APT::FTPArchive::Release::Origin="deepwash" \
            -o APT::FTPArchive::Release::Label="deepwash" \
            -o APT::FTPArchive::Release::Suite="stable" \
            -o APT::FTPArchive::Release::Codename="stable" \
            -o APT::FTPArchive::Release::Components="main" \
            -o APT::FTPArchive::Release::Architectures="amd64 arm64" \
            release dists/stable > dists/stable/Release

      - name: Sign Release
        env:
          PASSPHRASE: ${{ secrets.APT_GPG_PASSPHRASE }}
        run: |
          cd pages
          GPG_OPTS=(--batch --yes --pinentry-mode loopback)
          if [ -n "${PASSPHRASE:-}" ]; then GPG_OPTS+=(--passphrase "$PASSPHRASE"); fi
          gpg "${GPG_OPTS[@]}" --clearsign -o dists/stable/InRelease dists/stable/Release
          gpg "${GPG_OPTS[@]}" -abs -o dists/stable/Release.gpg dists/stable/Release

      - name: Publish keyring + disable Jekyll
        run: |
          cd pages
          cp ../source/deepwash-archive-keyring.gpg ./deepwash-archive-keyring.gpg
          touch .nojekyll

      - name: Commit + push gh-pages
        run: |
          cd pages
          git config user.name 'github-actions[bot]'
          git config user.email 'github-actions[bot]@users.noreply.github.com'
          git add -A
          git commit -m "apt: publish deepwash ${{ inputs.version }}" || echo "nothing to commit"
          git push origin gh-pages
```

- [ ] **Step 2: Validate YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/publish-apt.yml')); print('YAML OK')"`
Expected: `YAML OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/publish-apt.yml
git commit -m "ci: add publish-apt workflow (signed gh-pages repo)"
```

---

## Task 7: Brew formula template + reusable update-brew workflow

**Files:**
- Create: `packaging/brew/deepwash.rb.tmpl`
- Create: `.github/workflows/update-brew.yml`

- [ ] **Step 1: Create the formula template**

Create `packaging/brew/deepwash.rb.tmpl`:
```ruby
class Deepwash < Formula
  desc "A Command Line Interface to clean your machine (docker...)"
  homepage "https://github.com/Soluce-Technologies/deepwash"
  version "__VERSION__"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Soluce-Technologies/deepwash/releases/download/__VERSION__/deepwash-__VERSION__-aarch64-apple-darwin.tar.gz"
      sha256 "__SHA_ARM64__"
    end
    on_intel do
      url "https://github.com/Soluce-Technologies/deepwash/releases/download/__VERSION__/deepwash-__VERSION__-x86_64-apple-darwin.tar.gz"
      sha256 "__SHA_X86_64__"
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

- [ ] **Step 2: Create the update-brew workflow**

Create `.github/workflows/update-brew.yml`:
```yaml
name: Update Homebrew tap

on:
  workflow_call:
    inputs:
      version:
        required: true
        type: string
      ref:
        description: "Release tag holding the darwin tarballs"
        required: true
        type: string

permissions:
  contents: read

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/create-github-app-token@v1
        id: app-token
        with:
          app-id: ${{ vars.APP_ID }}
          private-key: ${{ secrets.APP_PRIVATE_KEY }}
          owner: SoluceTechnologies
          repositories: homebrew-deepwash

      - name: Checkout deepwash (for the template)
        uses: actions/checkout@v4
        with:
          ref: ${{ inputs.ref }}
          path: deepwash

      - name: Download darwin tarballs + hash
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          gh release download "${{ inputs.ref }}" \
            --repo "${{ github.repository }}" \
            --pattern 'deepwash-${{ inputs.version }}-*-apple-darwin.tar.gz' --dir dl
          SHA_ARM64=$(sha256sum dl/deepwash-${{ inputs.version }}-aarch64-apple-darwin.tar.gz | awk '{print $1}')
          SHA_X86=$(sha256sum dl/deepwash-${{ inputs.version }}-x86_64-apple-darwin.tar.gz | awk '{print $1}')
          echo "SHA_ARM64=$SHA_ARM64" >> "$GITHUB_ENV"
          echo "SHA_X86_64=$SHA_X86" >> "$GITHUB_ENV"

      - name: Render formula
        run: |
          sed -e "s/__VERSION__/${{ inputs.version }}/g" \
              -e "s/__SHA_ARM64__/${SHA_ARM64}/g" \
              -e "s/__SHA_X86_64__/${SHA_X86_64}/g" \
              deepwash/packaging/brew/deepwash.rb.tmpl > deepwash.rb
          cat deepwash.rb

      - name: Push to tap
        env:
          GH_TOKEN: ${{ steps.app-token.outputs.token }}
        run: |
          git clone "https://x-access-token:${{ steps.app-token.outputs.token }}@github.com/SoluceTechnologies/homebrew-deepwash.git" tap
          mkdir -p tap/Formula
          cp deepwash.rb tap/Formula/deepwash.rb
          cd tap
          git config user.name 'github-actions[bot]'
          git config user.email 'github-actions[bot]@users.noreply.github.com'
          git add Formula/deepwash.rb
          git commit -m "deepwash ${{ inputs.version }}" || echo "no change"
          git push origin HEAD
```

- [ ] **Step 3: Validate YAML + template render**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/update-brew.yml')); print('YAML OK')"
sed -e 's/__VERSION__/1.2.0/g' -e 's/__SHA_ARM64__/deadbeef/g' -e 's/__SHA_X86_64__/cafef00d/g' \
  packaging/brew/deepwash.rb.tmpl | grep -c 'sha256'
```
Expected: `YAML OK` then `2`.

- [ ] **Step 4: Commit**

```bash
git add packaging/brew/deepwash.rb.tmpl .github/workflows/update-brew.yml
git commit -m "ci: add homebrew tap update workflow + formula template"
```

---

## Task 8: Wire the new jobs into release.yml

**Files:**
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Add build-binaries after create-release**

In `.github/workflows/release.yml`, add this job (sibling of `publish-crate-registry`). Note `create-release` already outputs `version` and `draft_tag`; binaries upload to the draft tag:
```yaml
  build-binaries:
    needs: create-release
    if: ${{ needs.create-release.result == 'success' }}
    uses: ./.github/workflows/build-binaries.yml
    with:
      version: ${{ needs.create-release.outputs.version }}
      ref: ${{ needs.create-release.outputs.draft_tag }}
    secrets: inherit
```

> `ref` here is `draft_tag` (the `untagged-...` draft) because assets attach to the draft release before it is published. The crate publish keeps using `version` as its ref (the pushed git tag), unchanged.

- [ ] **Step 2: Add publish-apt + update-brew after build-binaries**

Add both jobs:
```yaml
  publish-apt:
    needs:
      - create-release
      - build-binaries
    uses: ./.github/workflows/publish-apt.yml
    with:
      version: ${{ needs.create-release.outputs.version }}
      ref: ${{ needs.create-release.outputs.draft_tag }}
    secrets: inherit

  update-brew:
    needs:
      - create-release
      - build-binaries
    uses: ./.github/workflows/update-brew.yml
    with:
      version: ${{ needs.create-release.outputs.version }}
      ref: ${{ needs.create-release.outputs.draft_tag }}
    secrets: inherit
```

- [ ] **Step 3: Make finalize-release wait for the new jobs**

Change the `finalize-release` `needs:` block from:
```yaml
    needs:
      - create-release
      - publish-crate-registry
```
to:
```yaml
    needs:
      - create-release
      - publish-crate-registry
      - build-binaries
      - publish-apt
      - update-brew
```
(Leave the rest of `finalize-release` unchanged — it un-drafts `draft_tag`, which now carries all assets.)

- [ ] **Step 4: Validate YAML**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('YAML OK')"`
Expected: `YAML OK`

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: wire apt/brew/binary jobs into release flow"
```

---

## Task 9: User-facing install docs

**Files:**
- Create: `docs/DISTRIBUTION.md`
- Modify: `README.md`

- [ ] **Step 1: Write docs/DISTRIBUTION.md**

Create `docs/DISTRIBUTION.md`:
```markdown
# Installing deepwash

## Homebrew (macOS)

    brew install soluce-technologies/deepwash/deepwash

## apt (Debian / Ubuntu)

    curl -fsSL https://soluce-technologies.github.io/deepwash/deepwash-archive-keyring.gpg \
      | sudo tee /usr/share/keyrings/deepwash.gpg >/dev/null
    echo "deb [signed-by=/usr/share/keyrings/deepwash.gpg] https://soluce-technologies.github.io/deepwash stable main" \
      | sudo tee /etc/apt/sources.list.d/deepwash.list
    sudo apt update && sudo apt install deepwash

## cargo

    cargo install deepwash

## Manual (.deb / tarball)

Download the `.deb` (amd64/arm64) or `.tar.gz` (macOS/Linux) for your platform
from the [latest release](https://github.com/Soluce-Technologies/deepwash/releases/latest).
```

- [ ] **Step 2: Add an Install section to README.md**

Add near the top of `README.md` (adapt to existing heading style):
```markdown
## Install

- **macOS (brew):** `brew install soluce-technologies/deepwash/deepwash`
- **Debian/Ubuntu (apt):** see [docs/DISTRIBUTION.md](docs/DISTRIBUTION.md)
- **cargo:** `cargo install deepwash`
```

- [ ] **Step 3: Verify links**

Run: `grep -q 'soluce-technologies.github.io/deepwash' docs/DISTRIBUTION.md && echo OK`
Expected: `OK`

- [ ] **Step 4: Commit**

```bash
git add -f docs/DISTRIBUTION.md
git add README.md
git commit -m "docs: add apt/brew install instructions"
```

> `docs/` is gitignored; `-f` is required (mirrors how spec/plan docs are committed).

---

## Task 10: First guarded release + end-to-end verification

This task runs only after Tasks 0–9 land on `main` and a real release fires. It cannot be pre-run; it is the acceptance gate.

- [ ] **Step 1: Trigger a release**

Merge a `feat:`/`fix:` PR to `main` (or an intentional version-bump PR). Watch the Actions run: `create-release → {publish-crate-registry, build-binaries} → {publish-apt, update-brew} → finalize-release` should all be green.

- [ ] **Step 2: Confirm release assets**

On the published release, verify presence of: 4 `.tar.gz` + 4 `.sha256` + 2 `.deb`.
Expected: all 10 assets attached, release no longer draft.

- [ ] **Step 3: Verify apt install in a clean container**

Run: `packaging/apt/verify-install.sh`
Expected: ends with `apt install smoke test PASSED` and prints the deepwash version. (Allow a minute for GitHub Pages to update after the push.)

- [ ] **Step 4: Verify brew (macOS, if available)**

```bash
brew untap soluce-technologies/deepwash 2>/dev/null || true
brew install soluce-technologies/deepwash/deepwash
deepwash --version
brew audit --strict --online deepwash || true
```
Expected: install succeeds, `--version` prints. Audit warnings are advisory.

- [ ] **Step 5: Record the outcome**

If all pass, distribution is live. If a channel fails, debug that channel's job logs; the other channels are independent and remain valid.

---

## Self-Review

**Spec coverage:**
- §1 build workflow → Tasks 3, 4. ✓
- §2 .deb → Task 2 (metadata) + Task 4 (build). ✓
- §3 apt repo on gh-pages → Tasks 5, 6; verification Task 10 Step 3. ✓
- §4 brew tap → Task 0 Step 4 (repo) + Task 7. ✓
- §5 wiring → Task 8. ✓
- Prerequisites (GPG key, tap repo, Pages, LICENSE) → Task 0 + Task 1. ✓
- Verification/testing → Task 5 (script) + Task 10. ✓

**Placeholder scan:** Formula uses `__VERSION__` / `__SHA_*__` sentinels — intentional, rendered by `sed` in Task 7 Step 2. `<KEYID>` in Task 0 is a runtime value the maintainer captures. No unresolved TODOs.

**Type/name consistency:** `draft_tag` / `version` outputs match `create-release` in the existing `release.yml`. Reusable-workflow inputs (`version`, `ref`) consistent across build-binaries/publish-apt/update-brew and their callers in Task 8. Tarball name pattern `deepwash-<version>-<target>.tar.gz` identical in build (Tasks 3/4), brew download+template (Task 7), and formula URLs. `.deb` arch names `amd64`/`arm64` consistent between Task 4 matrix and apt architectures (Task 6).

**Known risk flagged in-plan:** `cargo deb --no-build` reuse of the zigbuild artifact (Task 4 Step 1 note + Step 3 local rehearsal).
