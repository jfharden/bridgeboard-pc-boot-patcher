# bridgeboard-pc-boot-patcher

[![Post-merge Test and Release](https://github.com/jfharden/bridgeboard-pc-boot-patcher/actions/workflows/post-merge.yml/badge.svg?branch=main)](https://github.com/jfharden/bridgeboard-pc-boot-patcher/actions/workflows/post-merge.yml)

Release: [latest](https://github.com/jfharden/bridgeboard-pc-boot-patcher/releases/latest)

Patch the Amiga Bridgeboard option rom in the pc.boot file in order to not use the autoboot functionality of the
bridgeboard. This should allow an XTIDE set in the 0xC000 memory range to be used, specifically I found 0xCC00 to work.

It can also validate option rom checksums, and update the checksum (in the final byte of the rom).

## Requirements

1. An XTIDE using a recent version of the ROM (tested with r625) and using one of the ROMs which has VeryLateInit (this
   is the default in the XT rom in the r625 XTIDE rom)
2. An up to date version of the Bridgeboard software, tested with Janus Handler Version 36.85 and Janus Library Version
   36.83), I used The AmigaJanus 2.1 package from https://amiga.resource.cx/exp/a2286at

## Usage

To create a new pc.boot file with the patch applied:

```
$ bridgeboard-pc-boot-patcher <path_to_pc.boot> write-rom <new_pc.boot_file_name> --patch-rom
```

For example, if the file is pc.boot and you want to create pc.boot.new

```
$ bridgeboard-pc-boot-patcher pc.boot write-rom pc.boot.new --patch-rom
ORIGINAL_ROM_SIZE: 0x2000
PATCHED_ROM_SIZE: 0x2000
Rom written to pc.boot.new
```

You should then take that pc.boot file and copy it into SYS:PC/System/pc.boot, I strongly suggest keeping a backup of
pc.boot on the amiga, and also if you have an aboot.ctrl file to rename it:

For example, the file pc.boot.new is already in PC/System:

```
cd SYS:PC/System
copy pc.boot pc.boot.original
delete pc.boot
copy pc.boot.new pc.boot

# Optionally if you have an aboot.ctrl:
rename aboot.ctrl aboot.ctrl.original
```

Make sure you have the Memory map in PCPrefs set to the D000 range.

Now reboot the Amiga with the XTIDE in.

## Current Status

The ROM patch has been tested with Amiga Janus 2.1 only, and only on an Amiga 2000 with an A2286 Bridgeboard.

I've only been able to test the built executable on macOS.
