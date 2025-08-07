use serde::{Deserialize, Serialize};
use std::any::Any;
#[cfg(any(client, doc))]

/// A trait for `struct`s that can be made reactive. Typically, this will be
/// derived with the `#[make_rx]` macro, though it can be implemented manually
/// if you have more niche requirements.
pub trait MakeRx {
    /// The type of the reactive version that we'll convert to. By having this
    /// as an associated type, we can associate the reactive type with the
    /// unreactive, meaning greater inference and fewer arguments that the
    /// user needs to provide to macros.
    type Rx: MakeUnrx;
    /// This should be set to `true` to have this type be ignored by the hot
    /// state reloading system, which can be useful when working with state
    /// that you will regularly update in development.
    ///
    /// For example, take a documentation website that converts Markdown to HTML
    /// in its state generation process, storing that in reactive state. If you,
    /// in development, then wanted to change some of the documentation and see
    /// the result immediately, HSR would override the new state with the
    /// old, preserving your 'position' in the development cycle. Only a
    /// manual reload of the page would prevent this from continuing
    /// forever. Setting this associated constant to `true` will tell the
    /// HSR system to ignore this type from HSR thawing, meaning
    /// the new state will always be used. Note that, depending on the structure
    /// of your application, this can sometimes cause state inconsistencies
    /// (e.g. if one state object expects another one to be in a certain
    /// state, but then one of them is ignored by HSR and resets).
    ///
    /// This is a development-only setting, and does not exist in production. If
    /// the `hsr` feature flag is disabled, this will have no effect.
    ///
    /// Typically, you would set this using the `#[rx(hsr_ignore)]` derive
    /// helper macro with `#[derive(ReactiveState, ..)]`. Note that you only
    /// need to set this to `true` for the top-level state type for a page,
    /// not for any nested components.
    #[cfg(debug_assertions)]
    const HSR_IGNORE: bool = false;
    /// Transforms an instance of the `struct` into its reactive version.
    fn make_rx(self) -> Self::Rx;
}

/// A trait for reactive `struct`s that can be made un-reactive. This is the
/// opposite of `MakeRx`, and is intended particularly for state freezing. Like
/// `MakeRx`, this will usually be derived automatically with the `#[make_rx]`
/// macro, but you can also implement it manually.
///
/// The types that implement this are typically referred to as the *intermediate
/// state* types, as they are rendered far more ergonomic to use by being put
/// through Sycamore's `create_ref()` function.
pub trait MakeUnrx {
    /// The type of the unreactive version that we'll convert to.
    type Unrx: Serialize + for<'de> Deserialize<'de> + MakeRx;
    /// Transforms an instance of the `struct` into its unreactive version. By
    /// having this as an associated type, we can associate the reactive type
    /// with the unreactive, meaning greater inference and fewer arguments
    /// that the user needs to provide to macros.
    fn make_unrx(self) -> Self::Unrx;
    /// Calls all handlers on suspended state, spawning scoped futures for each
    /// of them (the futures *must* be scoped to prevent the same handler
    /// being run multiple times concurrently if a user leaves the page and
    /// then comes back).
    ///
    /// This has no return type, since it simply spawns futures for each of the
    /// user's handlers. Each handler must have the following function
    /// signature:
    ///
    /// ```text
    /// Fn(Scope<'a>, RxRef<'a>);
    /// ```
    ///
    /// Here, `RxRef` denotes the reference `struct` their template would be
    /// provided with. In the case of an individual, non-nested field, this
    /// will be `&'a Signal<T>`, where `T` is the type of the field.
    ///
    /// Fallible handlers should operate on fields with type `Result<T, E>` so
    /// they can propagate errors directly back to the user's template code.
    ///
    /// If you're implementing `MakeUnrx` manually, you can usually leave the
    /// body of this function empty unless you're using the suspended state
    /// system.
    #[cfg(any(client, doc))]
    fn compute_suspense(&self);
}

/// A trait for reactive `struct`s that can be made unreactive and serialized to
/// a `String`. `struct`s that implement this should implement `MakeUnrx` for
/// simplicity, but they technically don't have to (they always do in Perseus
/// macro-generated code).
pub trait Freeze {
    /// 'Freezes' the reactive `struct` by making it unreactive and converting
    /// it to a `String`.
    fn freeze(&self) -> String;
}

/// A convenience super-trait for `Freeze`able things that can be downcast to
/// concrete types.
pub trait AnyFreeze: Freeze + Any {
    /// Gives `&dyn Any` to enable downcasting.
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any + Freeze> AnyFreeze for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl std::fmt::Debug for (dyn AnyFreeze + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // See Rust std/core/any.rs:213
        f.debug_struct("AnyFreeze").finish_non_exhaustive()
    }
}

/// A marker trait for types that you want to be able to use with the Perseus
/// state platform, without using `#[derive(ReactiveState, ..)]`. If you want to
/// use unreactive state, implement this, and you'll automatically be able to
/// use your unreactive type without problems!
///
/// Since unreactive state will never be changed on the client-side, it is
/// automatically ignored y the hot state reloading system, if it is enabled.
pub trait UnreactiveState {}

/// A wrapper for storing unreactive state in Perseus, and allowing it to
/// interface with the (fundamentally reactive) state platform. Generally, you
/// would just use reactive state, however, sometimes, you may wish to use
/// unreactive state without sacrificing features like automatic page state
/// caching: this `struct` allows that.
///
/// This is handled automatically by the `#[template]` macro, and you should
/// never need to use this manually unless you don't use the macros.
///
/// This wrapper will automatically implement all the necessary `trait`s to
/// interface with Perseus' reactive state platform, along with `Serialize` and
/// `Deserialize` (provided the underlying type also implements the latter two).
#[derive(Clone, Debug)]
pub struct UnreactiveStateWrapper<
    T: Serialize + for<'de> Deserialize<'de> + UnreactiveState + Clone,
>(pub T);
// Automatically implement `MakeRx` for any marked unreactive type, using
// `UnreactiveStateWrapper` as the reactive type
impl<T: Serialize + for<'de> Deserialize<'de> + UnreactiveState + Clone> MakeRx for T {
    type Rx = UnreactiveStateWrapper<T>;
    // Unreactive state will never change on the client-side, and should therefore
    // by default be ignored by HSR
    #[cfg(debug_assertions)]
    const HSR_IGNORE: bool = true;
    fn make_rx(self) -> Self::Rx {
        UnreactiveStateWrapper(self)
    }
}
// And let it be converted back
impl<T: Serialize + for<'de> Deserialize<'de> + UnreactiveState + Clone> MakeUnrx
    for UnreactiveStateWrapper<T>
{
    type Unrx = T;
    fn make_unrx(self) -> Self::Unrx {
        self.0
    }
    // Suspense is not allowed on unreactive state
    #[cfg(any(client, doc))]
    fn compute_suspense(&self) {}
}
// And, since the underlying type can be serialized, implement `Freeze`
impl<T: Serialize + for<'de> Deserialize<'de> + UnreactiveState + Clone> Freeze
    for UnreactiveStateWrapper<T>
{
    fn freeze(&self) -> String {
        // Just serialize the underlying type
        serde_json::to_string(&self.0).unwrap()
    }
}
