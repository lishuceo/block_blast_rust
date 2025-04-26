// js_bridge.js - 用于在Macroquad WASM与JavaScript之间建立通信桥梁

// 用于存储JavaScript调用的结果
let last_js_result = "";

// 辅助函数：检查WASM环境是否就绪
function isWasmReady() {
    if (typeof wasm_memory === 'undefined' || !wasm_memory.buffer) {
        console.error("WASM 内存 (wasm_memory) 未就绪");
        return false;
    }
    if (typeof wasm_exports === 'undefined') {
        console.error("WASM 导出 (wasm_exports) 未就绪");
        return false;
    }
    return true;
}

// 辅助函数：检查内存分配器是否就绪
function isAllocatorReady() {
     if (!isWasmReady() || !wasm_exports.allocate_memory) {
        console.error("WASM 内存分配器 (wasm_exports.allocate_memory) 未就绪");
        return false;
     }
     return true;
}

// 定义我们的插件对象
const js_bridge_plugin = {
    register_plugin: function(importObject) {
        console.log("注册JS桥接函数的 env 对象");
        // 确保env对象存在
        importObject.env = importObject.env || {};
        
        // -- 用于同步调用的函数 --
        importObject.env.js_invoke_string = function(js_code_ptr, js_code_len) {
            if (!isWasmReady()) return 0; // 检查WASM是否就绪
            try {
                // 从WASM内存中获取JavaScript代码
                const mem_array = new Uint8Array(wasm_memory.buffer);
                const js_code_bytes = mem_array.slice(js_code_ptr, js_code_ptr + js_code_len);
                const js_code = new TextDecoder().decode(js_code_bytes);
                
                console.log("执行同步JavaScript: ", js_code);
                
                // 执行JavaScript代码
                const result = eval(js_code);
                
                console.log("执行同步JavaScript结果 result: ", typeof result, result);

                // 存储结果 (仅处理同步结果)
                if (result && typeof result.then === 'function') {
                     console.error("js_invoke_string 不应用于异步函数! 请使用 js_invoke_async_string。");
                     last_js_result = JSON.stringify({ success: false, message: "同步调用不支持异步函数" });
                     return 0; // 错误
                } else if (typeof result === 'string') {
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
                console.error("同步JavaScript执行错误: ", error);
                last_js_result = JSON.stringify({
                    success: false,
                    message: error.toString()
                });
                return 0; // 失败
            }
        };
        
        // -- 用于异步调用的新函数 --
        importObject.env.js_invoke_async_string = function(js_code_ptr, js_code_len) {
            if (!isWasmReady()) return 0; // 检查WASM是否就绪
            try {
                // 从WASM内存中获取JavaScript代码
                const mem_array = new Uint8Array(wasm_memory.buffer);
                const js_code_bytes = mem_array.slice(js_code_ptr, js_code_ptr + js_code_len);
                const js_code = new TextDecoder().decode(js_code_bytes);
                
                console.log("启动异步JavaScript: ", js_code);
                last_js_result = ""; // 清空结果，表示正在运行

                // 执行JavaScript代码，期望返回Promise
                const promise = eval(js_code);

                if (promise && typeof promise.then === 'function') {
                    // 是Promise，设置回调
                    promise.then(resolvedResult => {
                        console.log("异步JavaScript成功: ", resolvedResult);
                        // 存储结果
                        if (typeof resolvedResult === 'string') {
                            last_js_result = resolvedResult;
                        } else if (resolvedResult === null || resolvedResult === undefined) {
                            last_js_result = "";
                        } else if (typeof resolvedResult === 'object') {
                            try {
                                last_js_result = JSON.stringify(resolvedResult);
                            } catch (e) {
                                last_js_result = String(resolvedResult);
                            }
                        } else {
                            last_js_result = String(resolvedResult);
                        }
                    }).catch(error => {
                        console.error("异步JavaScript执行错误: ", error);
                        last_js_result = JSON.stringify({
                            success: false,
                            message: error.toString()
                        });
                    });
                    return 1; // 表示异步操作已成功启动
                } else {
                    // 不是Promise，这是个错误
                    console.error("js_invoke_async_string 需要一个返回 Promise 的表达式", promise);
                    last_js_result = JSON.stringify({ success: false, message: "调用的JS代码未返回Promise" });
                    return 0; // 错误
                }
            } catch (sync_error) {
                // 同步eval错误
                console.error("启动异步JavaScript时同步错误: ", sync_error);
                last_js_result = JSON.stringify({
                    success: false,
                    message: sync_error.toString()
                });
                return 0; // 失败
            }
        };
        
        // 新增：Rust日志输出到控制台的函数 (已重命名)
        importObject.env.game_log_js = function(text_ptr, text_len, log_level) {
            if (!isWasmReady()) return 0; // 检查WASM是否就绪
            const mem_array = new Uint8Array(wasm_memory.buffer);
            const text_bytes = mem_array.slice(text_ptr, text_ptr + text_len);
            const text = new TextDecoder().decode(text_bytes);
            
            // 根据日志级别选择不同的控制台方法
            switch(log_level) {
                case 0: // Debug
                    console.debug(`[RUST] ${text}`);
                    break;
                case 1: // Info
                    console.info(`[RUST] ${text}`);
                    break;
                case 2: // Warning
                    console.warn(`[RUST] ${text}`);
                    break;
                case 3: // Error
                    console.error(`[RUST] ${text}`);
                    break;
                default:
                    console.log(`[RUST] ${text}`);
            }
            
            return 1; // 成功
        };
        
        // 获取结果字符串的指针
        importObject.env.js_get_result_ptr = function() {
            if (!isAllocatorReady()) return 0; // 检查分配器是否就绪
            if (!last_js_result) return 0;
            
            const encoder = new TextEncoder();
            const result_bytes = encoder.encode(last_js_result);

            console.log("last_js_result: ", last_js_result);
            console.log("last_js_result length (bytes):", result_bytes.length);
            
            const ptr = wasm_exports.allocate_memory(result_bytes.length);
            if (ptr === 0) {
                console.error("WASM内存分配失败 (ptr=0)");
                return 0;
            }
            
            const memory = new Uint8Array(wasm_memory.buffer);
            memory.set(result_bytes, ptr); // 使用更高效的set方法
            
            return ptr;
        };
        
        // 获取结果字符串的长度
        importObject.env.js_get_result_len = function() {
             if (!isWasmReady()) return 0; // 检查WASM是否就绪
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