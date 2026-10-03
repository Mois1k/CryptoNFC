# nfc-in-rust

A bare-metal driver for the PN532 NFC chip over I2C, written in `no_std` Rust for the NXP
FRDM-MCXN236 board (Cortex-M33, target `thumbv8m.main-none-eabihf`).

There is no HAL and no `cortex-m-rt` in this project. The linker script, vector table and
reset handler are written by hand, and peripherals are driven by writing their registers
directly, following the reference manual.

This is still early work. The board boots into my own runtime and blinks the red LED once per
second, using GPIO and SysTick configured straight from the registers. The I2C and PN532 parts
haven't been started yet.

## Why

This is my dissertation project. I want to understand every step between power-on and reading
an NFC tag, so I'm avoiding crates that hide what the hardware is doing:

- No HAL. I write the peripheral registers myself, with the reference manual open.
- No `cortex-m-rt`. Writing the startup code myself (vector table, `.data`/`.bss` init, the
  jump into `main`) is part of what I want to learn.
- The only dependency I plan to allow is `mcx-pac`, a register map generated from NXP's SVD
  file. It only gives names to registers and fields, without adding any behaviour, and it can
  be checked against the datasheet. It isn't added yet, so for now registers are accessed
  through raw addresses.

## What works so far

- Toolchain set up for `thumbv8m.main-none-eabihf` (edition 2024, stable `rustc 1.93`).
- `memory.x` with 1 MiB of flash at `0x0000_0000` and 224 KiB of RAM at `0x2000_0000`.
- A linker script (`link.x`) that puts the vector table at the start of flash, places
  `.text`, `.rodata`, `.data` and `.bss`, and exports the symbols the startup code needs.
- The full Cortex-M33 vector table (16 entries). All core exceptions point to a
  `DefaultHandler` that just loops, so a fault stops the program in a known place.
- A reset handler that enables the FPU (`CPACR`), sets `VTOR`, copies `.data` from flash,
  zeroes `.bss` and then calls `main`.
- Small register helpers in `reg.rs` (`read_reg`, `write_reg`, `set_bits`). In release builds
  they get inlined, and the disassembly is the same as the raw pointer code they replaced.
- One module per peripheral, with the ARM core registers kept apart from the NXP ones.
- The PORT4 and GPIO4 clocks enabled through `SYSCON.AHBCLKCTRL0`.
- The red LED on P4_18 (active low) as an output. `PDOR` is written before `PDDR` so the LED
  doesn't flash on for a moment at startup, and `PTOR` toggles it.
- SysTick with a 1 ms reload value (`47_999` at 48 MHz) and a blocking `delay_ms` that polls
  `COUNTFLAG`.
- Everything above was tested on the board with `probe-rs`. I also checked the core clock
  two ways (a calibrated `nop` loop and SysTick) and both came out at 48 MHz.

## Still to do

- Add `mcx-pac` and replace the raw addresses.
- Configure the clocks. Right now the chip runs on whatever the boot ROM leaves (48 MHz).
- Device interrupts. SysTick is polled for now.
- GPIO inputs (the SW2/SW3 buttons) and proper pin functions instead of plain addresses.
- I2C (LPI2C).
- The PN532 frame format and commands.
- A `runner` in `.cargo/config.toml` so `cargo run` flashes the board.
- Tests and CI.
- In the linker script: handle `.ARM.exidx`/`.ARM.attributes`, add a stack overflow guard,
  and maybe use RAMX (96 KiB at `0x0400_0000`).
- Remove the two test statics (`DATA_CONTOR_TEST`, `BSS_ZERO_TEST`) once there are real ones.

## Building

You need the target and the LLVM tools (for `rust-objdump`):

```sh
rustup target add thumbv8m.main-none-eabihf
rustup component add llvm-tools
cargo install cargo-binutils   # optional, gives cargo objdump / cargo size
```

The target and linker flags are set in `.cargo/config.toml`, so a plain build works:

```sh
cargo build --release
```

Note that `cargo` doesn't notice changes to `memory.x` or `link.x`. After editing them, run
`cargo clean` and check the new stack top with `rust-objdump -t ... | grep _estack`.

To look at the output:

```sh
rust-objdump -h                  target/thumbv8m.main-none-eabihf/release/RustNFC   # sections
rust-objdump -s -j .vector_table target/thumbv8m.main-none-eabihf/release/RustNFC   # vector table
rust-objdump -d                  target/thumbv8m.main-none-eabihf/release/RustNFC   # disassembly
```

The vector table should be at `0x0` and `0x40` bytes long. The first word is the stack top
(`0x2003_8000`), the second is the address of `Reset` with the Thumb bit set (an odd number).

## Flashing

I use [probe-rs](https://probe.rs) with the MCU-Link debugger that's on the board.

A few things that cost me time:

- Jumper JP7 (SWD_DIS) has to be open. If it's closed, MCU-Link doesn't expose its CMSIS-DAP
  interface and probe-rs fails with "Could not determine a suitable packet size".
- On WSL the USB device has to be forwarded from Windows with `usbipd-win`
  (`usbipd attach --wsl --busid <BUSID>` after every replug), and the probe-rs udev rules
  need to be installed.
- `probe-rs info` can't detect this chip, so use the commands that take `--chip`.

```sh
probe-rs download --chip MCXN236VDF target/thumbv8m.main-none-eabihf/release/RustNFC
probe-rs reset    --chip MCXN236VDF

# read the .data/.bss test statics (should be 5 and 0)
probe-rs read  --chip MCXN236VDF b32 0x20000000 2

# toggle the red LED by hand
probe-rs write --chip MCXN236VDF b32 0x4009E04C 0x00040000
```

## Files

| Path                 | Contents                                                     |
|----------------------|--------------------------------------------------------------|
| `.cargo/config.toml` | default target and linker flags                              |
| `Cargo.toml`         | crate metadata, no dependencies yet                          |
| `memory.x`           | flash and RAM regions                                        |
| `link.x`             | linker script                                                |
| `src/main.rs`        | panic handler and `main` (the LED blink)                     |
| `src/startup.rs`     | linker symbols, vector table, `Reset`, `DefaultHandler`      |
| `src/reg.rs`         | `read_reg`, `write_reg`, `set_bits`                          |
| `src/scb.rs`         | ARM System Control Block registers (`CPACR`, `VTOR`)         |
| `src/systick.rs`     | ARM SysTick: `init` and `delay_ms`                            |
| `src/syscon.rs`      | NXP SYSCON registers (clock gating)                          |
| `src/gpio.rs`        | NXP GPIO4 registers (`PDOR`, `PTOR`, `PDDR`)                 |

## Notes on the design

### Why not `cortex-m-rt`

`cortex-m-rt` already provides the vector table, the reset handler, `.data`/`.bss` init, the
`#[entry]` and `#[exception]` macros and its own linker script. That's exactly the part I
want to write myself. What happens between reset and the first line of `main`, and what has
to be true about memory before Rust code is safe to run, is one of the most interesting parts
of embedded work, and with the crate it would all stay hidden.

### Why not a HAL

A HAL gives you a portable API in exchange for hiding the registers. Since the registers are
the subject of my dissertation, that trade doesn't make sense here.

### Why the PAC is fine

`mcx-pac` is generated from NXP's SVD. It gives each register and field a typed name and
nothing else: no init sequences, no hidden state. It removes a whole class of typos (wrong
offset, wrong bit) without hiding how anything works.

### Downsides

There's more boilerplate, and it's easy to get small things wrong: section alignment, `Sync`
impls, `#[used]`, volatile accesses, memory barriers. The code also only works on this one
board. That's fine for a dissertation, but I wouldn't do it this way in a real driver crate.

## Roadmap

1. ~~Finish the startup code~~ (`.data`, `.bss`, FPU, `VTOR`, `main`)
2. ~~Full vector table~~ for the core exceptions. Device IRQs come later.
3. Add `mcx-pac`.
4. Clocks: understand FRO/PLL/SCG on the MCX N, set a known core frequency and a clock for I2C.
5. ~~GPIO and blinky~~ (red LED on P4_18)
6. ~~SysTick delay~~. An interrupt-driven tick comes later.
7. I2C master from registers: pin muxing, baud rate, START/STOP, TX/RX FIFOs, NACK and
   arbitration loss.
8. PN532 transport: wakeup over I2C, polling the status byte, building and parsing frames
   (preamble, `00 FF` start code, `LEN`/`LCS`, `TFI` `0xD4`/`0xD5`, payload, `DCS`,
   postamble), ACK/NACK and error frames.
9. PN532 commands: `GetFirmwareVersion`, `SAMConfiguration`, `InListPassiveTarget`
   (ISO/IEC 14443 Type A), and reading a card UID from start to finish.
10. If there's time: use the PN532 IRQ line instead of polling, MIFARE Classic
    authentication, NDEF parsing.
