mc-read-trace
=============

This is a hackathon project to try to reimplement a tool based on Sam Moelius'
description.

Expected usage
--------------

1. Build our patched qemu.

On ubuntu you may need:

```
sudo apt-get install ninja libpixman-dev lib-glibdev
```

Then

```
./configure
make qemu-x86_64
```

Then copy `build/qemu-x86_64` to your path somewhere, like `~/.local/bin/`.

2. Build the parsing tool, and place it somewhere in your path (for simplicity)

```
qemu/qemu-mirror/mc-read-trace$ cargo build --release
   Compiling mc-read-trace v0.1.0 (/home/chris/qemu/qemu-mirror/mc-read-trace)
    Finished release [optimized] target(s) in 1.05s
qemu/qemu-mirror/mc-read-trace$ cp target/release/mc-read-trace ~/.local/bin/
```

3. Build our unit test executable appropriately.

In this case, I'll build the mc-oblivious repo, and target the mc-oblivious-ram crate.
Before doing this, I went to `mc-oblivious/oblivious-ram/src/lib.rs` and removed most
`#[test]` annotations except from `exercise_path_oram_8192_z4`, and reduced the number
of repetitions from `20_000` to `20`.

```
mobliecoinofficial/mc-oblivious$ RUSTFLAGS='-Cinline-threshold=0 -Cdebug-assertions=off -C target-cpu=skylake' cargo test --no-run --target x86_64-unknown-linux-musl --release
```

4. Run qemu against the unit test executable.

```
$ qemu-x86_64 -d in_asm,nochain target/x86_64-unknown-linux-musl/release/deps/mc_oblivious_ram-2d3859def83e3986 --test-threads=1 2>capture
```

This captures the in_asm output to a file called `capture`.

5. Run the parsing tool against the captured output.

```
cat capture | mc-read-trace mc_oblivious_ram
```

You should see output like:

```
Parsing in_asm blocks, with target_str = mc_oblivious_ram
2023-01-19 01:56:21.090007797 UTC INFO [src/main.rs:220] Finished parsing. Found 38 symbols.
2023-01-19 01:56:21.090020765 UTC INFO [src/main.rs:221] The following symbols had multiple traces:
2023-01-19 01:56:21.090099558 UTC INFO [src/main.rs:225] Symbol  _ZN112_$LT$mc_oblivious_ram..position_map..TrivialPositionMap$LT$R$GT$$u20$as$u20$mc_oblivious_traits..PositionMap$GT$5write17hc59c11b3848fff34E repeated 3 times:
2023-01-19 01:56:21.090104473 UTC INFO [src/main.rs:225] Symbol  _ZN128_$LT$mc_oblivious_ram..position_map..ORAMU32PositionMap$LT$ValueSize$C$O$C$R$GT$$u20$as$u20$mc_oblivious_traits..PositionMap$GT$3len17he160b2804b719098E repeated 2 times:
2023-01-19 01:56:21.090108175 UTC INFO [src/main.rs:225] Symbol  _ZN128_$LT$mc_oblivious_ram..position_map..ORAMU32PositionMap$LT$ValueSize$C$O$C$R$GT$$u20$as$u20$mc_oblivious_traits..PositionMap$GT$5write17he3187a026c0152bcE repeated 3 times:
2023-01-19 01:56:21.090111121 UTC INFO [src/main.rs:225] Symbol  _ZN140_$LT$mc_oblivious_ram..evictor..PathOramDeterministicEvictor$u20$as$u20$mc_oblivious_ram..evictor..EvictionStrategy$LT$ValueSize$C$Z$GT$$GT$26evict_from_stash_to_branch17h9cdae145008e799aE repeated 9 times:
2023-01-19 01:56:21.090114106 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor14prepare_target17h57e857da36b105a1E repeated 8 times:
2023-01-19 01:56:21.090116873 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor15prepare_deepest17h9d899c68f74d82c2E repeated 12 times:
2023-01-19 01:56:21.090119400 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor15prepare_deepest43update_goal_and_deepest_for_a_single_bucket17h25c80cb7b86c57e2E.llvm.784265846156795467 repeated 5 times:
2023-01-19 01:56:21.090122305 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor34index_of_deepest_block_from_bucket17h3ebac5b59faf1505E repeated 5 times:
2023-01-19 01:56:21.090125343 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor5tests15test_like_paper17had2218da724211adE repeated 26 times:
2023-01-19 01:56:21.090127900 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor5tests23test_bucket_has_vacancy17ha93a5ee2399ee22fE repeated 7 times:
2023-01-19 01:56:21.090130745 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor5tests27prepare_branch_from_buckets17h801079f8e6a30d57E repeated 10 times:
2023-01-19 01:56:21.090133640 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor5tests31populate_branch_with_fixed_data17h025602bb676cd9a2E repeated 33 times:
2023-01-19 01:56:21.090136208 UTC INFO [src/main.rs:225] Symbol  _ZN16mc_oblivious_ram7evictor5tests39prepare_target_nonoblivious_for_testing17h64b542562bc4dcc4E repeated 9 times:
```
