fn hello_world() {
    println!("Hello, world!");
}

static TEST_STATIC: usize = 0;

struct TestStruct {
    name: String,
    value: i32,
}

impl TestStruct {
    fn new(name: &str, value: i32) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    fn print(&self) {
        println!("Name: {}, Value: {}", self.name, self.value);
    }
}

enum TestEnum {
    A,
    B(i32),
    C { name: String },
}

trait TestTrait {
    fn test_method(&self) -> String;
    fn default_method(&self) -> i32 {
        42
    }
}

impl TestTrait for TestStruct {
    fn test_method(&self) -> String {
        format!("TestStruct: {}", self.name)
    }
}
