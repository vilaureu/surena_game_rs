# _Nim_ Example Game

This example game of _Nim_ shows how to use the _surena_game_ wrapper library.

## Getting Started

1. Build the project:
   ```
   $ cargo build
   ```
2. Checkout and build the _surena_ project matching the version used by
   _mirabel_sys_:
   ```
   $ cmake -S <surena project checkout> -B build`
   $ cmake --build build
   ```
3. Execute the example project (in the project root):
   ```
   $ ./build/surena --game-plugin ./target/debug/libexample.so
   ```

## Documentation

The source code is documented and, additionally, some documentation can be
generated using:

```
$ cargo doc --no-deps --document-private-items --open
```
