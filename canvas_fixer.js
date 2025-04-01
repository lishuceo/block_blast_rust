/**
 * canvas_fixer.js - 简化稳健版本 v3.1
 * 解决黑屏、坐标和显示比例问题
 */

// 全局变量
let canvasElement = null;
let gameWrapper = null;
let gameContainer = null;
let debugMode = true; // 调试模式
let isLandscapeMode = false; // 当前是否为横屏模式
let originalWidth = 400; // 原始逻辑宽度
let originalHeight = 600; // 原始逻辑高度
let lastFrameTime = 0; // 上一帧时间，用于节流
let initializationComplete = false; // 初始化是否完成
let webglInitialized = false; // WebGL是否初始化
let wasmReloadAttempted = false; // 是否已尝试过重载WASM
let lastUserInteraction = 0; // 上次用户交互时间

// 存储实际显示尺寸和缩放信息
let displayInfo = {
    scaleRatio: 1.0,    // 当前缩放比例
    displayWidth: 0,     // 实际显示宽度
    displayHeight: 0,    // 实际显示高度
    offsetX: 0,          // 水平偏移量
    offsetY: 0           // 垂直偏移量
};

// 调试日志函数
function debugLog(...args) {
    if (debugMode) {
        console.log('[CANVAS-FIX]', ...args);
    }
}

// 初始化函数 - 更安全的版本
function initCanvasFixer() {
    try {
        debugLog('初始化Canvas修复程序...');
        
        // 获取DOM元素 - 添加空值检查
        canvasElement = document.getElementById('glcanvas');
        gameWrapper = document.querySelector('.game-wrapper');
        gameContainer = document.querySelector('.game-container');
        
        if (!canvasElement) {
            debugLog('警告: canvas元素不存在，将等待DOM加载');
            return false;
        }
        
        if (!gameWrapper || !gameContainer) {
            debugLog('警告: 游戏容器元素不存在，将等待DOM加载');
            return false;
        }
        
        // 监控WebGL上下文初始化
        monitorWebGLInitialization();
        
        // 设置事件监听器 - 只处理坐标转换，不修改样式
        setupEventListeners();
        
        // 设置canvas尺寸
        updateCanvasSize();
        
        // 监听方向切换按钮
        const orientationToggle = document.getElementById('orientationToggle');
        if (orientationToggle) {
            orientationToggle.addEventListener('click', function() {
                debugLog('方向切换按钮点击');
                setTimeout(function() {
                    isLandscapeMode = gameWrapper.classList.contains('landscape');
                    updateCanvasSize();
                    tryForceRedraw();
                }, 100);
            });
        }
        
        // 监听窗口尺寸变化
        window.addEventListener('resize', function() {
            debugLog('窗口尺寸变化');
            updateCanvasSize();
            tryForceRedraw();
        });
        
        // 初始化完成
        initializationComplete = true;
        debugLog('Canvas修复程序初始化完成');
        return true;
    } catch (error) {
        debugLog('初始化过程中出错:', error);
        return false;
    }
}

// 监控WebGL上下文初始化
function monitorWebGLInitialization() {
    if (!canvasElement) return;
    
    // 尝试获取WebGL上下文
    const checkWebGLContext = function() {
        // 获取所有可能的WebGL上下文
        const contexts = ['webgl', 'experimental-webgl', 'webgl2', 'experimental-webgl2'];
        
        // 用于存储找到的上下文
        let gl = null;
        
        for (const contextType of contexts) {
            try {
                gl = canvasElement.getContext(contextType, { alpha: true, antialias: true });
                if (gl) {
                    debugLog(`WebGL上下文初始化: ${contextType}`);
                    break;
                }
            } catch (e) {
                debugLog(`获取${contextType}上下文出错:`, e);
            }
        }
        
        if (gl) {
            webglInitialized = true;
            debugLog('WebGL上下文已初始化');
            
            // WebGL已初始化，尝试强制重绘
            tryForceRedraw();
        } else {
            // 如果未找到WebGL上下文，稍后再试
            setTimeout(checkWebGLContext, 500);
        }
    };
    
    // 开始监控WebGL初始化
    checkWebGLContext();
}

// 尝试强制画面刷新
function tryForceRedraw() {
    if (!canvasElement) return;
    
    const currentTime = Date.now();
    const isUserInteracting = (currentTime - lastUserInteraction) < 2000; // 最近2秒内有用户交互
    
    try {
        debugLog('尝试强制重绘画面');
        
        // 触发强制重绘的方法1：CSS改变
        canvasElement.style.opacity = '0.99';
        setTimeout(() => {
            canvasElement.style.opacity = '1.0';
        }, 10);
        
        // 触发强制重绘的方法2：类添加和移除
        gameContainer.classList.add('force-redraw');
        setTimeout(() => {
            gameContainer.classList.remove('force-redraw');
        }, 20);
        
        // 触发强制重绘的方法3：布局刷新
        void canvasElement.offsetHeight;
        void gameWrapper.offsetHeight;
        
        // 只有在以下情况下才尝试重新加载WASM：
        // 1. 尚未尝试过重载WASM
        // 2. 当前不是由用户交互触发的重绘
        // 3. 页面加载后的首次渲染或者WebGL刚初始化
        if (window.load && !wasmReloadAttempted && !isUserInteracting) {
            // 设置标志以避免多次尝试
            wasmReloadAttempted = true;
            
            // 延迟一点时间再触发重新加载，避免资源尚未准备好
            setTimeout(() => {
                try {
                    const wasmUrl = 'block_blast_bin.wasm';
                    debugLog(`尝试重新加载WASM: ${wasmUrl}`);
                    window.load(wasmUrl);
                } catch (e) {
                    debugLog('WASM重新加载尝试失败:', e);
                }
            }, 300);
        }
    } catch (e) {
        debugLog('强制重绘尝试失败:', e);
    }
}

// 安全地更新Canvas尺寸和样式
function updateCanvasSize() {
    try {
        if (!canvasElement || !gameWrapper || !gameContainer) {
            debugLog('更新尺寸失败: 元素不存在');
            return;
        }
        
        // 获取当前窗口尺寸
        const windowWidth = window.innerWidth || document.documentElement.clientWidth || document.body.clientWidth;
        const windowHeight = window.innerHeight || document.documentElement.clientHeight || document.body.clientHeight;
        
        debugLog(`窗口尺寸: ${windowWidth}x${windowHeight}`);
        
        // 检查当前是否为横屏模式
        isLandscapeMode = gameWrapper.classList.contains('landscape');
        debugLog(`当前模式: ${isLandscapeMode ? '横屏' : '竖屏'}`);
        
        // 计算理想显示尺寸和缩放比例
        calculateDisplaySize(windowWidth, windowHeight);
        
        // 应用样式
        applyMinimalStyleChanges();
        
        // 尝试强制重绘
        if (webglInitialized) {
            tryForceRedraw();
        }
        
        debugLog(`显示信息:
        - 缩放比例: ${displayInfo.scaleRatio.toFixed(3)}
        - 显示尺寸: ${displayInfo.displayWidth}x${displayInfo.displayHeight}
        - 偏移量: X=${displayInfo.offsetX}, Y=${displayInfo.offsetY}`);
    } catch (error) {
        debugLog('更新尺寸过程中出错:', error);
    }
}

// 计算理想显示尺寸和缩放比例 - 简化版本
function calculateDisplaySize(windowWidth, windowHeight) {
    // 计算安全边距 - 对于小窗口减小边距，为游戏内容预留更多空间
    const minDimension = Math.min(windowWidth, windowHeight);
    const safetyMarginPercent = minDimension < 500 ? 0.01 : 0.02; // 小窗口时使用1%的边距
    const safetyMargin = minDimension * safetyMarginPercent;
    
    const maxWidth = windowWidth - safetyMargin * 2; // 两侧各留出边距
    const maxHeight = windowHeight - safetyMargin * 2; // 上下各留出边距
    
    debugLog(`屏幕区域: ${maxWidth}x${maxHeight}, 边距: ${safetyMargin}px`);
    
    let scaleX, scaleY, scaleRatio, displayWidth, displayHeight;
    
    // 计算游戏宽高比和屏幕宽高比
    const gameRatio = originalWidth / originalHeight;  // 游戏原始宽高比
    const screenRatio = maxWidth / maxHeight;          // 屏幕宽高比
    
    debugLog(`游戏比例: ${gameRatio.toFixed(3)}, 屏幕比例: ${screenRatio.toFixed(3)}`);
    
    if (isLandscapeMode) {
        // 横屏模式处理 - 交换宽高计算
        scaleX = maxHeight / originalWidth; // 宽度映射到高度
        scaleY = maxWidth / originalHeight; // 高度映射到宽度
        
        // 小窗口时特殊处理: 总是取较小值确保完全可见
        scaleRatio = Math.min(scaleX, scaleY);
        
        // 小窗口时额外缩小一点，确保不会超出边界
        if (minDimension < 500) {
            scaleRatio *= 0.98;
        }
        
        debugLog(`横屏模式: 使用缩放比例 ${scaleRatio.toFixed(3)}`);
        
        // 计算交换后的尺寸
        displayWidth = Math.floor(originalHeight * scaleRatio);
        displayHeight = Math.floor(originalWidth * scaleRatio);
    } else {
        // 竖屏模式处理
        scaleX = maxWidth / originalWidth;   // 按宽度缩放
        scaleY = maxHeight / originalHeight; // 按高度缩放
        
        // 关键改进: 在小窗口(高度<游戏高度)时必须使用高度比例
        if (windowHeight < originalHeight) {
            // 在小窗口中，优先确保游戏完全可见
            scaleRatio = scaleY * 0.98; // 额外留出一点空间
            debugLog(`小窗口处理: 强制使用高度比例缩放, 比例=${scaleRatio.toFixed(3)}`);
        } else if (gameRatio < 1.0) {
            // 正常窗口中的竖屏游戏 - 优先填满高度
            scaleRatio = scaleY;
            debugLog(`竖屏游戏: 优先填满高度，比例=${scaleRatio.toFixed(3)}`);
        } else {
            // 宽屏游戏 - 优先填满宽度
            scaleRatio = scaleX;
            debugLog(`宽屏游戏: 优先填满宽度，比例=${scaleRatio.toFixed(3)}`);
        }
        
        // 应用缩放比例
        displayWidth = Math.floor(originalWidth * scaleRatio);
        displayHeight = Math.floor(originalHeight * scaleRatio);
        
        // 最终的安全检查：确保不会超出实际屏幕
        if (displayWidth > maxWidth) {
            const adjustment = maxWidth / displayWidth;
            scaleRatio *= adjustment;
            displayWidth = Math.floor(originalWidth * scaleRatio);
            displayHeight = Math.floor(originalHeight * scaleRatio);
            debugLog(`调整: 宽度超出，缩放调整=${adjustment.toFixed(3)}`);
        }
        
        if (displayHeight > maxHeight) {
            const adjustment = maxHeight / displayHeight;
            scaleRatio *= adjustment;
            displayWidth = Math.floor(originalWidth * scaleRatio);
            displayHeight = Math.floor(originalHeight * scaleRatio);
            debugLog(`调整: 高度超出，缩放调整=${adjustment.toFixed(3)}`);
        }
    }
    
    // 确保尺寸至少为1像素
    displayWidth = Math.max(1, displayWidth);
    displayHeight = Math.max(1, displayHeight);
    
    // 计算居中偏移量
    const offsetX = Math.floor((windowWidth - displayWidth) / 2);
    const offsetY = Math.floor((windowHeight - displayHeight) / 2);
    
    debugLog(`最终显示尺寸: ${displayWidth}x${displayHeight}, 居中偏移: (${offsetX}, ${offsetY})`);
    
    // 更新显示信息
    displayInfo.scaleRatio = scaleRatio;
    displayInfo.displayWidth = displayWidth;
    displayInfo.displayHeight = displayHeight;
    displayInfo.offsetX = offsetX;
    displayInfo.offsetY = offsetY;
}

// 应用最小化的样式变更 - 避免重置所有样式
function applyMinimalStyleChanges() {
    if (!gameWrapper || !gameContainer || !canvasElement) return;
    
    debugLog('应用样式变更 - 强制覆盖模式');
    
    // 清除可能干扰的CSS内联样式
    gameWrapper.style.cssText = "";
    
    // 设置绝对关键样式，确保完全覆盖任何CSS规则
    gameWrapper.style.position = 'absolute';
    gameWrapper.style.width = `${displayInfo.displayWidth}px`;
    gameWrapper.style.height = `${displayInfo.displayHeight}px`;
    gameWrapper.style.left = `${displayInfo.offsetX}px`;
    gameWrapper.style.top = `${displayInfo.offsetY}px`;
    gameWrapper.style.margin = '0';
    gameWrapper.style.padding = '0';
    gameWrapper.style.boxSizing = 'border-box';
    gameWrapper.style.backgroundColor = '#000'; // 设置背景色以防止透明问题
    gameWrapper.style.zIndex = '1'; // 确保在页面堆叠顺序中有合理位置
    
    // 确保游戏容器填满wrapper
    gameContainer.style.width = '100%';
    gameContainer.style.height = '100%';
    gameContainer.style.overflow = 'hidden';
    gameContainer.style.position = 'relative';
    
    // 根据当前方向设置旋转变换
    if (isLandscapeMode) {
        gameWrapper.style.transform = 'rotate(-90deg)';
    } else {
        gameWrapper.style.transform = 'none';
    }
    
    // 设置变换原点
    gameWrapper.style.transformOrigin = 'center center';
    
    // 确保canvas元素填满容器
    canvasElement.style.width = '100%';
    canvasElement.style.height = '100%';
    canvasElement.style.display = 'block'; // 防止行内元素问题
    canvasElement.style.position = 'absolute';
    canvasElement.style.top = '0';
    canvasElement.style.left = '0';
    
    // 禁用任何动画或过渡，确保立即应用
    gameWrapper.style.transition = 'none';
    
    // 处理方向切换按钮的位置，确保它不会遮挡游戏，特别是在小窗口中
    const orientationToggle = document.getElementById('orientationToggle');
    if (orientationToggle) {
        const isSmallWindow = window.innerHeight < originalHeight || window.innerWidth < originalWidth;
        
        // 在小窗口中把按钮移到右上角，否则保持在右下角
        if (isSmallWindow) {
            orientationToggle.style.bottom = 'auto';
            orientationToggle.style.top = '10px';
            orientationToggle.style.right = '10px';
        } else {
            orientationToggle.style.top = 'auto';
            orientationToggle.style.bottom = '10px';
            orientationToggle.style.right = '10px';
        }
        
        // 确保按钮在所有元素之上
        orientationToggle.style.zIndex = '1000';
    }
    
    debugLog(`强制应用尺寸: ${displayInfo.displayWidth}x${displayInfo.displayHeight}, 位置: (${displayInfo.offsetX}, ${displayInfo.offsetY})`);
    
    // 强制浏览器重新计算布局
    void gameWrapper.offsetWidth;
}

// 设置事件监听器
function setupEventListeners() {
    if (!canvasElement) return;
    
    debugLog('设置事件监听器');
    
    // 添加事件监听器 - 只处理转换，不拦截原始事件
    canvasElement.addEventListener('mousedown', handleMouseEvent);
    canvasElement.addEventListener('mouseup', handleMouseEvent);
    canvasElement.addEventListener('mousemove', function(e) {
        const now = Date.now();
        if (now - lastFrameTime >= 16) { // 限制到约60fps
            lastFrameTime = now;
            handleMouseEvent(e);
        }
    });
    
    canvasElement.addEventListener('touchstart', handleTouchEvent, { passive: false });
    canvasElement.addEventListener('touchend', handleTouchEvent, { passive: false });
    canvasElement.addEventListener('touchmove', function(e) {
        const now = Date.now();
        if (now - lastFrameTime >= 16) {
            lastFrameTime = now;
            handleTouchEvent(e);
        }
    }, { passive: false });
    
    // 添加可视性变化监听，当页面从隐藏变为可见时尝试重绘
    document.addEventListener('visibilitychange', function() {
        if (document.visibilityState === 'visible') {
            debugLog('页面变为可见，尝试重绘');
            updateCanvasSize();
            tryForceRedraw();
        }
    });
    
    debugLog('事件监听器设置完成');
}

// 处理鼠标事件
function handleMouseEvent(e) {
    if (!canvasElement) return;
    
    // 记录最后交互时间
    lastUserInteraction = Date.now();
    
    // 获取鼠标相对于canvas的位置
    const rect = canvasElement.getBoundingClientRect();
    const canvasX = e.clientX - rect.left;
    const canvasY = e.clientY - rect.top;
    
    // 转换为游戏逻辑坐标
    const gameCoords = convertToGameCoordinates(canvasX, canvasY);
    
    if (e.type === 'mousedown' || e.type === 'mouseup') {
        debugLog(`${e.type}:
            客户端坐标: (${e.clientX}, ${e.clientY})
            Canvas相对坐标: (${canvasX.toFixed(1)}, ${canvasY.toFixed(1)})
            游戏逻辑坐标: (${gameCoords.x}, ${gameCoords.y})`);
    }
}

// 处理触摸事件
function handleTouchEvent(e) {
    if (!canvasElement) return;
    
    // 记录最后交互时间
    lastUserInteraction = Date.now();
    
    // 阻止滚动，但允许事件传播
    if (e.type === 'touchmove') {
        e.preventDefault();
    }
    
    // 确保有触摸点
    if (e.touches.length === 0 && e.changedTouches.length === 0) return;
    
    // 获取第一个触摸点
    const touch = e.type === 'touchend' ? e.changedTouches[0] : e.touches[0];
    
    // 获取触摸点相对于canvas的位置
    const rect = canvasElement.getBoundingClientRect();
    const canvasX = touch.clientX - rect.left;
    const canvasY = touch.clientY - rect.top;
    
    // 转换为游戏逻辑坐标
    const gameCoords = convertToGameCoordinates(canvasX, canvasY);
    
    if (e.type === 'touchstart' || e.type === 'touchend') {
        debugLog(`${e.type}:
            客户端坐标: (${touch.clientX}, ${touch.clientY})
            Canvas相对坐标: (${canvasX.toFixed(1)}, ${canvasY.toFixed(1)})
            游戏逻辑坐标: (${gameCoords.x}, ${gameCoords.y})`);
    }
}

// 将canvas上的坐标转换为游戏逻辑坐标
function convertToGameCoordinates(canvasX, canvasY) {
    if (!canvasElement) return {x: 0, y: 0};
    
    // 获取实际canvas尺寸
    const rect = canvasElement.getBoundingClientRect();
    const canvasWidth = rect.width;
    const canvasHeight = rect.height;
    
    // 计算相对比例位置
    const ratioX = canvasX / canvasWidth;
    const ratioY = canvasY / canvasHeight;
    
    let gameX, gameY;
    
    if (isLandscapeMode) {
        // 横屏模式 - 坐标系统旋转90度
        // y轴映射到x轴，x轴反向映射到y轴
        gameX = Math.round(ratioY * originalWidth);
        gameY = Math.round((1 - ratioX) * originalHeight);
    } else {
        // 竖屏模式 - 直接映射
        gameX = Math.round(ratioX * originalWidth);
        gameY = Math.round(ratioY * originalHeight);
    }
    
    // 确保坐标在有效范围内
    gameX = Math.max(0, Math.min(gameX, originalWidth));
    gameY = Math.max(0, Math.min(gameY, originalHeight));
    
    return { x: gameX, y: gameY };
}

// 导出公共方法 - 更安全的版本
window.forceCanvasSize = function() {
    try {
        if (canvasElement && gameWrapper) {
            updateCanvasSize();
            tryForceRedraw();
            debugLog('强制调整Canvas尺寸');
        } else {
            debugLog('无法强制调整尺寸: 元素不存在');
            
            // 如果元素不存在，尝试延迟重试
            setTimeout(function() {
                canvasElement = document.getElementById('glcanvas');
                gameWrapper = document.querySelector('.game-wrapper');
                gameContainer = document.querySelector('.game-container');
                
                if (canvasElement && gameWrapper && gameContainer) {
                    initCanvasFixer();
                    updateCanvasSize();
                    tryForceRedraw();
                    debugLog('延迟初始化成功');
                }
            }, 500);
        }
    } catch (error) {
        debugLog('强制调整尺寸过程中出错:', error);
    }
};

// 稳健的初始化策略
function safeInitialize() {
    try {
        debugLog('开始安全初始化过程');
        
        // 等待DOM准备好
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', function() {
                // DOM已加载，但可能还有资源未加载完成
                setTimeout(function() {
                    if (!initializationComplete) {
                        initCanvasFixer();
                    }
                }, 200);
            });
        } else {
            // DOM已经加载完成，直接初始化
            setTimeout(function() {
                if (!initializationComplete) {
                    initCanvasFixer();
                }
            }, 100);
        }
        
        // 确保页面完全加载后再次尝试初始化
        window.addEventListener('load', function() {
            debugLog('页面完全加载');
            
            // 页面完全加载后，再次尝试初始化（如果还没有完成）
            if (!initializationComplete) {
                setTimeout(function() {
                    initCanvasFixer();
                    
                    // 额外的延迟更新，确保WebGL已准备好
                    setTimeout(updateCanvasSize, 300);
                    setTimeout(function() {
                        updateCanvasSize();
                        tryForceRedraw();
                    }, 800);
                    setTimeout(function() {
                        updateCanvasSize();
                        tryForceRedraw();
                    }, 1500);
                }, 200);
            } else {
                // 即使初始化已完成，也强制更新尺寸几次
                setTimeout(function() {
                    updateCanvasSize();
                    tryForceRedraw();
                }, 300);
                setTimeout(function() {
                    updateCanvasSize();
                    tryForceRedraw();
                }, 800);
                setTimeout(function() {
                    updateCanvasSize();
                    tryForceRedraw();
                }, 1500);
            }
        });
        
        // 添加额外的强制更新逻辑
        for (let delay of [100, 500, 1000, 2000, 3000, 5000]) {
            setTimeout(function() {
                if (initializationComplete) {
                    debugLog(`强制更新尺寸 (${delay}ms)`);
                    updateCanvasSize();
                    tryForceRedraw();
                } else {
                    // 如果初始化尚未完成，尝试再次初始化
                    initCanvasFixer();
                }
            }, delay);
        }
        
        // 额外的最终保障 - 确保在页面完全加载后进行三次尝试
        const finalRetries = [8000, 12000, 20000];
        for (let delay of finalRetries) {
            setTimeout(function() {
                debugLog(`最终保障检查 (${delay}ms)`);
                if (!webglInitialized) {
                    debugLog('WebGL尚未初始化，尝试强制重绘');
                    initCanvasFixer();
                    updateCanvasSize();
                    tryForceRedraw();
                }
            }, delay);
        }
    } catch (error) {
        debugLog('安全初始化过程中出错:', error);
    }
}

// 开始安全初始化过程
safeInitialize();