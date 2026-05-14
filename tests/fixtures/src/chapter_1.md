# Chapter 1

This chapter demonstrates the function_body preprocessor:

```rust
#![function_body!("test_file_book.rs", hello_world)]
```

```rust
#![struct!("test_file_book.rs", Testing)]
```

```rust
#![impl_method!("test_file_book.rs", TestStruct::new)]
```

```rust
#![static!("test_file_book.rs", TEST_STATIC)]
```

```rust
#![const!("test_file_book.rs", TEST_CONST)]
```

```rust
#![trait!("test_file_book.rs", X)]
```


```rust
#![enum!("<rustc>/lib/rustlib/src/rust/library/proc_macro/src/lib.rs", TokenTree)]
```

```rust
#![function!("<crateio>/clap_derive-4.5.32/src/lib.rs", to_compile_error)]
```

```rust
#![struct!("<github>/num_enum-9def65cfcc1e6dea/2e53bd3/num_enum/src/lib.rs", TryFromPrimitiveError)]
```
