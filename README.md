# trycatch

This library emulates java try catch using panics and panic handlers in rust.

The idea is the user can define custom exceptions that implement the [Exception] trait.

Then he can use [throw] to throw those exceptions and [catch] to catch it at any point of the
call stack.

Here is an example:
```rust
   use trycatch::{Exception,ExceptionDowncast,throw,catch,CatchError};

   // Create our custom exception and implement `Exception` trait on it
   #[derive(Exception)]
   struct MyE;

   // Our example of a call stack.
   fn nested() {
       fn call() {
           fn stack() {
               // throw our exception here
               throw(MyE);
           }
           stack();
       }
       call();
   }

   // Run our normal callstack inside a `catch` call.
   // The result is `CatchError` which is either an exception or a normal panic
   let result = catch(nested);

   if let Err(CatchError::Exception(e)) = result {
       assert_eq!(e.name(), "MyE");
       assert!(matches!(e.downcast::<MyE>(), MyE));
   } else {
       panic!("test failed");
   }
```

