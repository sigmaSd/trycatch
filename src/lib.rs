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
//!    use trycatch::{Exception,throw,catch,CatchError, register_catch};
//!
//!    // Specificy the exception payload, this can be anytype. It can be reterived later with `Exception::payload()`
//!    type Payload = &'static str;
//!
//!    // Create our custom exception and implement `Exception` trait on it
//!    #[derive(Debug)]
//!    struct MyE {}
//!
//!    impl Exception<Payload> for MyE {
//!        fn payload(&self) -> Payload {
//!            "MyE exception"
//!        }
//!    }
//!
//!    // This method removes panic error messages from exceptions while the guard is alive.
//!    // The payload of the exception needs to be specified as a type argument.
//!    let _g = register_catch::<Payload>();
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
//!    // `catch` needs to know the payload type.
//!    // The result is `CatchError` which is either an exception or a normal panic
//!    let result = catch! {nested() => <Payload>};
//!
//!    if let Err(CatchError::Exception(e)) = result {
//!        assert_eq!(e.payload(), MyE {}.payload());
//!    } else {
//!        panic!("test failed");
//!    }
//! ```
//!

use std::any::Any;
use std::fmt::Debug;
use std::panic;

/// This method removes panic error messages from exceptions while the guard is alive.\
/// The payload of the exception needs to be specified as a type argument.
#[must_use]
pub fn register_catch<P: 'static>() -> impl Drop {
    std::panic::set_hook(Box::new(|panic_info| {
        if panic_info
            .payload()
            .downcast_ref::<Box<dyn Exception<P>>>()
            .is_some()
        {
            // Don't print anything
        } else {
            // ~= Normal flow
            eprintln!("{}", panic_info)
        }
    }));
    struct Unregister;
    impl Drop for Unregister {
        fn drop(&mut self) {
            let _ = std::panic::take_hook();
        }
    }
    Unregister
}

/// The result of [catch]\
/// It can be either an exception or a normal panic
#[derive(Debug)]
pub enum CatchError<P> {
    /// User exception
    Exception(Box<dyn Exception<P>>),
    /// Normal panic
    Panic(Box<dyn Any + Send>),
}

/// Run an expression and catch its exceptions and panics.\
/// It needs to know the exception payload type.
#[macro_export]
macro_rules! catch {
    ($e: expr => <$payload: ty>) => {
        std::panic::catch_unwind(move || $e).map_err(|e| {
            if e.is::<Box<dyn Exception<$payload>>>() {
                CatchError::Exception(e.downcast::<Box<dyn Exception<$payload>>>().unwrap())
            } else {
                CatchError::Panic(e)
            }
        })
    };
}

/// User defined exception needs to implement this trait.\
/// It can have an arbitrary payload that can be retrieved with with [Exception::payload].
pub trait Exception<P>: 'static + Send + Debug {
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
    panic::panic_any(Box::new(e) as Box<dyn Exception<P>>)
}

#[test]
fn it() {
    type Payload = String;
    #[derive(Debug)]
    struct MyE {}
    impl Exception<Payload> for MyE {
        fn payload(&self) -> Payload {
            "MyE exception".into()
        }
    }
    let _g = register_catch::<Payload>();
    fn a() {
        fn b() {
            fn c() {
                throw(MyE {});
            }
            c();
        }
        b();
    }

    let r = catch! {a() => <Payload>};

    if let Err(CatchError::Exception(e)) = r {
        assert_eq!(e.payload(), MyE {}.payload());
    } else {
        panic!("test failed");
    }
}
