# Cleaner CLI

A command-line interface (CLI) tool written in Rust to clean up Docker instances by removing unused containers, images, volumes, and networks. Simplify your Docker environment management with a single command.

## Features

- Remove unused Docker containers, images, volumes, and networks.
- Safe cleanup with confirmation prompts (optional).
- Filter options to target specific resources.
- Dry-run mode to preview cleanup actions.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.65 or higher)
- [Docker](https://docs.docker.com/get-docker/) installed and running
- Cargo (included with Rust)

### Install from Crates.io

```bash
cargo install dockerclean
```

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Soluce-Technologies/cleaner-cli.git
   cd cleaner-cli
   ```

2. Build and install:
   ```bash
   cargo build --release
   cargo install --path .
   ```

## Usage

Run `cleaner-cli` to clean up your environment. Below are the available commands and options:

```bash
cleaner-cli --help
```

### Examples

- **Clean all unused resources** (containers, images, volumes, networks):
  ```bash
  cleaner-cli clean
  ```

- **Dry-run mode** (preview what would be deleted):
  ```bash
  cleaner-cli clean --dry-run
  ```

- **Clean only unused containers**:
  ```bash
  cleaner-cli clean --containers
  ```

- **Force cleanup without confirmation**:
  ```bash
  cleaner-cli clean --force
  ```

### Available Commands and Flags

- `clean`: Perform cleanup of Docker resources.
    - `--containers`: Clean only containers.
    - `--images`: Clean only images.
    - `--volumes`: Clean only volumes.
    - `--networks`: Clean only networks.
    - `--dry-run`: Simulate cleanup without deleting.
    - `--force`: Skip confirmation prompts.
    - `--all`: Clean all unused resources (default).

- `--help`: Display help information.
- `--version`: Display the version of `cleaner-cli`.

## Configuration

No configuration file is required. All options are passed via command-line flags. Ensure the Docker daemon is running and accessible by the user running `cleaner-cli`.

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-name`).
3. Commit your changes (`git commit -m 'Add feature'`).
4. Push to the branch (`git push origin feature-name`).
5. Open a pull request.

Please ensure your code follows the [Rust Style Guidelines](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md) and includes tests.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For issues or suggestions, please open an issue on the [GitHub repository](https://github.com/username/dockerclean).