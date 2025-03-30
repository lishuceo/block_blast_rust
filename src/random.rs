// 简单的随机数生成器，不依赖wasm-bindgen
// 使用Xorshift算法，适用于WebAssembly环境

/// 自定义随机数生成器
pub struct SimpleRandom {
    state: u64,
}

impl SimpleRandom {
    /// 使用种子创建新的随机数生成器
    /// 如果种子为0，使用一个默认非零值
    pub fn new(seed: u64) -> Self {
        let seed = if seed == 0 { 0x853c49e6748fea9b } else { seed };
        SimpleRandom { state: seed }
    }
    
    /// 创建使用当前时间作为种子的随机数生成器
    pub fn new_from_time() -> Self {
        // 使用macroquad的get_time()生成种子
        let time = macroquad::time::get_time();
        let seed = (time * 1000.0) as u64;
        Self::new(seed)
    }
    
    /// 生成下一个随机u64值
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    
    /// 生成0到1之间的随机浮点数
    pub fn next_float(&mut self) -> f32 {
        // 将u64转换为0到1之间的f32
        (self.next_u64() & 0x00_FFFF_FFFF) as f32 / 0x1_0000_0000 as f32
    }
    
    /// 生成指定范围内的随机整数
    pub fn gen_range(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next_u64() % (max - min) as u64) as i32
    }
    
    /// 生成指定范围内的随机浮点数
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        if max <= min {
            return min;
        }
        min + self.next_float() * (max - min)
    }
    
    /// 从切片中随机选择一个元素
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let index = self.gen_range(0, slice.len() as i32) as usize;
        slice.get(index)
    }
}

// 提供一个全局随机数生成器
static mut GLOBAL_RNG: Option<SimpleRandom> = None;

/// 初始化全局随机数生成器
pub fn init_global_rng() {
    unsafe {
        GLOBAL_RNG = Some(SimpleRandom::new_from_time());
    }
}

/// 生成0到max之间的随机整数
pub fn gen_range(min: i32, max: i32) -> i32 {
    unsafe {
        if GLOBAL_RNG.is_none() {
            init_global_rng();
        }
        GLOBAL_RNG.as_mut().unwrap().gen_range(min, max)
    }
}

/// 生成min到max之间的随机浮点数
pub fn gen_range_f32(min: f32, max: f32) -> f32 {
    unsafe {
        if GLOBAL_RNG.is_none() {
            init_global_rng();
        }
        GLOBAL_RNG.as_mut().unwrap().gen_range_f32(min, max)
    }
}

/// 从切片中随机选择一个元素
pub fn choose<'a, T>(slice: &'a [T]) -> Option<&'a T> {
    unsafe {
        if GLOBAL_RNG.is_none() {
            init_global_rng();
        }
        GLOBAL_RNG.as_mut().unwrap().choose(slice)
    }
} 