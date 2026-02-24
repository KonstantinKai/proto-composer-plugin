# proto-composer-plugin

A community [WASM plugin](https://moonrepo.dev/docs/proto/wasm-plugin) for [proto](https://github.com/moonrepo/proto) that manages [Composer](https://getcomposer.org/) versions.

## Prerequisites

Composer requires PHP. Install PHP via [proto-php-plugin](https://github.com/KonstantinKai/proto-php-plugin) or have it available on your system PATH.

## Installation

```bash
proto plugin add composer "github://KonstantinKai/proto-composer-plugin"
proto install composer
```

## Usage

```bash
# Install Composer
proto install composer 2.8

# Use Composer
proto run composer -- --version

# List available versions
proto versions composer

# Pin a version in the current directory
proto pin composer 2.8
```

## Configuration

Configure in `.prototools` under `[tools.composer]`:

```toml
[tools.composer]
# Custom COMPOSER_HOME directory (optional)
composer-home = "/path/to/composer/home"
```

## Supported Platforms

| Platform       | Architecture      | Install method       |
|----------------|-------------------|----------------------|
| Linux          | x64, arm64        | Download PHAR binary |
| macOS          | x64, arm64        | Download PHAR binary |
| Windows        | x64               | Download PHAR + .bat wrapper |

## How It Works

Unlike most proto plugins that download prebuilt native binaries, Composer is distributed as a PHP archive (PHAR). The plugin uses `native_install` to:

1. Download `composer.phar` from `getcomposer.org/download/<version>/composer.phar`
2. On Unix: save as `composer` and `chmod +x`
3. On Windows: save as `composer.phar` and create a `composer.bat` wrapper

The plugin declares `requires: ["php"]` so proto ensures PHP is available before installing Composer.

## Version Aliases

- `latest` — resolves to the newest stable Composer 2.x release
- `stable` / `lts` — same as `latest`
- Partial versions like `2.8` resolve to the latest patch (e.g. `2.8.6`)

## Related

- [proto-php-plugin](https://github.com/KonstantinKai/proto-php-plugin) — PHP version management for proto

## License

MIT
