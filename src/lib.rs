#![warn(missing_docs)]
//! This library emulates java try catch using panics and panic handlers in rust.
//!
//! The idea is the user can define custom exceptions that implement the [Exception] trait.
//!
//! Then he can use [throw] to throw those exceptions and [catch] to catch it at any point of the
//! call stack.
//!
//! Here is an example:
//! ```rust
//!    use trycatch::{Exception,throw,catch,CatchError};
//!
//!    // Specificy the exception payload, this can be anytype. It can be reterived later with `Exception::payload()`
//!    type Payload = &'static str;
//!
//!    // Create our custom exception and implement `Exception` trait on it
//!    struct MyE {}
//!
//!    impl Exception<Payload> for MyE {
//!        fn payload(&self) -> Payload {
//!            "MyE exception"
//!        }
//!    }
//!
//!    // Our example of a call stack.
//!    fn nested() {
//!        fn call() {
//!            fn stack() {
//!                // throw our exception here
//!                throw(MyE {});
//!            }
//!            stack();
//!        }
//!        call();
//!    }
//!
//!    // Run our normal callstack inside a `catch` call.
//!    // `catch` needs to know the exception type.
//!    // The result is `CatchError` which is either an exception or a normal panic
//!    let result = catch! {nested() => MyE};
//!
//!    if let Err(CatchError::Exception(e)) = result {
//!        assert_eq!(e.payload(), MyE {}.payload());
//!    } else {
//!        panic!("test failed");
//!    }
//! ```
//!

use std::any::Any;
use std::panic;

/// The result of [catch]\
/// It can be either an exception or a normal panic
pub enum CatchError<E> {
    /// User exception
    Exception(E),
    /// Normal panic
    Panic(Box<dyn Any + Send>),
}

/// Run an expression and catch its exceptions and panics.\
/// It needs to know the exception type.
#[macro_export]
macro_rules! catch {
    ($e: expr => $exception: ty) => {{
        // The the exception needs to be specified as a type argument.
        // This method removes panic error messages from exceptions while the guard is alive.\
        fn register_catch<P: 'static>() -> impl Drop {
            let last_hook = std::panic::take_hook();

            std::panic::set_hook(Box::new(|panic_info| {
                if panic_info.payload().downcast_ref::<P>().is_some() {
                    // Don't print anything
                } else {
                    // ~= Normal flow
                    eprintln!("{}", panic_info)
                }
            }));
            struct Unregister(
                Option<Box<dyn for<'r, 's> Fn(&'r std::panic::PanicInfo<'s>) + Send + Sync>>,
            );
            impl Drop for Unregister {
                fn drop(&mut self) {
                    std::panic::set_hook(self.0.take().unwrap());
                }
            }
            Unregister(Some(last_hook))
        }
        {
            let _g = register_catch::<$exception>();
            std::panic::catch_unwind(move || $e).map_err(|e| {
                if e.is::<$exception>() {
                    CatchError::Exception(e.downcast::<$exception>().unwrap())
                } else {
                    CatchError::Panic(e)
                }
            })
        }
    }};
}

/// User defined exception needs to implement this trait.\
/// It can have an arbitrary payload that can be retrieved with with [Exception::payload].
pub trait Exception<P = ()>: 'static + Send {
    /// Arbitrary payload.
    fn payload(&self) -> P;
}
impl<P: 'static> Exception<P> for Box<dyn Exception<P>> {
    fn payload(&self) -> P {
        (**self).payload()
    }
}

/// Throw an exception that can be caught with [catch]
pub fn throw<P: 'static, E: Exception<P>>(e: E) {
    panic::panic_any(e);
}

#[test]
fn it() {
    type Payload = String;
    struct MyE {}
    impl Exception<Payload> for MyE {
        fn payload(&self) -> Payload {
            "MyE exception".into()
        }
    }
    fn a() {
        fn b() {
            fn c() {
                throw(MyE {});
            }
            c();
        }
        b();
    }

    let r = catch! {a() => MyE};

    if let Err(CatchError::Exception(e)) = r {
        assert_eq!(e.payload(), MyE {}.payload());
    } else {
        panic!("test failed");
    }
}

#[test]
fn simple() {
    struct E;
    impl Exception for E {
        fn payload(&self) {}
    }
    let r = catch! {throw(E) => E};
    if let Err(CatchError::Exception(e)) = r {
        assert!(matches!(*e, E));
    } else {
        panic!("test failed");
    }
}
