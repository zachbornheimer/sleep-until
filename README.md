# slp

`slp` is a small `sleep` that can also wait until a time of day.

```sh
slp 90             # pause 90 seconds
slp --until 14:39  # pause until 14:39 local time
```

## Why it exists

Long-running shell scripts and background jobs often hold a `sleep` open for hours. `slp` keeps that process small and adds `--until`, so you do not have to compute the number of seconds yourself.

Measured on Apple Silicon with `hyperfine --warmup 3 'sleep 0.3' 'slp 0.3'`:

| Command     | Resident memory | Wall time    |
| ----------- | --------------- | ------------ |
| `sleep 0.3` | 1.2 MiB         | about 320 ms |
| `slp 0.3`   | 1.5 MiB         | about 312 ms |

A bare Rust binary has a floor of about 1.4 MiB on this platform. The 0.3 MiB gap to `/bin/sleep` comes from the toolchain, not from `slp` code.

## Install

```sh
mise install   # installs the Rust toolchain, then builds and installs slp
```

`slp` goes to `~/.local/bin`. Set `SLP_INSTALL_ROOT` to install under `$SLP_INSTALL_ROOT/bin` instead. Make sure that directory is on your `PATH`. To reinstall after a change, run `mise run install`.

## Usage

| Command                          | What it does                                           |
| -------------------------------- | ------------------------------------------------------ |
| `slp 90`                         | Pause 90 seconds.                                      |
| `slp 1.5m 30`                    | Pause 2 minutes (90 s + 30 s). Arguments are added.    |
| `slp 2h`                         | Pause 2 hours.                                         |
| `slp 1d`                         | Pause 1 day.                                           |
| `slp 1h30m`                      | Pause 1 hour 30 minutes. Also `2m30s`, `1.5h`.         |
| `slp inf`                        | Pause forever.                                         |
| `slp --until 14:39`              | Pause until 14:39 local time.                          |
| `slp --until 7:05:30`            | Pause until 07:05:30 local time.                       |
| `slp --until=14:39`              | Same as `--until 14:39`.                               |
| `slp --until '2026-10-10 07:00'` | Pause until that date and time.                        |
| `slp -n 1h30m`                   | Print the wake time and exit without pausing.          |
| `slp -- 0.1`                     | `--` ends options; everything after it is an interval. |
| `slp --help`                     | Print usage and exit.                                  |
| `slp --version`                  | Print the version and exit.                            |

Rules:

- Suffixes are `s` (seconds, the default), `m`, `h`, and `d`.
- Numbers can be decimal (`0.5`, `.5`, `1e1`). Negative numbers are rejected.
- Several arguments are added together.
- Segments can be joined: `1h30m`, `2m30s`. Every segment needs a suffix, so `1h30` is rejected. `inf` must be the whole argument.
- `inf` and `infinity` mean forever. So does a number too large to represent.
- `--until` takes `HH:MM` or `HH:MM:SS` in 24-hour local time, with one or two digits per field. If that time has already passed today, `slp` waits for tomorrow.
- `--until 'YYYY-MM-DD HH:MM[:SS]'` waits for that local moment. A `T` can replace the space before the time. A target in the past is an error.
- `-n` / `--dry-run` prints `wake at YYYY-MM-DD HH:MM:SS +ZZZZ` (or `wake at never` for `inf`) and exits 0 without sleeping. `--help` and `--version` win.
- `--until` cannot be combined with intervals and cannot be given twice.

## Exit status and errors

`slp` exits 0 when the wait finishes and 1 on any error. Errors go to stderr. Usage errors also print a pointer to `--help`.

```text
$ slp abc
slp: invalid time interval 'abc'
Try 'slp --help' for more information.

$ slp --until 25:00
slp: invalid time of day '25:00' (expected HH:MM or HH:MM:SS)
Try 'slp --help' for more information.

$ slp --until 14:39 5
slp: '--until' cannot be combined with a time interval
Try 'slp --help' for more information.

$ slp 1h30
slp: invalid time interval '1h30'
Try 'slp --help' for more information.

$ slp --until '2020-01-01 00:00'
slp: --until 2020-01-01 00:00 has already passed
```

Dry run, with `TZ=UTC`:

```text
$ slp -n --until '2026-10-10 07:00'
wake at 2026-10-10 07:00:00 +0000

$ slp -n inf
wake at never
```

## Behavior notes

- **Suspend.** Sleeping uses a clock that stops while the machine is suspended. For `--until`, `slp` re-reads the wall clock at least every 30 seconds (`MAX_SLICE` in `src/wait.rs`), so after a resume it fires within that bound of the target.
- **Daylight saving.** The target time is resolved by the C library (`mktime`). The result for a time skipped or repeated by a clock change is whatever `mktime` returns. This holds for dated `--until` too: a time that does not exist moves forward. With `TZ=America/New_York`, `slp -n --until "2027-03-14 02:30"` prints `wake at 2027-03-14 03:30:00 -0400`.
- **Memory.** The only dependency is `libc`. `slp` starts no threads of its own, and the waiting code in `src/wait.rs` and `src/system.rs` does not allocate. On Apple Silicon, use a native Rust toolchain (`brew install rustup`). An emulated x86_64 toolchain roughly doubles resident memory, and `mise run doctor` flags it.
- **Compared with `sleep`.** GNU `sleep` and the macOS `/bin/sleep` (checked on macOS 26: `sleep 1m` waits 60 seconds) already accept the `s m h d` suffixes. `slp` adds `--until` and `inf`; the macOS `sleep` rejects `inf` with a usage message.

## Development

Run tasks with `mise run <task>`.

| Task      | What it does                                                           | Defined in |
| --------- | ---------------------------------------------------------------------- | ---------- |
| `fmt`     | Format source (`dprint`, then `cargo fmt`).                            | this repo  |
| `lint`    | Profile lint, `cargo fmt --check`, and clippy with warnings as errors. | this repo  |
| `test`    | Unit, workflow, adapter, architecture, and binary tests.               | this repo  |
| `build`   | Build the release binary.                                              | this repo  |
| `doctor`  | Profile check plus a native-toolchain and PATH check.                  | this repo  |
| `install` | Install `slp` into `$SLP_INSTALL_ROOT/bin`.                            | this repo  |
| `setup`   | Install tools and dependencies.                                        | profile    |
| `scan`    | Secret and dependency scan.                                            | profile    |
| `ci`      | The full gate: `lint`, `test`, `scan`.                                 | profile    |

`LAYOUT.md` names each source file's single purpose. Git hooks are global (hk), so the repo carries none.
