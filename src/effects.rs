// 音效和特效
use macroquad::prelude::*;

// 单个粒子的定义
pub struct Particle {
    position: Vec2,
    velocity: Vec2,
    color: Color,
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
}

impl Particle {
    pub fn new(position: Vec2, velocity: Vec2, color: Color, size: f32, lifetime: f32) -> Self {
        Particle {
            position,
            velocity,
            color,
            size,
            lifetime,
            max_lifetime: lifetime,
        }
    }
    
    // 更新粒子状态
    pub fn update(&mut self, dt: f32) -> bool {
        self.position += self.velocity * dt;
        self.lifetime -= dt;
        self.velocity.y += 50.0 * dt; // 简单重力效果
        
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
            let speed = macroquad::rand::gen_range(30.0, 80.0);
            let velocity = Vec2::new(
                angle.cos() * speed,
                angle.sin() * speed
            );
            
            // 随机大小
            let size = macroquad::rand::gen_range(3.0, 8.0);
            
            // 随机生命周期
            let lifetime = macroquad::rand::gen_range(0.5, 1.5);
            
            // 创建新粒子
            let particle = Particle::new(
                position,
                velocity,
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

pub struct Effects {
    // 游戏中使用的音效和特效
    pub particles: ParticleSystem,
}

impl Effects {
    pub fn new() -> Self {
        Effects {
            particles: ParticleSystem::new()
        }
    }
    
    pub fn play_place_sound(&self) {
        // 播放放置方块的音效
    }
    
    pub fn play_clear_sound(&self, _count: u32) {
        // 播放消除的音效，根据消除数量调整
    }
    
    pub fn show_clear_effect(&mut self, x: f32, y: f32, color: Color) {
        // 显示消除特效
        let position = Vec2::new(x, y);
        // 生成20-40个粒子
        let count = macroquad::rand::gen_range(20, 40);
        self.particles.create_clear_effect(position, color, count);
    }
    
    pub fn show_combo_effect(&mut self, combo: u32, x: f32, y: f32) {
        // 显示连击特效 - 连击数越高，粒子越多
        let position = Vec2::new(x, y);
        // 根据连击数增加粒子数量
        let count = 10 + (combo as i32 * 5);
        // 连击效果使用金色粒子
        let gold_color = Color::new(1.0, 0.85, 0.0, 1.0);
        self.particles.create_clear_effect(position, gold_color, count);
    }
    
    pub fn update(&mut self, dt: f32) {
        // 更新所有特效
        self.particles.update(dt);
    }
    
    pub fn draw(&self) {
        // 绘制所有特效
        self.particles.draw();
    }
} 