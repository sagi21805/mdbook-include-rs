# Chapter 1

This chapter demonstrates the function_body preprocessor:

```rust
#![function_body!("test_file_book.rs", hello_world)]
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
