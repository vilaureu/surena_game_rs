//! This is a wrapper library for the game API of the
//! [_surena_](https://github.com/RememberOfLife/surena/) game engine.

pub mod ptrvec;
pub mod surena;

pub use ptrvec::PtrVec;
pub use surena::{
    buf_sizer, game_feature_flags, game_methods, move_code, player_id, semver, MOVE_NONE,
    PLAYER_NONE, PLAYER_RAND,
};

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    num::NonZeroU8,
    os::raw::c_char,
    ptr::{self, null, null_mut},
};

/// This automatically exports the game API version to the outside.
///
/// You do not have to use this function yourself.
#[no_mangle]
pub extern "C" fn plugin_get_game_capi_version() -> u64 {
    surena::SURENA_GAME_API_VERSION
}

/// This macro creates the `plugin_get_game_methods` function.
///
/// Is must be supplied with all [`game_methods`](surena::game_methods) which
/// should be exported.
/// These can be generated using [`create_game_methods`].
/// This method can only be called once but with multiple methods.
///
/// ## Example
/// ```ignore
/// plugin_get_game_methods!(create_game_methods::<MyGame>(metadata));
/// ```
#[macro_export]
macro_rules! plugin_get_game_methods {
    ( $( $x:expr ),* ) => {
        static mut PLUGIN_GAME_METHODS: ::std::mem::MaybeUninit<
            [$crate::surena::game_methods; $crate::count!($($x),*)]
        > = ::std::mem::MaybeUninit::uninit();

        #[no_mangle]
        pub unsafe extern "C" fn plugin_get_game_methods(
            count: *mut u32,
            methods: *mut *const $crate::surena::game_methods,
        ) {
            count.write($crate::count!($($x),*));
            if methods.is_null() {
                return;
            }

            let src = ::std::mem::MaybeUninit::write(&mut self::PLUGIN_GAME_METHODS, [$($x),*]);
            for i in 0..$crate::count!($($x),*) {
                methods.add(i).write(&src[i]);
            }
        }
    };
}

/// Type for C-compatible error strings.
///
/// This allows to have no error string (`None`), a static error string
/// (`Cow::Borrowed`), or a dynamic error string (`Cow::Owned`).
pub type ErrorString = Option<Cow<'static, CStr>>;

/// Error type for API functions.
///
/// The surena game API always expects an error code and optionally an error
/// message.
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: ErrorString,
}

impl Error {
    /// Create an error from a static C string.
    ///
    /// # Panics
    /// Panics if the byte string is not **nul-terminated**.
    ///
    /// # Example
    /// ```
    /// # use surena_game::*;
    /// Error::new_static(ErrorCode::InvalidInput, b"state string malformed\0");
    /// ```
    pub fn new_static(code: ErrorCode, message: &'static [u8]) -> Self {
        Error {
            code,
            message: Some(Cow::Borrowed(
                CStr::from_bytes_with_nul(message).expect("C string not null-terminated"),
            )),
        }
    }

    /// Create an error from a [`String`].
    ///
    /// It removes NUL bytes from `message` for C compatibility.
    ///
    /// # Example
    /// ```
    /// # use surena_game::*;
    /// Error::new_dynamic(ErrorCode::InvalidOptions, format!("board size larger than {}", 42));
    /// ```
    pub fn new_dynamic(code: ErrorCode, mut message: String) -> Self {
        message.retain(|c| c != '\0');
        Error {
            code,
            message: Some(Cow::Owned(CString::new(message).unwrap())),
        }
    }
}

impl From<ErrorCode> for Error {
    /// Create an error without a `message`.
    #[inline]
    fn from(code: ErrorCode) -> Self {
        Self {
            code,
            message: Default::default(),
        }
    }
}

/// _surena_ error codes as a Rust enum.
///
/// Custom error codes are currently not supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCode {
    StateUnrecoverable,
    StateCorrupted,
    OutOfMemory,
    FeatureUnsupported,
    StateUninitialized,
    InvalidInput,
    InvalidOptions,
    UnstablePosition,
    SyncCounterMismatch,
    Retry,
}

impl From<ErrorCode> for surena::error_code {
    fn from(error: ErrorCode) -> Self {
        match error {
            ErrorCode::StateUnrecoverable => surena::ERR_ERR_STATE_UNRECOVERABLE,
            ErrorCode::StateCorrupted => surena::ERR_ERR_STATE_CORRUPTED,
            ErrorCode::OutOfMemory => surena::ERR_ERR_OUT_OF_MEMORY,
            ErrorCode::FeatureUnsupported => surena::ERR_ERR_FEATURE_UNSUPPORTED,
            ErrorCode::StateUninitialized => surena::ERR_ERR_STATE_UNINITIALIZED,
            ErrorCode::InvalidInput => surena::ERR_ERR_INVALID_INPUT,
            ErrorCode::InvalidOptions => surena::ERR_ERR_INVALID_OPTIONS,
            ErrorCode::UnstablePosition => surena::ERR_ERR_UNSTABLE_POSITION,
            ErrorCode::SyncCounterMismatch => surena::ERR_ERR_SYNC_COUNTER_MISMATCH,
            ErrorCode::Retry => surena::ERR_ERR_RETRY,
        }
    }
}

/// Result type using the special [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;

macro_rules! surena_try {
    ( $game: expr, $result:expr ) => {
        match $result {
            Ok(v) => v,
            Err(error) => {
                Aux::get($game).set_error(error.message);
                return error.code.into();
            }
        }
    };
}

/// A [`PtrVec`] for writing and returning C strings.
pub type StrBuf<'b> = PtrVec<'b, NonZeroU8>;

/// Main trait which needs to be implemented by your game struct.
///
/// See `./surena/includes/surena/game.h` for API documentation.
/// You should **not implement `[...]_wrapped`** methods.
///
/// Games need to implement [`Drop`] for custom `destroy` handling.
/// `clone` is handled by the [`Clone`] implementation and `compare` by [`Eq`].
/// The [`Send`] bound is required by the surena API.
///
/// # Example
/// See the `./example` crate in the project root.
pub trait GameMethods: Sized + Clone + Eq + Send {
    /// Also invoked when the `create_with_opts_str` `string` is NULL.
    fn create_default() -> Result<(Self, buf_sizer)>;
    fn copy_from(&mut self, other: &mut Self) -> Result<()>;
    fn import_state(&mut self, string: Option<&str>) -> Result<()>;
    fn export_state(&mut self, str_buf: &mut StrBuf) -> Result<()>;
    fn players_to_move(&mut self, players: &mut PtrVec<player_id>) -> Result<()>;
    fn get_concrete_moves(
        &mut self,
        player: player_id,
        moves: &mut PtrVec<move_code>,
    ) -> Result<()>;
    fn get_move_code(&mut self, player: player_id, string: &str) -> Result<move_code>;
    fn make_move(&mut self, player: player_id, mov: move_code) -> Result<()>;
    fn get_results(&mut self, players: &mut PtrVec<player_id>) -> Result<()>;
    /// Sync counters are currently not supported.
    #[allow(clippy::wrong_self_convention)]
    fn is_legal_move(&mut self, player: player_id, mov: move_code) -> Result<()>;

    /// Must be implemented when the [`game_feature_flags::options`] is enabled.
    #[allow(unused_variables)]
    fn create_with_opts_str(string: &str) -> Result<(Self, buf_sizer)> {
        unimplemented!("create_with_opts_str")
    }
    /// Must be implemented when the [`game_feature_flags::options`] is enabled.
    #[allow(unused_variables)]
    fn export_options_str(&mut self, str_buf: &mut StrBuf) -> Result<()> {
        unimplemented!("export_options_str")
    }
    /// Must be implemented when the [`game_feature_flags::print`] is enabled.
    #[allow(unused_variables)]
    fn debug_print(&mut self, str_buf: &mut StrBuf) -> Result<()> {
        unimplemented!("debug_print")
    }

    #[doc(hidden)]
    unsafe extern "C" fn get_last_error_wrapped(game: *mut surena::game) -> *const c_char {
        Aux::get(game)
            .error
            .as_ref()
            .map(|message| message.as_ptr())
            .unwrap_or(null())
    }

    #[doc(hidden)]
    unsafe extern "C" fn create_with_opts_str_wrapped(
        game: *mut surena::game,
        string: *const c_char,
    ) -> surena::error_code {
        match surena_try!(game, cstring_to_rust(string)) {
            Some(string) => create(game, || Self::create_with_opts_str(string)),
            None => create(game, Self::create_default),
        }
    }

    #[doc(hidden)]
    unsafe extern "C" fn create_default_wrapped(game: *mut surena::game) -> surena::error_code {
        create(game, Self::create_default)
    }

    #[doc(hidden)]
    unsafe extern "C" fn export_options_str_wrapped(
        game: *mut surena::game,
        ret_size: *mut usize,
        str_buf: *mut c_char,
    ) -> surena::error_code {
        let mut str_buf = StrBuf::from_c_char(str_buf, get_sizer(game).options_str);
        surena_try!(
            game,
            get_data::<Self>(game).export_options_str(&mut str_buf)
        );
        str_buf.nul_terminate();
        ret_size.write(str_buf.len());

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn destroy_wrapped(game: *mut surena::game) -> surena::error_code {
        let data = &mut (*game).data1;
        if !data.is_null() {
            Box::from_raw(*data);
            // Leave as null pointer to catch use-after-free errors.
            *data = null_mut();
        }
        Aux::free(game);

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn clone_wrapped(
        game: *mut surena::game,
        clone_target: *mut surena::game,
    ) -> surena::error_code {
        clone_target.copy_from_nonoverlapping(game, 1);

        Aux::init(game);
        // Initialize data1 to zero in case clone fails.
        ptr::write(&mut (*game).data1, null_mut());

        let data = get_data::<Self>(game).clone();
        // data1 is already initialized.
        (*game).data1 = Box::into_raw(Box::new(data)).cast();

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn copy_from_wrapped(
        game: *mut surena::game,
        other: *mut surena::game,
    ) -> surena::error_code {
        let other = get_data::<Self>(other);
        surena_try!(game, get_data::<Self>(game).copy_from(other));

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn compare_wrapped(
        game: *mut surena::game,
        other: *mut surena::game,
        ret_equal: *mut bool,
    ) -> surena::error_code {
        let other = get_data::<Self>(other);
        ret_equal.write(get_data::<Self>(game).eq(&other));

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn import_state_wrapped(
        game: *mut surena::game,
        string: *const c_char,
    ) -> surena::error_code {
        let string = surena_try!(game, cstring_to_rust(string));
        surena_try!(game, get_data::<Self>(game).import_state(string));

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn export_state_wrapped(
        game: *mut surena::game,
        ret_size: *mut usize,
        str_buf: *mut c_char,
    ) -> surena::error_code {
        let mut str_buf = StrBuf::from_c_char(str_buf, get_sizer(game).state_str);
        surena_try!(game, get_data::<Self>(game).export_state(&mut str_buf));
        str_buf.nul_terminate();
        ret_size.write(str_buf.len());

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn players_to_move_wrapped(
        game: *mut surena::game,
        ret_count: *mut u8,
        players: *mut player_id,
    ) -> surena::error_code {
        let mut players = PtrVec::new(players, get_sizer(game).max_players_to_move.into());
        surena_try!(game, get_data::<Self>(game).players_to_move(&mut players));
        ret_count.write(players.len() as u8);

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn get_concrete_moves_wrapped(
        game: *mut surena::game,
        player: player_id,
        ret_count: *mut u32,
        moves: *mut move_code,
    ) -> surena::error_code {
        let mut moves = PtrVec::new(moves, get_sizer(game).max_moves as usize);
        surena_try!(
            game,
            get_data::<Self>(game).get_concrete_moves(player, &mut moves)
        );
        ret_count.write(moves.len() as u32);

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn is_legal_move_wrapped(
        game: *mut surena::game,
        player: player_id,
        mov: move_code,
        _sync: surena::sync_counter,
    ) -> surena::error_code {
        surena_try!(game, get_data::<Self>(game).is_legal_move(player, mov));

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn make_move_wrapped(
        game: *mut surena::game,
        player: player_id,
        mov: move_code,
    ) -> surena::error_code {
        surena_try!(game, get_data::<Self>(game).make_move(player, mov));

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn get_results_wrapped(
        game: *mut surena::game,
        ret_count: *mut u8,
        players: *mut player_id,
    ) -> surena::error_code {
        let mut players = PtrVec::new(players, get_sizer(game).max_results.into());
        surena_try!(game, get_data::<Self>(game).get_results(&mut players));
        ret_count.write(players.len() as u8);

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn get_move_code_wrapped(
        game: *mut surena::game,
        player: player_id,
        string: *const c_char,
        ret_move: *mut move_code,
    ) -> surena::error_code {
        let string = surena_try!(game, cstring_to_rust(string)).expect("move string was NULL");
        let result = surena_try!(game, get_data::<Self>(game).get_move_code(player, string));
        ret_move.write(result);

        surena::ERR_ERR_OK
    }

    #[doc(hidden)]
    unsafe extern "C" fn debug_print_wrapped(
        game: *mut surena::game,
        ret_size: *mut usize,
        str_buf: *mut c_char,
    ) -> surena::error_code {
        let mut str_buf = StrBuf::from_c_char(str_buf, get_sizer(game).print_str);
        surena_try!(game, get_data::<Self>(game).debug_print(&mut str_buf));
        str_buf.nul_terminate();
        ret_size.write(str_buf.len());

        surena::ERR_ERR_OK
    }
}

/// Non-function members for [`game_methods`].
///
/// # Example
/// ```
/// # use surena_game::*;
/// use std::ffi::CStr;
///
/// let mut features = game_feature_flags::default();
/// features.set_print(true);
///
/// let metadata = Metadata {
///     game_name: CStr::from_bytes_with_nul(b"Example\0").unwrap(),
///     variant_name: CStr::from_bytes_with_nul(b"Standard\0").unwrap(),
///     impl_name: CStr::from_bytes_with_nul(b"surena_game_rs\0").unwrap(),
///     version: semver {
///         major: 0,
///         minor: 1,
///         patch: 0,
///     },
///     features,
/// };
/// ```
pub struct Metadata {
    pub game_name: &'static CStr,
    pub variant_name: &'static CStr,
    pub impl_name: &'static CStr,
    pub version: semver,
    pub features: game_feature_flags,
}

/// Create _surena_ [`game_methods`] from game struct `G` and `metadata`.
///
/// If feature flags are disabled, corresponding function pointers will be set
/// to zero.
///
/// # Example
/// ```ignore
/// create_game_methods::<MyGame>(metadata);
/// ```
pub fn create_game_methods<G: GameMethods>(metadata: Metadata) -> game_methods {
    game_methods {
        game_name: metadata.game_name.as_ptr(),
        variant_name: metadata.variant_name.as_ptr(),
        impl_name: metadata.impl_name.as_ptr(),
        version: metadata.version,
        features: metadata.features,
        get_last_error: Some(G::get_last_error_wrapped),
        create_with_opts_str: if metadata.features.options() {
            Some(G::create_with_opts_str_wrapped)
        } else {
            None
        },
        create_default: Some(G::create_default_wrapped),
        export_options_str: if metadata.features.options() {
            Some(G::export_options_str_wrapped)
        } else {
            None
        },
        destroy: Some(G::destroy_wrapped),
        clone: Some(G::clone_wrapped),
        copy_from: Some(G::copy_from_wrapped),
        compare: Some(G::compare_wrapped),
        import_state: Some(G::import_state_wrapped),
        export_state: Some(G::export_state_wrapped),
        players_to_move: Some(G::players_to_move_wrapped),
        get_concrete_moves: Some(G::get_concrete_moves_wrapped),
        is_legal_move: Some(G::is_legal_move_wrapped),
        make_move: Some(G::make_move_wrapped),
        get_results: Some(G::get_results_wrapped),
        get_move_code: Some(G::get_move_code_wrapped),
        debug_print: if metadata.features.print() {
            Some(G::debug_print_wrapped)
        } else {
            None
        },
        ..Default::default()
    }
}

/// Simple helper function to create a [`CStr`] from a byte literal.
///
/// # Panics
/// `bytes` must be NUL terminated and must not contain any other NUL byte.
///
/// # Example
/// ```
/// # use surena_game::cstr;
/// cstr(b"my C-style string\0");
/// ```
#[inline]
pub fn cstr(bytes: &[u8]) -> &CStr {
    CStr::from_bytes_with_nul(bytes).expect("invalid C string")
}

#[derive(Default)]
struct Aux {
    error: ErrorString,
}

impl Aux {
    unsafe fn init(game: *mut surena::game) {
        ptr::write(
            &mut (*game).data2,
            Box::into_raw(Box::<Self>::new(Self::default())).cast(),
        );
    }

    #[inline]
    unsafe fn get<'l>(game: *mut surena::game) -> &'l mut Self {
        &mut *(*game).data2.cast()
    }

    unsafe fn free(game: *mut surena::game) {
        let aux = (*game).data2;
        if !aux.is_null() {
            Box::<Self>::from_raw(aux.cast());
            // Leave as null pointer to catch use-after-free errors.
            (*game).data2 = null_mut();
        }
    }

    #[inline]
    fn set_error(&mut self, error: ErrorString) {
        self.error = error;
    }
}

#[inline]
unsafe fn get_data<'l, G>(game: *mut surena::game) -> &'l mut G {
    &mut *(*game).data1.cast()
}

#[inline]
unsafe fn get_features(game: *mut surena::game) -> game_feature_flags {
    (*(*game).methods).features
}

#[inline]
unsafe fn get_sizer<'l>(game: *mut surena::game) -> &'l buf_sizer {
    &(*game).sizer
}

fn check_sizer(sizer: &buf_sizer, features: game_feature_flags) {
    const FAILURE: &str = "string buffer length must not be 0";

    if features.options() {
        assert!(sizer.options_str > 0, "{FAILURE}");
    }
    assert!(sizer.state_str > 0, "{FAILURE}");
    // This can only happen on <32bit platforms:
    let _: usize = sizer
        .max_moves
        .try_into()
        .expect("max_moves does not fit usize");
    assert!(sizer.move_str > 0, "{FAILURE}");
    if features.print() {
        assert!(sizer.print_str > 0, "{FAILURE}");
    }
}

unsafe fn cstring_to_rust<'l>(string: *const c_char) -> Result<Option<&'l str>> {
    Ok(if string.is_null() {
        None
    } else {
        Some(CStr::from_ptr(string).to_str().map_err(|e| {
            Error::new_dynamic(ErrorCode::InvalidInput, format!("UTF-8 error: {e}"))
        })?)
    })
}

unsafe fn create<G, F: FnOnce() -> Result<(G, buf_sizer)>>(
    game: *mut surena::game,
    func: F,
) -> surena::error_code {
    Aux::init(game);
    // Initialize data1 to zero in case creation fails.
    ptr::write(&mut (*game).data1, null_mut());

    let (data, sizer) = surena_try!(game, func());
    check_sizer(&sizer, get_features(game));
    ptr::write(&mut (*game).sizer, sizer);
    // data1 is already initialized.
    (*game).data1 = Box::into_raw(Box::new(data)).cast();

    surena::ERR_ERR_OK
}

/// Internally used by [`plugin_get_game_methods`].
#[macro_export]
macro_rules! count {
    () => { 0 };
    ($_e: expr $(, $rest: expr)*) => { 1 + $crate::count!($($rest),*) }
}
