# slp

A memory-lean `sleep` with `--until`.

```sh
slp 1.5m 30        # pause 2 minutes (arguments are summed; suffixes s m h d; inf waits forever)
slp --until 14:39  # pause until 14:39 local time (today, or tomorrow if already past)
slp --until 7:05:30
```

`--until` takes `HH:MM` or `HH:MM:SS` in local time and cannot be combined with intervals. Exit status is 0 on success and 1 on a usage error.

- Clock changes and suspend: sleeps measure monotonic time, which stops while the machine is suspended. `--until` re-reads the wall clock at least every 30 seconds, so it fires within that bound of the target after a resume.
- Daylight saving: the target is resolved by the C library (`mktime`). A time skipped by a spring-forward resolves to the shifted hour; an ambiguous fall-back time resolves to the C library's choice.
- Memory: no runtime dependency beyond `libc`, no threads, no allocation after argument parsing. mise build and install compile for the native CPU (see scripts/cargo-native.sh), because an emulated toolchain's default target roughly doubles resident memory.

## Install

```sh
mise install        # toolchain, then builds and installs slp to ~/.local/bin
```

Set `SLP_INSTALL_ROOT` to install under another prefix. Make sure `~/.local/bin` is on `PATH`.

## Development

`mise run ci` is the full gate (`lint`, `typecheck`, `test`, `scan`, `build`). `mise run doctor` reports missing prerequisites. `LAYOUT.md` names each source file's single purpose.
