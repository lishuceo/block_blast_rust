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
        
        // 存储最近的错误和日志消息
        const recentMessages = [];
        const MAX_MESSAGES = 5; // 最多显示5条消息
        let isDragging = false; // 拖动状态标志
        let dragOffset = { x: 0, y: 0 }; // 拖动偏移量
        
        // 添加用于在屏幕上显示错误的辅助函数
        function displayErrorOnScreen(message, logLevel = 3) {
            // 确定消息类型和颜色
            let messageType, baseColor, darkColor;
            switch(logLevel) {
                case 0: // Debug
                    messageType = "调试";
                    baseColor = "#55aaff"; // 蓝色
                    darkColor = "#3377cc";
                    break;
                case 1: // Info
                    messageType = "信息";
                    baseColor = "#55ff55"; // 绿色
                    darkColor = "#33cc33";
                    break;
                case 2: // Warning
                    messageType = "警告";
                    baseColor = "#ffaa55"; // 橙色
                    darkColor = "#cc7733";
                    break;
                case 3: // Error
                default:
                    messageType = "错误";
                    baseColor = "#ff5555"; // 红色
                    darkColor = "#cc3333";
                    break;
            }
            
            // 将新消息添加到最近消息列表中
            const timestamp = new Date().toLocaleTimeString();
            recentMessages.unshift({
                text: `[${timestamp}] ${message}`, 
                level: logLevel,
                color: baseColor,
                darkColor: darkColor,
                type: messageType
            }); // 添加到数组开头
            
            // 保持最多固定数量的消息
            if (recentMessages.length > MAX_MESSAGES) {
                recentMessages.pop(); // 移除最旧的消息
            }
            
            // 创建或获取消息显示元素
            let messageDisplay = document.getElementById('game-message-display');
            if (!messageDisplay) {
                messageDisplay = document.createElement('div');
                messageDisplay.id = 'game-message-display';
                messageDisplay.style.position = 'fixed';
                messageDisplay.style.top = '20vh';  // 屏幕高度的20%处
                messageDisplay.style.left = '10px';
                messageDisplay.style.backgroundColor = 'rgba(0, 0, 0, 0.85)';
                messageDisplay.style.padding = '10px';
                messageDisplay.style.borderRadius = '5px';
                messageDisplay.style.fontFamily = 'monospace';
                messageDisplay.style.fontSize = '14px';
                messageDisplay.style.maxWidth = '80%';
                messageDisplay.style.maxHeight = '40vh'; // 限制最大高度
                messageDisplay.style.overflow = 'auto'; // 允许滚动
                messageDisplay.style.zIndex = '10000';  // 确保显示在最上层
                messageDisplay.style.boxShadow = '0 0 10px rgba(0, 0, 0, 0.5)';
                messageDisplay.style.cursor = 'move'; // 指示可拖动
                document.body.appendChild(messageDisplay);
                
                // 添加拖动功能
                messageDisplay.addEventListener('mousedown', startDrag);
                messageDisplay.addEventListener('touchstart', startDrag, { passive: false });
            }
            
            // 构建消息 HTML
            let messageHTML = `
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px;">
                    <div style="font-weight: bold; color: ${baseColor};">游戏${messageType}日志</div>
                    <div style="display: flex; gap: 5px;">
                        <button id="clear-logs-btn" style="background: #333; color: white; border: none; border-radius: 3px; padding: 2px 5px; cursor: pointer; font-size: 11px;">清空</button>
                        <button id="close-logs-btn" style="background: #333; color: white; border: none; border-radius: 3px; padding: 2px 5px; cursor: pointer; font-size: 11px;">关闭</button>
                    </div>
                </div>
            `;
            
            // 添加每条消息信息
            recentMessages.forEach((msg, index) => {
                const color = index === 0 ? msg.color : msg.darkColor;
                const typeLabel = `<span style="color: ${color}; font-weight: bold;">[${msg.type}]</span>`;
                messageHTML += `<div style="color: ${color}; margin-bottom: 3px;">• ${typeLabel} ${msg.text}</div>`;
            });
            
            // 设置消息内容
            messageDisplay.innerHTML = messageHTML;
            
            // 添加按钮事件处理
            setTimeout(() => {
                const closeBtn = document.getElementById('close-logs-btn');
                if (closeBtn) {
                    closeBtn.addEventListener('click', function(e) {
                        e.stopPropagation(); // 阻止事件冒泡到拖动处理
                        fadeOutAndHide(messageDisplay);
                    });
                }
                
                const clearBtn = document.getElementById('clear-logs-btn');
                if (clearBtn) {
                    clearBtn.addEventListener('click', function(e) {
                        e.stopPropagation(); // 阻止事件冒泡到拖动处理
                        recentMessages.length = 0;
                        displayErrorOnScreen("日志已清空", 1); // 显示清空确认消息
                    });
                }
            }, 0);
            
            // 显示几秒后自动隐藏
            messageDisplay.style.display = 'block';
            // 添加淡入动画
            messageDisplay.style.animation = 'fadeIn 0.3s';
            messageDisplay.style.opacity = '1';
            
            // 清除之前的定时器（如果有）
            if (messageDisplay.hideTimer) {
                clearTimeout(messageDisplay.hideTimer);
            }
            
            // 设置自动隐藏，错误信息显示更长时间
            const hideTimeout = logLevel >= 3 ? 10000 : 5000; // 错误显示30秒，其他15秒
            messageDisplay.hideTimer = setTimeout(() => {
                fadeOutAndHide(messageDisplay);
            }, hideTimeout);
            
            // 辅助函数：淡出并隐藏元素
            function fadeOutAndHide(element) {
                element.style.animation = 'fadeOut 1s';
                element.style.opacity = '0';
                setTimeout(() => {
                    element.style.display = 'none';
                }, 1000);
            }
            
            // 辅助函数：开始拖动
            function startDrag(e) {
                e.preventDefault();
                
                // 忽略按钮点击引起的拖动
                if (e.target.tagName === 'BUTTON') return;
                
                isDragging = true;
                
                // 获取鼠标/触摸起始位置
                const clientX = e.clientX || (e.touches && e.touches[0].clientX) || 0;
                const clientY = e.clientY || (e.touches && e.touches[0].clientY) || 0;
                
                // 计算偏移量
                const rect = messageDisplay.getBoundingClientRect();
                dragOffset.x = clientX - rect.left;
                dragOffset.y = clientY - rect.top;
                
                // 添加移动和结束拖动事件监听器
                document.addEventListener('mousemove', doDrag);
                document.addEventListener('touchmove', doDrag, { passive: false });
                document.addEventListener('mouseup', stopDrag);
                document.addEventListener('touchend', stopDrag);
                
                // 拖动时添加一些视觉反馈
                messageDisplay.style.boxShadow = '0 0 15px rgba(0, 0, 0, 0.7)';
            }
            
            // 辅助函数：执行拖动
            function doDrag(e) {
                if (!isDragging) return;
                e.preventDefault();
                
                // 获取当前鼠标/触摸位置
                const clientX = e.clientX || (e.touches && e.touches[0].clientX) || 0;
                const clientY = e.clientY || (e.touches && e.touches[0].clientY) || 0;
                
                // 移动元素
                messageDisplay.style.left = (clientX - dragOffset.x) + 'px';
                messageDisplay.style.top = (clientY - dragOffset.y) + 'px';
            }
            
            // 辅助函数：停止拖动
            function stopDrag() {
                isDragging = false;
                document.removeEventListener('mousemove', doDrag);
                document.removeEventListener('touchmove', doDrag);
                document.removeEventListener('mouseup', stopDrag);
                document.removeEventListener('touchend', stopDrag);
                
                // 恢复正常阴影
                messageDisplay.style.boxShadow = '0 0 10px rgba(0, 0, 0, 0.5)';
            }
        }
        
        // 在game_log_js函数中修改调用方式，传递日志级别
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
                    displayErrorOnScreen(text, 2); // 显示警告
                    break;
                case 3: // Error
                    console.error(`[RUST] ${text}`);
                    displayErrorOnScreen(text, 3); // 显示错误
                    break;
                default:
                    console.log(`[RUST] ${text}`);
            }
            
            return 1; // 成功
        };
        
        // 添加CSS动画
        if (!document.getElementById('game-message-display-style')) {
            const style = document.createElement('style');
            style.id = 'game-message-display-style';
            style.textContent = `
                @keyframes fadeIn {
                    from { opacity: 0; transform: translateY(-10px); }
                    to { opacity: 1; transform: translateY(0); }
                }
                @keyframes fadeOut {
                    from { opacity: 1; }
                    to { opacity: 0; }
                }
                /* 防止按钮在拖动时选中文本 */
                #game-message-display {
                    user-select: none;
                    -webkit-user-select: none;
                }
                #game-message-display button:hover {
                    background: #555 !important;
                }
            `;
            document.head.appendChild(style);
        }
        
        // 给window添加一个显示调试信息的全局函数
        window.showGameMessage = function(message, level) {
            level = level || 1; // 默认为info级别
            displayErrorOnScreen(message, level);
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

        // 新增：将 js_trigger_vibration 添加到 env 对象
        importObject.env.js_trigger_vibration = function(duration_ms) {
            if (navigator.vibrate) {
                try {
                    if (typeof duration_ms === 'number' && duration_ms > 0) {
                        navigator.vibrate(duration_ms);
                        // console.log("Vibrating for " + duration_ms + "ms"); // 用于调试
                    } else {
                        // console.warn("Invalid duration for vibration: " + duration_ms);
                    }
                } catch (e) {
                    // console.error("Vibration failed: ", e);
                }
            } else {
                // console.log("Vibration API not supported."); // 用于调试
            }
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

// 在 js_bridge.js 文件末尾添加 (这部分现在会被移到插件内部，所以可以考虑移除或保留作为 window 对象的备用)
/* window.js_trigger_vibration = function(duration_ms) {
    if (navigator.vibrate) {
        try {
            if (typeof duration_ms === 'number' && duration_ms > 0) {
                navigator.vibrate(duration_ms);
                // console.log("Vibrating for " + duration_ms + "ms"); // 用于调试
            } else {
                // console.warn("Invalid duration for vibration: " + duration_ms);
            }
        } catch (e) {
            // console.error("Vibration failed: ", e);
        }
    } else {
        // console.log("Vibration API not supported."); // 用于调试
    }
}; */ 