// js_bridge.js - 用于在Macroquad WASM与JavaScript之间建立通信桥梁

// 用于存储JavaScript调用的结果
let last_js_result = "";

// 定义我们的插件对象
const js_bridge_plugin = {
    register_plugin: function(importObject) {
        console.log("注册JS桥接函数的 env 对象");
        // 确保env对象存在
        importObject.env = importObject.env || {};
        
        // 执行JavaScript代码并返回结果
        importObject.env.js_invoke_string = function(js_code_ptr, js_code_len) {
            try {
                // 从WASM内存中获取JavaScript代码
                const mem_array = new Uint8Array(wasm_memory.buffer);
                const js_code_bytes = mem_array.slice(js_code_ptr, js_code_ptr + js_code_len);
                const js_code = new TextDecoder().decode(js_code_bytes);
                
                console.log("执行JavaScript: ", js_code);
                
                // 执行JavaScript代码
                const result = eval(js_code);
                
                // 存储结果
                if (typeof result === 'string') {
                    last_js_result = result;
                } else if (result === null || result === undefined) {
                    last_js_result = "";
                } else if (typeof result === 'object') {
                    try {
                        last_js_result = JSON.stringify(result);
                    } catch (e) {
                        last_js_result = String(result);
                    }
                } else {
                    last_js_result = String(result);
                }
                
                return 1; // 成功
            } catch (error) {
                console.error("JavaScript执行错误: ", error);
                last_js_result = JSON.stringify({
                    success: false,
                    message: error.toString()
                });
                return 0; // 失败
            }
        };
        
        // 获取结果字符串的指针
        importObject.env.js_get_result_ptr = function() {
            if (!last_js_result) return 0;
            
            const encoder = new TextEncoder();
            const result_bytes = encoder.encode(last_js_result);
            
            // 检查 wasm_exports 是否已定义
            if (typeof wasm_exports === 'undefined' || !wasm_exports.allocate_memory) {
                console.error("WASM导出或allocate_memory未定义");
                return 0;
            }
            
            const ptr = wasm_exports.allocate_memory(result_bytes.length);
            if (ptr === 0) {
                console.error("WASM内存分配失败");
                return 0;
            }
            
            const memory = new Uint8Array(wasm_memory.buffer);
            memory.set(result_bytes, ptr); // 使用更高效的set方法
            
            return ptr;
        };
        
        // 获取结果字符串的长度
        importObject.env.js_get_result_len = function() {
            if (!last_js_result) return 0;
            
            const encoder = new TextEncoder();
            return encoder.encode(last_js_result).length;
        };
    }
};

// 直接尝试注册插件
// 假设此时mq_js_bundle.js已加载并定义了miniquad_add_plugin
if (typeof miniquad_add_plugin === 'function') {
    miniquad_add_plugin(js_bridge_plugin);
    console.log("JS桥接插件已注册");
} else {
    // 增加一个警告，以防万一顺序还是有问题
    console.warn("miniquad_add_plugin 在 js_bridge.js 加载时未定义。这可能导致WASM导入错误。");
    // 尝试延迟注册作为后备方案
    window.addEventListener('load', function() {
        setTimeout(function() {
            if (typeof miniquad_add_plugin === 'function') {
                miniquad_add_plugin(js_bridge_plugin);
                console.log("JS桥接插件 (延迟) 已注册");
            } else {
                console.error("延迟尝试后 miniquad_add_plugin 仍然未定义!");
            }
        }, 100);
    });
}

// 移除旧的内存函数设置逻辑，因为Rust端会导出它们 