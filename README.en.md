# pwde2e

An **end-to-end acceptance program** for BORUIX, verifying that the POSIX account lookup interfaces work inside a real user-space process.

[简体中文](README.md)

## What it tests

The program runs as a **standalone process**, calling the account lookup functions through standard C interfaces. There are 16 checks:

| Check | Contents |
| --- | --- |
| Lookup with no account table | Returns a null pointer, **fabricating no account** |
| `getpwnam` | Looks up `alice` by name; returns the real UID / GID / name length / home directory |
| `getpwuid` | Looks up `bob` by UID; round-trips both ways |
| UID fallback | `carol`, declared without a GID, falls back to its UID |
| Misses | An unknown name or UID returns a null pointer with `errno` set to `ENOENT` |
| `getpwent` | Enumeration yields exactly 3 records |
| Stale cache | After the table is removed, `alice` must not still resolve |

Account data comes from `/config/users.json`. The fixtures are 3 accounts: `alice`(1000:1000), `bob`(1001:1001), `carol`(1002).

## Why a separate program

A separate account interface check runs inside the `shell` process. This program is a **standalone process**, verifying the path where **any user program calls these functions through the C library** — covering real process startup, linking, and the interface crossing.

## Usage

Started by the quick group of the system self-test via a real `exec` path; deployed as `/programs/pwde2e.elf`.

## Exit codes

| Exit code | Meaning |
| --- | --- |
| `0` | All 16 checks passed |
| `1`–`16` | Number of the first failing check |

## Building

```bash
cargo build --release
```

## Layout

```
pwde2e/
├── Cargo.toml    # package definition
├── build.rs      # injects the linker script
├── linker.ld     # user-space section layout
└── src/
    └── main.rs   # the 16 acceptance checks
```

## Related projects

- [`libc`](https://github.com/BRX-Boruix/libc) — provides the C interface implementations for account lookup
- [`selftest`](https://github.com/BRX-Boruix/selftest) — starts this program and judges its exit code
- [`userd`](https://github.com/BRX-Boruix/userd) — the account daemon maintaining home directories and identity projections

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
