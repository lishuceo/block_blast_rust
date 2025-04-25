# Macroquad引擎WASM版本与JavaScript交互指南

## Macroquad与JavaScript交互的基本方法

Macroquad使用了一种特定的方式来处理WASM与JavaScript的交互。当使用Macroquad编译为WASM时，其JavaScript加载器会调用WASM导出的"main"函数，并将JS事件转发为WASM函数调用。


## 从JavaScript调用Rust函数

Miniquad（Macroquad的底层库）的JS加载器会提供两个全局变量：
1. "wasm_exports" - 包含所有从Rust导出的函数
2. "wasm_memory" - WASM内存

您可以直接从JavaScript调用任何在Rust中标记为`#[no_mangle] pub extern "C"`的函数：
```javascript
// 调用Rust函数示例
wasm_exports.my_rust_function(1, 2, 3);
```

## 从Rust调用JavaScript函数

要从Rust调用JavaScript函数，需要在加载WASM之前进行初始化工作。所有可供WASM调用的JS函数必须在WASM加载前明确列出。这些JS函数集合在Miniquad术语中被称为"Plugin"。

1. 创建一个plugin.js文件：
```javascript
// 定义要从Rust调用的JS函数
register_plugin = function (importObject) {
  importObject.env.js_function_name = function (param) {
    console.log("从WASM调用的JS函数，参数:", param);
    return 42; // 返回值
  }
}

// 添加插件
miniquad_add_plugin({register_plugin});
```

2. 在Rust代码中声明外部函数：
```rust
extern "C" {
    fn js_function_name(param: i32) -> i32;
}

// 在Rust中使用
fn call_js() {
    let result = unsafe { js_function_name(123) };
    println!("JS函数返回: {}", result);
}
```

## 交互注意事项

在WASM和JS之间传递数据时，只有有限的类型可用：
- f32/f64
- i8/u8
- i32/u32（不支持i64/u64）
- 指针

对于复杂数据结构，通常需要通过内存操作或JSON序列化来传递。

## 示例：传递字符串

1. 从Rust传递字符串到JS：
```rust
#[no_mangle]
pub extern "C" fn send_string_to_js(ptr: *const u8, len: usize) {
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let string = std::str::from_utf8(slice).unwrap();
    // 处理字符串...
}
```

2. 从JS调用并传递字符串：
```javascript
function sendStringToRust(str) {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);
    
    // 分配内存
    const ptr = wasm_exports.allocate_memory(bytes.length);
    
    // 复制字符串到WASM内存
    const memory = new Uint8Array(wasm_memory.buffer);
    for (let i = 0; i < bytes.length; i++) {
        memory[ptr + i] = bytes[i];
    }
    
    // 调用Rust函数
    wasm_exports.send_string_to_js(ptr, bytes.length);
    
    // 释放内存
    wasm_exports.deallocate_memory(ptr, bytes.length);
}
```

通过这些方法，您可以实现Macroquad WASM版本与JavaScript之间的双向通信。需要注意的是，Macroquad使用自定义的WASM构建过程，这与一些标准的Rust WASM工具（如wasm-bindgen）有所不同。