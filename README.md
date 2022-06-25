# _surena_game_rs_

_surena_game_rs_ is a wrapper library for the game API of the
[_surena_](https://github.com/RememberOfLife/surena/) game engine.
With _surena_game_rs_, you can program board games for _surena_ in **safe Rust**
and do not have to deal with the FFI interface.

## Getting Started

1. Update the `surena` submodule:
   ```
   $ git submodule update
   ```
2. Have a look at the surena game API: `./surena/includes/surena/game.h`
3. Examine the example project: `./example`.
4. Generate the documentation:
   ```
   $ cargo doc --package surena_game --open
   ```
5. Create your own game using the example project as a base.

## Project Layout

- `./src` - The source code of the wrapper library.
- `./surena` - A submodule containing the surena game engine. Used for obtaining
  header files.
- `./example` - An example game implementation of _Nim_ to show off the wrapper
  interface.
- `./build.rs` - Build script to generate _surena_ game API bindings.

## TODOs

- Implementing missing API wrappers
- Testing

## Libraries

This project uses the following libraries:

- [_surena_](https://github.com/RememberOfLife/surena/) under the
  [_MIT License_](https://github.com/RememberOfLife/surena/blob/master/LICENSE)
- [_bindgen_](https://github.com/rust-lang/rust-bindgen) under the
  [_BSD 3-Clause License_](https://github.com/rust-lang/rust-bindgen/blob/master/LICENSE)

## License

See the `LICENSE` file.
