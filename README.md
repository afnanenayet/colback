# colback

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/afnanenayet/colback/ci.yaml?style=flat-square)
![docs.rs](https://img.shields.io/docsrs/colback?style=flat-square)
![Crates.io Version](https://img.shields.io/crates/v/colback?style=flat-square)
![Crates.io License](https://img.shields.io/crates/l/colback?style=flat-square)

Column backed lists of structs.

## Synopsis

This is similar to the SOA crate, except instead of using raw Rust arrays, this
uses columns from dataframes to back the data that the arrays pull from.

## Motivation

When writing code that deals with dataframes, sometimes you want the ability
to do complex operations across multiple columns but write code as you normally
would without paying for the cost of copying over all of the data into a potentially
less cache-friendly memory layout.

This also handles a lot of the boilerplate around extracting values from dataframes
using the Polars API.

## Development

This is a standard Rust project that uses cargo.

### Run tests

We recommend using `cargo-nextest` as your test runner.

```bash
cargo nextest run
```
