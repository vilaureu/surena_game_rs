# _Nim_ Example Game

This example game of _Nim_ shows how to use the _surena_game_ wrapper library.

## Getting Started

1. Build the project:
   ```
   $ cargo build
   ```
2. Build the surena project (in `../surena`):
   ```
   $ cmake -S . -B build`
   $ cmake --build build
   ```
3. Execute the example project (in the project root):
   ```
   $ ./surena/build/surena --game-plugin ./target/debug/libexample.so
   ```

## Documentation

The source code is documented and, additionally, some documentation can be
generated using:

```
$ cargo doc --no-deps --document-private-items --open
```
