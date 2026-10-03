# nfc-in-rust

An NFC-unlocked encryption box for USB sticks, written in bare-metal `no_std` Rust for the NXP
FRDM-MCXN236 board (Cortex-M33, target `thumbv8m.main-none-eabihf`).

You plug a USB stick into the board and tap an NFC tag. If the stick holds normal data, the
board encrypts it. If the stick was already encrypted by the board, it decrypts it. Without the
right tag, the data on the stick stays unreadable.

There is no HAL and no `cortex-m-rt` in this project. The linker script, vector table and
reset handler are written by hand, and every peripheral (GPIO, timers, I2C, USB, flash) is
driven by writing its registers directly, following the reference manual.

This is still early work. Right now the board boots into my own runtime and blinks an LED
using GPIO and SysTick configured from the registers. The NFC, USB and crypto parts haven't
been started yet.

## How it should work

The hardware:

- the FRDM-MCXN236 board
- a PN532 NFC module, connected over I2C
- a USB port on the board working as a USB host, where the stick goes in
- an RGB LED that shows what the board is doing

The flow:

| What happens                             | LED    | What the board does                          |
|------------------------------------------|--------|----------------------------------------------|
| nothing plugged in                       | off    | waits for a stick                            |
| a USB stick is plugged in                | yellow | reads the stick and checks if it's encrypted |
| an NFC tag is brought close to the board | red    | encrypts or decrypts the whole stick         |

The board decides what to do by itself:

- If the stick isn't marked as encrypted, it encrypts it with the key linked to that NFC tag
  and marks it as encrypted.
- If the stick is already marked as encrypted, it picks the matching key and decrypts it.

The keys are stored in the board's internal flash, in a small list. The NFC tag and the flag
saved for each stick tell the board which key from the list to use.

## Why

This is my dissertation project. I want to understand every step between power-on and an
encrypted USB stick, so I'm avoiding crates that hide what the hardware is doing:

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
| `src/main.rs`        | panic handler and `main` (the LED blink for now)             |
| `src/startup.rs`     | linker symbols, vector table, `Reset`, `DefaultHandler`      |
| `src/reg.rs`         | `read_reg`, `write_reg`, `set_bits`                          |
| `src/scb.rs`         | ARM System Control Block registers (`CPACR`, `VTOR`)         |
| `src/systick.rs`     | ARM SysTick: `init` and `delay_ms`                            |
| `src/syscon.rs`      | NXP SYSCON registers (clock gating)                          |
| `src/gpio.rs`        | NXP GPIO4 registers (`PDOR`, `PTOR`, `PDDR`)                 |

## Roadmap

The basics, done:

1. ~~Startup code~~ (`.data`, `.bss`, FPU, `VTOR`, `main`)
2. ~~Vector table~~ for the core exceptions
3. ~~GPIO and blinky~~ (red LED on P4_18)
4. ~~SysTick delay~~

Board foundations:

5. Add `mcx-pac`.
6. Clocks: understand FRO/PLL/SCG on the MCX N and set known frequencies for the core, I2C
   and USB.
7. Interrupts: device IRQ slots in the vector table and an interrupt-driven SysTick tick.
8. The RGB LED with all three colours (yellow is red and green together).

NFC:

9. I2C master from registers: pin muxing, baud rate, START/STOP, TX/RX FIFOs, NACK and
   arbitration loss.
10. PN532 transport: wakeup over I2C, polling the status byte, building and parsing frames
    (preamble, `00 FF` start code, `LEN`/`LCS`, `TFI` `0xD4`/`0xD5`, payload, `DCS`,
    postamble), ACK/NACK and error frames.
11. PN532 commands: `GetFirmwareVersion`, `SAMConfiguration`, `InListPassiveTarget`
    (ISO/IEC 14443 Type A), and reading a tag UID.

USB stick:

12. USB host controller: detect when a device is plugged in or removed, reset the port,
    control transfers.
13. Enumeration: read the device descriptor, set an address, pick the configuration.
14. Mass Storage Class (Bulk-Only Transport) with the SCSI commands needed to read and write
    blocks (`INQUIRY`, `READ CAPACITY`, `READ(10)`, `WRITE(10)`).

Keys and encryption:

15. Internal flash driver, to keep the key list and the per-stick flags across power cycles.
16. AES, checked against the official NIST test vectors. I still have to see if the MCXN236
    has a hardware AES block I can use, or if it has to be done in software.
17. Encrypting and decrypting a whole stick block by block, and marking it as encrypted.

Putting it together:

18. The full flow: stick in, yellow LED, tag tapped, red LED, encrypt or decrypt, done.

## Open questions

Things I still need to decide before the encryption part:

- Where the "encrypted" flag lives. Keeping it only on the board means another board can't
  tell that the stick is encrypted. Writing a small header on the stick itself would fix
  that, and the board could keep the key list.
- How a stick is recognised. The USB serial number is the obvious choice, but not every
  stick has a unique one.
- What happens if the stick is pulled out or the power drops halfway through. The stick would
  end up half encrypted, so the board needs to remember how far it got.
- How the keys in flash are protected, so someone with a debugger can't just read them out.
- Block level or file level encryption. Encrypting raw blocks is simpler and doesn't need a
  FAT driver, but the stick then can't be read by a computer until it's decrypted again.

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
board. That's fine for a dissertation, but I wouldn't build a real product this way. Writing
my own AES is also something you shouldn't do in production. Here it's for learning, and it
gets checked against the standard test vectors.
