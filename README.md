# zirvstdlib

The Zirvium standard library: a `no_std`, 64-bit Rust library shared by the
Zirvium kernel, DisplayJet, MOSIX and future userspace.

It deliberately contains no C, no bindings to a host libc and no 32-bit code.
The crate builds for `x86_64-unknown-none` and `x86_64-unknown-uefi`, and its
unit tests run on the host with `cargo test --target x86_64-unknown-linux-gnu`.

## Modules

| Module        | Purpose                                              |
|---------------|------------------------------------------------------|
| `abi`         | Stable `repr(C)` syscall/input/GPU ABI               |
| `bits`        | Bitmaps and bit manipulation                         |
| `collections` | Small helpers over `alloc`                           |
| `color`       | ARGB packing, blending and HSL conversion            |
| `error`       | Error codes and results                              |
| `fmt`         | Fixed buffers and formatting helpers                 |
| `geometry`    | Points, sizes, rectangles and clipping               |
| `hash`        | FNV-1a and simple hashing                            |
| `io`          | Byte readers and writers                             |
| `log`         | Log records and the logger trait                     |
| `math`        | Floating-point helpers missing from the target core  |
| `mem`         | Physical/virtual address wrappers and alignment      |
| `path`        | Path normalisation and joining                       |
| `prelude`     | Common re-exports                                    |
| `rand`        | A small deterministic PRNG                           |
| `ring`        | Fixed-capacity ring buffer                           |
| `sync`        | Spin lock, once cell and atomics helpers             |
| `sys`         | Syscall wrappers around the Zirvium ABI              |
| `time`        | Monotonic clock and calendar conversion              |

## Usage

```toml
[dependencies]
zirvstdlib = { git = "https://github.com/gauthamnair2005/zirvstdlib" }
```

## License

GPL-3.0-or-later.
