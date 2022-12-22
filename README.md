# surena_game_rs

_surena_game_rs_ is a wrapper library for the game API of the
[_surena_](https://github.com/RememberOfLife/surena/) game engine.
With _surena_game_rs_, you can program board games for _surena_ in **safe Rust**
and do not have to deal with the FFI interface.

## Getting Started

1. Update the `surena` submodule:
   ```
   $ git submodule update
   ```
2. Have a look at the surena game API:
   [`game.h`](https://github.com/RememberOfLife/surena/blob/master/includes/surena/game.h)
3. Examine the example project: `./example`.
4. Generate the documentation:
   ```
   $ cargo doc --package surena_game --open
   ```
5. Create your own game using the example project as a base.

## Project Layout

- `./src` - The source code of the wrapper library.
- `./example` - An example game implementation of _Nim_ to show off the wrapper
  interface.
- `./build.rs` - Build script to generate _surena_ game API bindings.

## TODOs

- Rename repository to surena_rs
- Implement engine wrapper
- Implementing missing API wrappers
- Testing

## Libraries

This project uses the following libraries:

- [_mirabel_sys_](https://github.com/vilaureu/mirabel_sys) under the
  [_MIT License_](https://github.com/vilaureu/mirabel_sys/blob/master/LICENSE)

## License

See the `LICENSE` file.
