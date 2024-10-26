# Qt Resource Extractor (qrex)

[![Current Crates.io Version](https://img.shields.io/crates/v/qrex?logo=rust&style=flat&color=blue)](https://crates.io/crates/qrex)

## Installation

Install from [crates.io](https://crates.io/crates/qrex):

```sh
cargo install qrex
```

or directly from [GitHub](https://github.com/mb1986/qrex):

```sh
cargo install --git https://github.com/mb1986/qrex
```

## Configuration

```yaml
binary_path: binary                # path to a qt application binary
output_path: output/directory      # path to a directory for extracted resources

base_address: 0x10000              # (optional) value which will be subtracted from any of resource addresses

resources:                         # resource entries container
    - version: 3                   # Qt resource version (only version 3 is currently supported)
      addresses: { tree: 0xb34bd8, names: 0xb34a28, data: 0xb369d8 } # triple of addresses pointing to resource structures
      calls: [0x61b30, 0x64228]    # (optional) addresses from where a resource has been registered

    - version: 3
      addresses: { tree: 0xa962e0, names: 0xa96170, data: 0xa95f98 }
      calls: [0x632e4, 0x64330]

    - version: 3
      addresses: { tree: 0x7811a8, names: 0x7810f8, data: 0x780c48 }
      calls: [0x63a88]
```

```toml
binary_path = "binary"             # path to a qt application binary
output_path = "output/directory"   # path to a directory for extracted resources

base_address = 0x10000             # (optional) value which will be subtracted from any of resource addresses

[[resource]]                       # resource entry
version = 3                        # Qt resource version (only version 3 is currently supported)
addresses = { tree = 0xb34bd8, names = 0xb34a28, data = 0xb369d8 } # triple of addresses pointing to resource structures
calls = [0x61b30, 0x64228]         # (optional) addresses from where a resource has been registered

[[resource]]
version = 3
addresses = { tree = 0xa962e0, names = 0xa96170, data = 0xa95f98 }
calls = [0x632e4, 0x64330]

[[resource]]
version = 3
addresses = { tree = 0x7811a8, names = 0x7810f8, data = 0x780c48 }
calls = [0x63a88]
```

## Usage

```text
Usage: qrex [OPTIONS] <CONFIG>

Arguments:
  <CONFIG>  Path to the YAML or TOML configuration file

Options:
  -e, --extract    Extract resources
  -s, --skip-dirs  Skip creating separate directories for each resource
  -m, --metadata   Save resources' metadata
  -d, --debug      Enable debug messages
  -h, --help       Print help
  -V, --version    Print version
```

## Todo

- [ ] Documentation
- [x] Format metadata as TOML
- [x] Error handling (at least `anyhow`)
- [ ] Tests
