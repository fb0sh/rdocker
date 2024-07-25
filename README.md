# a api for docker in rust
```rust
let dr =  Docker::new().unwrap();
let p = dr.head("/_ping");
let p = dr.get("/version");
println!("{}", p.status_code());
println!("{}", p.headers);
println!("{}", p.data);
```