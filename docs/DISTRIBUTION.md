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
