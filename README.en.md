# pwde2e

A BORUIX acceptance test for the full account look-up chain by name and by id on the real system.

[简体中文](README.md)

## What it tests

The C library offers look-up of users by name and by id. Running as an independent process, this
program verifies the whole chain from the account table to the query result:

- With no account table on disk, look-up reports "no such user" and fabricates nothing
- After writing a table of three accounts, a by-name query returns the correct uid, gid, name and home directory
- A by-id query holds too, and inverts the by-name query
- The home directory matches what the account table registered

Each check prints `OK` as it passes; exit code 0 means all passed.

## Exit codes

- `0` — all passed
- `1` to `16` — the number of the failed check, matching the `FAIL(n)` position in the output

## Building

```bash
cargo build --release
```

## Repository layout

```
pwde2e/
├── Cargo.toml    # package manifest
├── build.rs      # injects the linker script
├── linker.ld     # user-space segment layout
└── src/
    └── main.rs   # account table writing and query checks
```

## Related projects

- [`libc`](https://github.com/BRX-Boruix/libc) — the look-up interfaces under test
- [`userd`](https://github.com/BRX-Boruix/userd) — the account daemon
- [`selftest`](https://github.com/BRX-Boruix/selftest) — spawns this program as a real process

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
