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
//!    // Create our custom exception and implement `Exception` trait on it
//!    #[derive(Exception)]
//!    struct MyE;
//!
//!    // Our example of a call stack.
//!    fn nested() {
//!        fn call() {
//!            fn stack() {
//!                // throw our exception here
//!                throw(MyE);
//!            }
//!            stack();
//!        }
//!        call();
//!    }
//!
//!    // Run our normal callstack inside a `catch` call.
//!    // `catch` needs to know the exception type.
//!    // The result is `CatchError` which is either an exception or a normal panic
//!    let result = catch(nested);
//!
//!    if let Err(CatchError::Exception(e)) = result {
//!        assert_eq!(e.name(), "MyE");
//!        assert!(matches!(*e.into_any().downcast::<MyE>().unwrap(), MyE));
//!    } else {
//!        panic!("test failed");
//!    }
//! ```
//!

use std::any::Any;
use std::panic::{self, UnwindSafe};

/// The result of [catch]\
/// It can be either an exception or a normal panic
pub enum CatchError {
    /// User exception
    Exception(Box<dyn Exception>),
    /// Normal panic
    Panic(Box<dyn Any + Send>),
}

/// Runs a function and catch its exceptions and panics.
pub fn catch(expr: impl FnOnce() + UnwindSafe) -> Result<(), CatchError> {
    // The the exception needs to be specified as a type argument.
    // This method removes panic error messages from exceptions while the guard is alive.\
    fn register_catch() -> impl Drop {
        let last_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(|panic_info| {
            if panic_info
                .payload()
                .downcast_ref::<Box<dyn Exception>>()
                .is_some()
            {
                // Don't print anything
            } else {
                // ~= Normal flow
                eprintln!("{}", panic_info)
            }
        }));
        type PanicHook = dyn for<'r, 's> Fn(&'r std::panic::PanicInfo<'s>) + Send + Sync;
        struct Unregister(Option<Box<PanicHook>>);
        impl Drop for Unregister {
            fn drop(&mut self) {
                std::panic::set_hook(self.0.take().unwrap());
            }
        }
        Unregister(Some(last_hook))
    }
    {
        let _g = register_catch();
        std::panic::catch_unwind(expr).map_err(|e| {
            if e.is::<Box<dyn Exception>>() {
                CatchError::Exception(*e.downcast::<Box<dyn Exception>>().unwrap())
            } else {
                CatchError::Panic(e)
            }
        })
    }
    //}};
}

/// User defined exception needs to implement this trait.\
/// The concrete exception type can be reterived with [Exception::into_any] and [Any::downcast]
pub trait Exception: 'static + Send {
    /// The name of the exception, useful to figure out the type of dyn exception before
    /// downcasting it to a concrete type
    fn name(&self) -> &'static str {
        unimplemented!()
    }
    /// Cast Box<dyn Exception> to <dyn Any>, useful inorder to retrieve the concrete exception type.
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}
impl Exception for Box<dyn Exception> {
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Throw an exception that can be caught with [catch]
pub fn throw(e: impl Exception) {
    panic::panic_any(Box::new(e) as Box<dyn Exception>);
}
pub use trycatch_derive::Exception;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it() {
        #[derive(Exception)]
        struct MyE {}
        fn a() {
            fn b() {
                fn c() {
                    throw(MyE {});
                }
                c();
            }
            b();
        }

        let r = catch(a);

        if let Err(CatchError::Exception(e)) = r {
            assert!(matches!(*e.into_any().downcast::<MyE>().unwrap(), MyE {}));
        } else {
            panic!("test failed");
        }
    }

    #[test]
    fn catch_panics() {
        assert!(matches!(
            catch(|| panic!("this is an intended test panic")),
            Err(CatchError::Panic(_))
        ));
    }

    #[test]
    fn simple() {
        #[derive(Exception)]
        struct E;
        let r = catch(|| throw(E));
        if let Err(CatchError::Exception(e)) = r {
            assert!(matches!(*e.into_any().downcast::<E>().unwrap(), E));
        } else {
            panic!("test failed");
        }
    }

    #[test]
    fn multi_exception() {
        #[derive(Exception)]
        struct A;
        #[derive(Exception)]
        struct B;

        fn c() {
            throw(B);
            throw(A);
        }
        let r = catch(c);

        if let Err(CatchError::Exception(e)) = r {
            match e.name() {
                "A" => assert!(matches!(*e.into_any().downcast::<A>().unwrap(), A)),
                "B" => assert!(matches!(*e.into_any().downcast::<B>().unwrap(), B)),
                _ => unreachable!(),
            }
        } else {
            panic!("test failed");
        }
    }

    #[test]
    fn simpler() {
        #[derive(Exception)]
        struct A;

        assert!(matches!(catch(|| throw(A)), Err(CatchError::Exception(_))))
    }

    #[test]
    fn complex() {
        #[derive(Exception)]
        struct A(B);
        #[derive(Exception)]
        struct B;

        let excep_b = if let Err(CatchError::Exception(excep_b)) = catch(|| {
            let excep_a = if let Err(CatchError::Exception(excep_a)) = catch(|| {
                throw(A(B));
            }) {
                assert_eq!(excep_a.name(), "A");
                *excep_a.into_any().downcast::<A>().unwrap()
            } else {
                unreachable!()
            };
            throw(excep_a.0);
        }) {
            excep_b
        } else {
            unreachable!()
        };
        assert_eq!(excep_b.name(), "B");
    }
}
