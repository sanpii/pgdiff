# PgDiff

[![Crates.io](https://img.shields.io/crates/v/pgdiff)](https://crates.io/crates/pgdiff)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/pgdiff)
[![Github actions](https://github.com/sanpii/pgdiff/workflows/.github/workflows/ci.yml/badge.svg)](https://github.com/sanpii/pgdiff/actions?query=workflow%3A.github%2Fworkflows%2Fci.yml)
[![pipeline status](https://gitlab.com/sanpi/pgdiff/badges/main/pipeline.svg)](https://gitlab.com/sanpi/pgdiff/-/commits/main)

A tool to generate diff beetween two databases.

```
cargo run -- --old postgresql://localhost/old --new postgresql://localhost/new
```
