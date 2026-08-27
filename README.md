# downmanager

A simple command-line download manager written in Rust.

## Tech Stack

* **rust**
* **tokio** 
* **reqwest**

## Usage

Build:

```bash
cargo build --release
```

To install globally (.cargo should be in your PATH):

```bash
cargo install --path . --force
```

Example:

```bash
downmanager <URL>
```
Downloads in your current working directory

## Options

```text
-h, --help       Show help
-v, --version    Show version
-r, --recover    Recover interrupted downloads (this is not implemented yet)
```

## Future features
* Multi-connection downloads 
* Download recovery/resuming 
