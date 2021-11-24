# waitforit
`waitforit` is a crate used to aid in simple, synchronous delays on various conditions. It extracts this logic from the console app [`waitfor`](https://github.com/aeshirey/waitfor/). Given some set of conditions, it will simply block until any one of the specified conditions is met.


```rust
let duration = waitforit::parse_duration("10m30s").unwrap();
let waits =
    Wait::elapsed(duration) 
    | Wait::exists("foo.txt") 
    // To check for an existing file to be updated, we need to be able to get metadata for it
    | Wait::update("bar.tmp").unwrap();

// Block until one of the above conditions is met, checking every 2 seconds
waits.wait(Duration::from_secs(2));
```