/**
 * canvas_fixer.js - 简化稳健版本 v3.0
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
                }, 100);
            });
        }
        
        // 监听窗口尺寸变化
        window.addEventListener('resize', function() {
            debugLog('窗口尺寸变化');
            updateCanvasSize();
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
        
        // 不直接修改DOM元素的样式，只更新我们的计算值
        applyMinimalStyleChanges();
        
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
    // 安全边距，确保不会超出屏幕
    const safetyMargin = Math.min(windowWidth, windowHeight) * 0.05; // 5%的边距
    const maxWidth = windowWidth - safetyMargin;
    const maxHeight = windowHeight - safetyMargin;
    
    let scaleX, scaleY;
    
    if (isLandscapeMode) {
        // 横屏模式 - 需要交换宽高计算
        scaleX = maxHeight / originalWidth;
        scaleY = maxWidth / originalHeight;
    } else {
        // 竖屏模式 - 正常计算
        scaleX = maxWidth / originalWidth;
        scaleY = maxHeight / originalHeight;
    }
    
    // 取最小缩放比例，确保保持原始宽高比
    const scaleRatio = Math.min(scaleX, scaleY) * 0.95; // 略微降低缩放比例以增加安全边距
    
    debugLog(`缩放计算: scaleX=${scaleX.toFixed(3)}, scaleY=${scaleY.toFixed(3)}, 最终比例=${scaleRatio.toFixed(3)}`);
    
    // 计算实际显示尺寸
    let displayWidth, displayHeight;
    
    if (isLandscapeMode) {
        // 横屏模式 - 交换宽高并应用缩放
        displayWidth = Math.floor(originalHeight * scaleRatio);
        displayHeight = Math.floor(originalWidth * scaleRatio);
    } else {
        // 竖屏模式 - 直接应用缩放
        displayWidth = Math.floor(originalWidth * scaleRatio);
        displayHeight = Math.floor(originalHeight * scaleRatio);
    }
    
    // 计算居中偏移量
    const offsetX = Math.floor((windowWidth - displayWidth) / 2);
    const offsetY = Math.floor((windowHeight - displayHeight) / 2);
    
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
    
    // 保存原始的transform值
    const originalTransform = gameWrapper.style.transform;
    
    // 仅设置尺寸和位置相关的样式
    gameWrapper.style.width = `${displayInfo.displayWidth}px`;
    gameWrapper.style.height = `${displayInfo.displayHeight}px`;
    gameWrapper.style.position = 'absolute';
    gameWrapper.style.left = `${displayInfo.offsetX}px`;
    gameWrapper.style.top = `${displayInfo.offsetY}px`;
    
    // 恢复原始的transform属性(如果有方向旋转)
    if (isLandscapeMode && !originalTransform.includes('rotate')) {
        gameWrapper.style.transform = 'rotate(-90deg)';
    }
    
    debugLog(`应用尺寸: ${displayInfo.displayWidth}x${displayInfo.displayHeight}, 位置: (${displayInfo.offsetX}, ${displayInfo.offsetY})`);
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
    
    debugLog('事件监听器设置完成');
}

// 处理鼠标事件
function handleMouseEvent(e) {
    if (!canvasElement) return;
    
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
            debugLog('强制调整Canvas尺寸');
        } else {
            debugLog('无法强制调整尺寸: 元素不存在');
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
                }, 500);
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
                    setTimeout(updateCanvasSize, 500);
                    setTimeout(updateCanvasSize, 1000);
                }, 300);
            }
        });
    } catch (error) {
        debugLog('安全初始化过程中出错:', error);
    }
}

// 开始安全初始化过程
safeInitialize();