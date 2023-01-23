# nenv

Node environment manager written in rust.

## Features

- Written in fast and safe rust
- Multiple active nodejs versions at the same time
- Configuration for project specific versions
- Version matching with semver expressions

## Installation

You can either
- Install the application with cargo by downloading the repo and running `cargo install --path .` inside
- Download the binary from the releases page

Now to initialize everything install any nodejs version with `nenv install <version>`.
Afterwards add the `bin` directory to your `PATH` variable.

On windows this should be `C:\Users\<yourusername>\AppData\Roaming\nenv\bin`.
On linux this will be `~/.local/share/nenv/bin`.

## Usage

### Install node versions

```sh
# install the latest available node version
nenv install latest

# install the latest lts version
nenv install lts

# install the latest 14.x.x version.
nenv install 14
```

### Change the system-wide default version

```sh
nenv default latest
```

### Refresh installed binaries and upstream versions

```sh
nenv refresh
```

### List nodejs versions

```sh
nenv list-versions
```


## Version detection

The node version nenv uses is controlled by

1. The `engines.node` config field in the `package.json` which is parsed as a semver requirement.
```json
{
  "name": "my project",
  "engines": {
    "node": "18"
  }
}
```

2. The `.node-version` file in the current or parent directories which contains the version string.
```
19.4.0  
```

3. The `NODE_VERSION` environment variable.
4. The default version set with `nenv default`.

## License

GPL-3.0