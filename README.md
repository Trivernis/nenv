# nenv

Node environment manager written in rust.

## Features

- Written in fast and safe rust
- Multiple active nodejs versions at the same time
- Configuration for project specific versions with the `engines.node` field in the package.json
- Version matching with semver expressions

## Installation

You can either
- Install the application with cargo by downloading the repo and running `cargo install --path .` inside
- Download the binary from the releases page

Now to initialize everything install any nodejs version with `nenv install <version>`.
Afterwards add the `bin` directory to your `PATH` variable.

On windows this should be `C:\Users\<yourusername>\AppData\Roaming\nenv\bin`.
On linux this will should be `~/.local/share/nenv/bin`.

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


## License

GPL-3.0