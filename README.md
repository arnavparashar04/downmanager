# downmanager

A simple command-line download manager with multi connection download support.

## Tech Stack

* **rust**
* **tokio** 
* **reqwest**

## NOTE
This download manager only supports multi connection downloads on Unix like operating systems. Windows is not supported.
## Usage

Build:

```bash
cargo build --release
```

To install globally (.cargo should be in your PATH):

```bash
cargo install --path . --force
```

### Example:

Downloads in your current working directory
```bash
downmanager <URL>
```
To force number of connections

```bash
downmanager <URL> --force <number of connections>
```


## Options

```text
-h, --help       Show help
-v, --version    Show version
-r, --recover    Recover interrupted downloads (this is not implemented yet)
```

## Future features
* Download recovery/resuming 
* Expand into a torrent client
