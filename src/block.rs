// 方块模块，包含方块形状定义和生成逻辑
use macroquad::prelude::*;
// 不再需要使用我们的自定义随机数生成器

// 定义不同形状的方块
pub struct BlockShape {
    pub cells: Vec<(i32, i32)>,
    pub color: Color,
}

impl BlockShape {
    // 获取所有可用的方块形状
    fn get_all_shapes() -> Vec<Vec<(i32, i32)>> {
        vec![
            // 简单形状 (1-2个单元格)
            vec![(0, 0)],  // 单个方块
            vec![(0, 0), (1, 0)],  // 水平2格
            vec![(0, 0), (0, 1)],  // 垂直2格
            
            // 标准形状 (大部分俄罗斯方块形状)
            vec![(0, 0), (1, 0), (2, 0), (3, 0)],  // I形
            vec![(0, 0), (1, 0), (0, 1), (1, 1)],  // 方形
            vec![(0, 0), (1, 0), (2, 0), (2, 1)],  // L形
            vec![(0, 0), (1, 0), (2, 0), (1, 1)],  // T形
            vec![(0, 0), (1, 0), (1, 1), (2, 1)],  // Z形
            
            // 复杂形状 (更多单元格或不规则形状)
            vec![(0, 0), (0, 1), (0, 2), (1, 0), (2, 0)],  // 大L形
            vec![(0, 0), (1, 0), (2, 0), (0, 1), (0, 2)],  // 反大L形
            vec![(0, 0), (1, 0), (0, 1), (1, 1), (2, 1)],  // 阶梯形
            vec![(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)],  // 十字形
        ]
    }
    
    // 随机生成一个方块(原始实现)
    pub fn random() -> Self {
        let shapes = Self::get_all_shapes();
        let shape_idx = macroquad::rand::gen_range(0, shapes.len() as i32);
        
        // 随机选择颜色
        let colors = [RED, GREEN, BLUE, YELLOW, PURPLE, PINK, ORANGE];
        let color_idx = macroquad::rand::gen_range(0, colors.len() as i32);
        
        BlockShape {
            cells: shapes[shape_idx as usize].clone(),
            color: colors[color_idx as usize],
        }
    }
    
    // 根据自定义概率随机生成一个方块
    pub fn random_with_chances(simple_chance: i32, standard_chance: i32) -> Self {
        let shapes = Self::get_all_shapes();
        
        // 确保概率有效(在0-100之间)
        let simple_chance = simple_chance.max(0).min(100);
        let standard_chance = standard_chance.max(0).min(100);
        let _complex_chance = 100 - simple_chance - standard_chance;
        
        // 根据概率选择形状类别
        let category_roll = macroquad::rand::gen_range(0, 100);
        
        let shape_idx = if category_roll < simple_chance {
            // 选择简单形状(索引0-2)
            macroquad::rand::gen_range(0, 3)
        } else if category_roll < simple_chance + standard_chance {
            // 选择标准形状(索引3-7)
            macroquad::rand::gen_range(3, 8)
        } else {
            // 选择复杂形状(索引8-11)
            macroquad::rand::gen_range(8, 12)
        };
        
        // 随机选择颜色
        let colors = [RED, GREEN, BLUE, YELLOW, PURPLE, PINK, ORANGE];
        let color_idx = macroquad::rand::gen_range(0, colors.len() as i32);
        
        BlockShape {
            cells: shapes[shape_idx as usize].clone(),
            color: colors[color_idx as usize],
        }
    }
} 