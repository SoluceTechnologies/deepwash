# Deep Wash 🧹

A command-line interface (CLI) tool written in Rust to clean up Docker instances by removing unused containers, images, volumes, and networks. Simplify your Docker environment management with a single command.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.65 or higher)
- [Docker](https://docs.docker.com/get-docker/) installed and running
- Cargo (included with Rust)

### Install from Crates.io

```bash
cargo install deepwash
```

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/SoluceTechnologies/deepwash.git
   cd deepwash
   ```

2. Build and install:
   ```bash
   cargo build --release
   cargo install --path .
   ```

## Usage

Run `deepwash` to clean up your environment. Below are the available commands and options:

```bash
deepwash --help
```
