# nenv

Node environment manager written in rust.


## Installation

Figure out how to install it yourself (for now).
Add the nenv `bin` directory to your PATH variable.

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


## License

GPL-3.0