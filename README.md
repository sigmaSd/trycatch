# trycatch

This library emulates java try catch using panics and panic handlers in rust.

The idea is the user can define custom exceptions that implement the [Exception] trait.

Then he can use [throw] to throw those exceptions and [catch] to catch it at any point of the
call stack.

Here is an example:
```rust
   use trycatch::{Exception,throw,catch,CatchError, register_catch};

   // Specificy the exception payload, this can be anytype. It can be reterived later with `Exception::payload()`
   type Payload = &'static str;

   // Create our custom exception and implement `Exception` trait on it
   #[derive(Debug)]
   struct MyE {}

   impl Exception<Payload> for MyE {
       fn payload(&self) -> Payload {
           "MyE exception"
       }
   }

   // This method removes panic error messages from exceptions while the guard is alive.
   // The payload of the exception needs to be specified as a type argument.
   let _g = register_catch::<Payload>();

   // Our example of a call stack.
   fn nested() {
       fn call() {
           fn stack() {
               // throw our exception here
               throw(MyE {});
           }
           stack();
       }
       call();
   }

   // Run our normal callstack inside a `catch` call.
   // `catch` needs to know the payload type.
   // The result is `CatchError` which is either an exception or a normal panic
   let result = catch! {nested() => <Payload>};

   if let Err(CatchError::Exception(e)) = result {
       assert_eq!(e.payload(), MyE {}.payload());
   } else {
       panic!("test failed");
   }
```

