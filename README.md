# bridgeboard-pc-boot-patcher

[![Post-merge Test and Release](https://github.com/jfharden/bridgeboard-pc-boot-patcher/actions/workflows/post-merge.yml/badge.svg?branch=main)](https://github.com/jfharden/bridgeboard-pc-boot-patcher/actions/workflows/post-merge.yml)

Release: [latest](https://github.com/jfharden/bridgeboard-pc-boot-patcher/releases/latest)

Patch the Amiga Bridgeboard option rom in the pc.boot file in order to not use the autoboot functionality of the
bridgeboard. This should allow an XTIDE set in the 0xC000 memory range to be used.

It can also validate option rom checksums, and update the checksum (in the final byte of the rom).

## Current Status

The ROM validation and checksum updating works. The current status of the patch to the option rom contained in the
pc.boot file doesn't work and is under current development.

A more comprehensive README update will come soon.
