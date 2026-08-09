# brainfuck

A brainfuck interpreter in Rust.

## Usage

```
cargo build --release
./target/release/brainfuck examples/helloworld.bf
```

The program path may be a regular file, a pipe, or a character device, so
generated programs work without a temporary file:

```
./target/release/brainfuck <(printf '++++++++[>++++++++<-]>+.')   # prints A
```

## Language semantics

Brainfuck has no standard, so the choices this interpreter makes are:

| | |
|---|---|
| Cells | 8-bit, wrapping (`255 + 1 == 0`, `0 - 1 == 255`) |
| Tape | Grows on demand, no upper bound |
| `<` at cell 0 | Error, exit 1 — the tape has a left edge |
| `>` past the end | Always valid, allocates |
| `,` at EOF | Writes 0 to the current cell |
| Source encoding | Raw bytes; non-UTF-8 files are accepted |

Everything other than `> < + - . , [ ]` is a comment. Note that prose comments
are live code if they contain any of those eight characters — a `,` in an
English sentence is an input instruction.

Unbalanced brackets are rejected before execution starts, with the position of
the offending bracket.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Program ran to completion |
| 0 | stdout closed early (`\| head`) — treated as normal termination |
| 1 | Bad arguments, unreadable source, syntax error, or runtime error |

Runtime and syntax errors report `line:col` of the instruction that failed.

## Layout

- `examples/` — working programs: hello world, rot13, an ASCII christmas tree
- `tests/programs/` — inputs that exercise specific edge cases

The test programs each exist because they broke something:

| File | Exercises |
|---|---|
| `bin.bf` | Non-UTF-8 bytes in a comment |
| `hibyte.bf` | Output of a byte above 127, emitted raw rather than re-encoded |
| `no_instr.bf` | Source containing no instructions at all |
| `spew.bf` | Unbounded output — for closing the pipe early |
| `underflow.bf` | `<` at the left edge of the tape |

Some read stdin:

```
printf 'Hello' | ./target/release/brainfuck examples/rot13.bf
printf '4'     | ./target/release/brainfuck examples/xmastree.bf
```

`xmastree.bf` reads its height as decimal digits terminated by EOF, so the
input must not have a trailing newline — use `printf`, not `echo`.
