// 音效和特效
use macroquad::prelude::*;

// 单个粒子的定义
pub struct Particle {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,  // 新增：加速度字段
    color: Color,
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
}

impl Particle {
    pub fn new(position: Vec2, velocity: Vec2, acceleration: Vec2, color: Color, size: f32, lifetime: f32) -> Self {
        Particle {
            position,
            velocity,
            acceleration,  // 新增：初始化加速度
            color,
            size,
            lifetime,
            max_lifetime: lifetime,
        }
    }
    
    // 更新粒子状态
    pub fn update(&mut self, dt: f32) -> bool {
        // 先更新速度（根据加速度）
        self.velocity += self.acceleration * dt;
        
        // 再更新位置（根据速度）
        self.position += self.velocity * dt;
        
        // 更新生命周期
        self.lifetime -= dt;
        
        // 返回粒子是否存活
        self.lifetime > 0.0
    }
    
    // 绘制粒子
    pub fn draw(&self) {
        // 随着生命周期减少透明度
        let alpha = self.lifetime / self.max_lifetime;
        let draw_color = Color::new(
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a * alpha
        );
        
        draw_rectangle(
            self.position.x, 
            self.position.y, 
            self.size * alpha, // 随时间缩小
            self.size * alpha,
            draw_color
        );
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem { 
            particles: Vec::new()
        }
    }
    
    // 在指定位置创建消除效果
    pub fn create_clear_effect(&mut self, position: Vec2, color: Color, count: i32) {
        for _ in 0..count {
            // 随机方向
            let angle = macroquad::rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = macroquad::rand::gen_range(100.0, 200.0);
            let velocity = Vec2::new(
                angle.cos() * speed,
                angle.sin() * speed
            );
            
            // 随机大小
            let size = macroquad::rand::gen_range(3.0, 8.0);
            
            // 随机生命周期
            let lifetime = macroquad::rand::gen_range(0.3, 1.0);
            
            // 设置重力加速度（向下）
            let gravity_acceleration = Vec2::new(0.0, 300.0); // 重力效果
            
            // 创建新粒子
            let particle = Particle::new(
                position,
                velocity,
                gravity_acceleration,  // 使用重力加速度
                color,
                size,
                lifetime
            );
            
            self.particles.push(particle);
        }
    }
    
    // 新增：创建带自定义加速度的粒子效果
    pub fn create_effect_with_acceleration(
        &mut self, 
        position: Vec2, 
        color: Color, 
        count: i32,
        speed_range: (f32, f32),  // 速度范围
        acceleration: Vec2,        // 自定义加速度
        size_range: (f32, f32),    // 尺寸范围
        lifetime_range: (f32, f32) // 生命周期范围
    ) {
        for _ in 0..count {
            // 随机方向
            let angle = macroquad::rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = macroquad::rand::gen_range(speed_range.0, speed_range.1);
            let velocity = Vec2::new(
                angle.cos() * speed,
                angle.sin() * speed
            );
            
            // 随机大小
            let size = macroquad::rand::gen_range(size_range.0, size_range.1);
            
            // 随机生命周期
            let lifetime = macroquad::rand::gen_range(lifetime_range.0, lifetime_range.1);
            
            // 创建新粒子
            let particle = Particle::new(
                position,
                velocity,
                acceleration,
                color,
                size,
                lifetime
            );
            
            self.particles.push(particle);
        }
    }
    
    // 更新所有粒子
    pub fn update(&mut self, dt: f32) {
        // 反向迭代以便安全移除元素
        let mut i = self.particles.len();
        while i > 0 {
            i -= 1;
            
            // 更新粒子，如果返回false表示粒子生命周期结束
            if !self.particles[i].update(dt) {
                self.particles.swap_remove(i);
            }
        }
    }
    
    // 绘制所有粒子
    pub fn draw(&self) {
        for particle in &self.particles {
            particle.draw();
        }
    }
    
    // 当前粒子数量
    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

// --- 新增: 全屏闪光特效结构体 ---
// pub struct ScreenFlash {
//     color: Color,
//     duration: f32,
//     lifetime: f32,
// }
//
// impl ScreenFlash {
//     fn new(color: Color, duration: f32) -> Self {
//         Self {
//             color,
//             duration,
//             lifetime: duration,
//         }
//     }
//
//     fn update(&mut self, dt: f32) -> bool {
//         self.lifetime -= dt;
//         self.lifetime > 0.0
//     }
//
//     fn draw(&self) {
//         // 使用缓出曲线 (ease-out) 计算透明度，使其开始时最亮，然后快速衰减
//         let t = 1.0 - (self.lifetime / self.duration); // 从 0.0 到 1.0
//         let alpha_multiplier = 1.0 - t * t; // (1-t^2) 缓出曲线
//
//         let flash_color = Color::new(self.color.r, self.color.g, self.color.b, self.color.a * alpha_multiplier);
//         draw_rectangle(0.0, 0.0, screen_width(), screen_height(), flash_color);
//     }
// }
// --- 结束 ---

// --- 新增: 浮动文本特效结构体 ---
pub struct FloatingText {
    text: String,
    position: Vec2,
    velocity: Vec2,
    color: Color,
    font_size: f32,
    lifetime: f32,
    max_lifetime: f32,
}

impl FloatingText {
    fn new(text: String, position: Vec2, color: Color, font_size: f32, lifetime: f32) -> Self {
        Self {
            text,
            position,
            velocity: Vec2::new(0.0, -50.0),
            color,
            font_size,
            lifetime,
            max_lifetime: lifetime,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        self.position += self.velocity * dt;
        self.lifetime -= dt;
        self.velocity *= 0.98; 
        self.lifetime > 0.0
    }

    fn draw(&self) {
        // --- 动画计算 ---
        let lifetime_ratio = self.lifetime / self.max_lifetime; // 从 1.0 递减到 0.0

        // 平滑的"弹出"缩放动画: 只在生命周期的前30%发生
        let scale_effect_duration_ratio = 0.3; // 动画持续时间占总生命周期的比例
        let scale_animation = if lifetime_ratio > (1.0 - scale_effect_duration_ratio) {
            // 计算缩放动画自身的进度 t (从 0.0 到 1.0)
            let t = (1.0 - lifetime_ratio) / scale_effect_duration_ratio;
            // 应用一个平滑的缓出函数 (easeOutQuint)
            let ease_out_factor = 1.0 - (1.0 - t).powi(5);
            // 从 1.5 倍大小平滑地插值到 1.0 倍
            1.5 - 0.5 * ease_out_factor
        } else {
            1.0 // 动画结束后，保持正常尺寸
        };
        let current_font_size = self.font_size * scale_animation;

        // 整体淡出效果
        let alpha = lifetime_ratio.powf(0.5);

        // --- 颜色计算 ---
        let shadow_color = Color::new(0.0, 0.0, 0.0, 0.5 * alpha);
        let text_color = Color::new(self.color.r, self.color.g, self.color.b, self.color.a * alpha);

        // --- 绘制 ---
        let text_dims = measure_text(&self.text, None, current_font_size as u16, 1.0);
        let shadow_offset = current_font_size * 0.05;

        // 1. 绘制投影
        draw_text(
            &self.text,
            self.position.x - text_dims.width / 2.0 + shadow_offset,
            self.position.y + shadow_offset,
            current_font_size,
            shadow_color,
        );
        
        // 2. 绘制主文本
        draw_text(
            &self.text,
            self.position.x - text_dims.width / 2.0,
            self.position.y,
            current_font_size,
            text_color,
        );
    }
}
// --- 结束 ---

pub struct Effects {
    // 游戏中使用的音效和特效
    pub particles: ParticleSystem,
    // pub screen_flash: Option<ScreenFlash>, // <-- 移除
    pub floating_texts: Vec<FloatingText>, // <-- 新增
}

impl Effects {
    pub fn new() -> Self {
        Effects {
            particles: ParticleSystem::new(),
            // screen_flash: None, // <-- 移除
            floating_texts: Vec::new(), // <-- 初始化
        }
    }
    
    pub fn play_place_sound(&self) {
        // 播放放置方块的音效
    }
    
    pub fn play_clear_sound(&self, _count: u32) {
        // 播放消除的音效，根据消除数量调整
    }
    
    // 用于单行消除的基础粒子效果
    pub fn show_clear_effect(&mut self, x: f32, y: f32, color: Color) {
        let position = Vec2::new(x, y);
        // 基础粒子数量
        let count = macroquad::rand::gen_range(15, 25);
        self.particles.create_effect_with_acceleration(
            position,
            color,
            count,
            (50.0, 150.0),            // 较慢的速度
            Vec2::new(0.0, 200.0),    // 轻微重力
            (2.0, 5.0),               // 标准尺寸
            (0.4, 1.2)                // 标准生命周期
        );
    }
    
    // 新增：用于多行消除的增强粒子效果
    pub fn show_multi_clear_effect(&mut self, x: f32, y: f32, color: Color) {
        let position = Vec2::new(x, y);
        // 更多的粒子
        let count = macroquad::rand::gen_range(30, 45); 
        self.particles.create_effect_with_acceleration(
            position,
            color,
            count,
            (100.0, 250.0),           // 更快的速度
            Vec2::new(0.0, 300.0),    // 略强的重力
            (3.0, 7.0),               // 更大的尺寸
            (0.8, 1.8)                // 更长的生命周期
        );
    }
    
    pub fn show_combo_effect(&mut self, combo: u32, x: f32, y: f32) {
        let position = Vec2::new(x, y);
        let count = 10 + (combo as i32 * 5);
        let gold_color = Color::new(1.0, 0.85, 0.0, 1.0);
        self.particles.create_clear_effect(position, gold_color, count);
    }
    
    // 新增：烟花效果（向上发射然后下落）
    pub fn show_firework_effect(&mut self, x: f32, y: f32, color: Color) {
        let position = Vec2::new(x, y);
        let count = 30;
        
        self.particles.create_effect_with_acceleration(
            position,
            color,
            count,
            (150.0, 300.0),
            Vec2::new(0.0, 500.0),
            (2.0, 6.0),
            (1.0, 2.0)
        );
    }
    
    // 新增：爆炸效果（向外扩散，带阻力）
    pub fn show_explosion_effect(&mut self, x: f32, y: f32, color: Color) {
        let position = Vec2::new(x, y);
        let count = 50;
        
        for _ in 0..count {
            let angle = macroquad::rand::gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = macroquad::rand::gen_range(200.0, 400.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            
            let drag_acceleration = -velocity.normalize() * 300.0;
            
            let size = macroquad::rand::gen_range(4.0, 8.0);
            let lifetime = macroquad::rand::gen_range(0.5, 1.5);
            
            let particle = Particle::new(
                position,
                velocity,
                drag_acceleration,
                color,
                size,
                lifetime
            );
            
            self.particles.particles.push(particle);
        }
    }
    
    // --- 新增: 触发全屏闪光的方法 ---
    // pub fn show_screen_flash(&mut self, color: Color, duration: f32) { ... }
    // --- 结束 ---
    
    // --- 新增: 触发浮动文本的方法 ---
    pub fn show_floating_text(&mut self, text: String, position: Vec2, color: Color, size: f32, duration: f32) {
        self.floating_texts.push(FloatingText::new(text, position, color, size, duration));
    }
    // --- 结束 ---
    
    pub fn update(&mut self, dt: f32) {
        // 更新所有特效
        self.particles.update(dt);

        // 更新浮动文本
        self.floating_texts.retain_mut(|text| text.update(dt));
    }
    
    pub fn draw(&self) {
        // 绘制所有特效
        self.particles.draw();

        // 绘制浮动文本
        for text in &self.floating_texts {
            text.draw();
        }
    }
} 