# trycatch

This library emulates java try catch using panics and panic handlers in rust.

The idea is the user can define custom exceptions that implement the [Exception] trait.

Then he can use [throw] to throw those exceptions and [catch] to catch it at any point of the
call stack.

Here is an example:
```rust
   use trycatch::{Exception,throw,catch,CatchError};

   // Create our custom exception and implement `Exception` trait on it
   struct MyE;

   impl Exception for MyE {
       // Specify the name the exception.
       // We can use this to distinguish this exception type and use it to downcast the payload correctly.
       fn name(&self) -> &'static str {
           "MyE"
       }
       // Specify the exception payload, it can be anytype.
       fn payload(&self) -> Box<dyn std::any::Any> {
           Box::new("MyE exception")
       }
   }

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
   // `catch` needs to know the exception type.
   // The result is `CatchError` which is either an exception or a normal panic
   let result = catch(nested);

   if let Err(CatchError::Exception(e)) = result {
       assert_eq!(e.name(), "MyE");
       assert_eq!(*e.payload().downcast::<&'static str>().unwrap(), "MyE exception");
   } else {
       panic!("test failed");
   }
```

