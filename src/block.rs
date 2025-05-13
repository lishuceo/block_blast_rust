// 方块模块，包含方块形状定义和生成逻辑
use macroquad::prelude::*;
use crate::GameMode; // 导入 GameMode
// 不再需要使用我们的自定义随机数生成器

// --- Base Shape Definitions ---
// Using &'static [(i32, i32)] for efficiency
// 公开一些基础形状，以便 grid.rs 可以引用它们进行匹配
pub const SHAPE_DOT: &[(i32, i32)] = &[(0, 0)];
pub const SHAPE_H2: &[(i32, i32)] = &[(0, 0), (1, 0)];
const SHAPE_DG: &[(i32, i32)] = &[(0, 0), (1, 1)]; 
const SHAPE_H3: &[(i32, i32)] = &[(0, 0), (1, 0), (2, 0)];
const SHAPE_I: &[(i32, i32)] = &[(0, 0), (1, 0), (2, 0), (3, 0)];
const SHAPE_O: &[(i32, i32)] = &[(0, 0), (1, 0), (0, 1), (1, 1)];
const SHAPE_L: &[(i32, i32)] = &[(0, 0), (0, 1), (0, 2), (1, 2)]; // Base L
const SHAPE_T: &[(i32, i32)] = &[(0, 0), (1, 0), (2, 0), (1, 1)];
const SHAPE_Z: &[(i32, i32)] = &[(0, 0), (1, 0), (1, 1), (2, 1)]; // Base Z
const SHAPE_CROSS: &[(i32, i32)] = &[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)];
const SHAPE_RECT_2X3: &[(i32, i32)] = &[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)];
const SHAPE_SQUARE_3X3: &[(i32, i32)] = &[(0,0), (1,0), (2,0), (0,1), (1,1), (2,1), (0,2), (1,2), (2,2)];
const SHAPE_L_LARGE: &[(i32, i32)] = &[(0, 0), (0, 1), (0, 2), (1, 0), (2, 0)]; // 大厂形
const SHAPE_STAIR: &[(i32, i32)] = &[(0, 0), (1, 0), (0, 1), (1, 1), (2, 1)]; // 阶梯形

// --- Pool Entry Definition ---
#[derive(Clone)] // Added Clone for potential future use if needed
struct PoolEntry {
    shape_ref: &'static [(i32, i32)], // Reference to the base shape
    weight: u32,                    // Generation weight
}

// --- Mode Pool Definitions ---
// Weights are relative probabilities within each pool
const EASY_POOL: &[PoolEntry] = &[
    PoolEntry { shape_ref: SHAPE_DOT, weight: 2 },
    PoolEntry { shape_ref: SHAPE_H2, weight: 5 },
    PoolEntry { shape_ref: SHAPE_O, weight: 10 },
    PoolEntry { shape_ref: SHAPE_I, weight: 10 },
    PoolEntry { shape_ref: SHAPE_H3, weight: 6 },
    PoolEntry { shape_ref: SHAPE_L, weight: 8 },
    PoolEntry { shape_ref: SHAPE_T, weight: 2 },
    PoolEntry { shape_ref: SHAPE_Z, weight: 2 },
    // Lower probability for larger/complex shapes in easy mode
];

const NORMAL_POOL: &[PoolEntry] = &[
    PoolEntry { shape_ref: SHAPE_DOT, weight: 3 }, 
    PoolEntry { shape_ref: SHAPE_H2, weight: 5 },
    PoolEntry { shape_ref: SHAPE_O, weight: 10 },
    PoolEntry { shape_ref: SHAPE_I, weight: 8 },
    PoolEntry { shape_ref: SHAPE_H3, weight: 7 },
    PoolEntry { shape_ref: SHAPE_L, weight: 9 },
    PoolEntry { shape_ref: SHAPE_T, weight: 9 },
    PoolEntry { shape_ref: SHAPE_Z, weight: 8 },
    PoolEntry { shape_ref: SHAPE_STAIR, weight: 5 },
    PoolEntry { shape_ref: SHAPE_L_LARGE, weight: 4 },
    PoolEntry { shape_ref: SHAPE_CROSS, weight: 3 },
    PoolEntry { shape_ref: SHAPE_RECT_2X3, weight: 4 },
    PoolEntry { shape_ref: SHAPE_SQUARE_3X3, weight: 3 },
];

const HAPPY_POOL: &[PoolEntry] = &[
    PoolEntry { shape_ref: SHAPE_DOT, weight: 1 }, 
    PoolEntry { shape_ref: SHAPE_H2, weight: 5 },
    PoolEntry { shape_ref: SHAPE_O, weight: 10 },
    PoolEntry { shape_ref: SHAPE_I, weight: 10 },
    PoolEntry { shape_ref: SHAPE_H3, weight: 7 },
    PoolEntry { shape_ref: SHAPE_L, weight: 7 },
    PoolEntry { shape_ref: SHAPE_T, weight: 7 },
    PoolEntry { shape_ref: SHAPE_Z, weight: 8 },
    PoolEntry { shape_ref: SHAPE_STAIR, weight: 5 },
    PoolEntry { shape_ref: SHAPE_L_LARGE, weight: 4 },
    PoolEntry { shape_ref: SHAPE_RECT_2X3, weight: 4 },
    PoolEntry { shape_ref: SHAPE_SQUARE_3X3, weight: 2 },
];

// 内部辅助函数：顺时针旋转90度
pub fn rotate_90_clockwise(cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
    cells.iter().map(|&(x, y)| (y, -x)).collect()
}

// 内部辅助函数：标准化坐标，将左上角移至(0,0)附近
pub fn normalize_cells(cells: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    if cells.is_empty() {
        return cells;
    }
    let min_x = cells.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = cells.iter().map(|(_, y)| *y).min().unwrap_or(0);
    cells
        .into_iter()
        .map(|(x, y)| (x - min_x, y - min_y))
        .collect()
}

// 定义不同形状的方块
#[derive(Debug, Clone)] // Added Debug and Clone
pub struct BlockShape {
    pub cells: Vec<(i32, i32)>,
    pub color: Color,
}

// 公共函数：获取一个随机的方块颜色
pub fn get_random_block_color() -> Color {
    let colors = [
        Color::from_rgba(235, 177, 67, 255), // EBB143
        Color::from_rgba(212, 59, 54, 255),  // D43B36
        Color::from_rgba(68, 96, 223, 255),  // 4460DF
        Color::from_rgba(141, 94, 208, 255), // 8D5ED0
        Color::from_rgba(62, 181, 224, 255), // 3EB5E0
        Color::from_rgba(71, 185, 71, 255),  // 47B947
        Color::from_rgba(227, 95, 57, 255),  // E35F39
    ];
    let color_idx = macroquad::rand::gen_range(0, colors.len() as i32);
    colors[color_idx as usize]
}

impl BlockShape {
    // Selects a shape based on weights from a single pool
    fn select_weighted_shape(pool: &[PoolEntry]) -> &'static [(i32, i32)] {
        let total_weight: u32 = pool.iter().map(|entry| entry.weight).sum();
        if total_weight == 0 {
            // Fallback if pool is empty or weights are all zero
            return SHAPE_DOT; 
        }
        let mut roll = macroquad::rand::gen_range(0, total_weight); // roll in [0, total_weight - 1]

        for entry in pool {
            if roll < entry.weight {
                return entry.shape_ref;
            }
            roll -= entry.weight;
        }
        // Should not be reached if total_weight > 0, but as a safeguard:
        pool.last().map(|entry| entry.shape_ref).unwrap_or(SHAPE_DOT)
    }

    /// Creates a new 1x1 dot block shape.
    pub fn new_dot() -> Self {
        BlockShape {
            cells: SHAPE_DOT.to_vec(), // SHAPE_DOT is already normalized (0,0)
            color: get_random_block_color(), // 使用新的公共颜色函数
        }
    }

    /// Generates a block using a complexity factor (0.0 to 1.0)
    /// Higher complexity means higher chance of complex shapes.
    pub fn generate_with_complexity(complexity: f32) -> Self {
        // Clamp complexity to [0.0, 1.0]
        let complexity = complexity.max(0.0).min(1.0);

        // Determine the contribution weight of each pool based on complexity
        // These weights determine how likely we are to pick from each pool
        // Example interpolation:
        let easy_weight = (1.0 - complexity).powi(2); // High weight when complexity is low
        let normal_weight = 1.0 - (complexity * 2.0 - 1.0).abs(); // Peaks at complexity 0.5
        let happy_weight = complexity.powi(2); // High weight when complexity is high

        // Calculate total weight across all *relevant* entries
        let mut weighted_entries: Vec<(&'static [(i32, i32)], f32)> = Vec::new();
        let mut total_blend_weight = 0.0;

        // Add entries from EASY_POOL, scaled by easy_weight
        for entry in EASY_POOL {
            let weight = entry.weight as f32 * easy_weight;
            if weight > 0.0 {
                weighted_entries.push((entry.shape_ref, weight));
                total_blend_weight += weight;
            }
        }
        // Add entries from NORMAL_POOL, scaled by normal_weight
        for entry in NORMAL_POOL {
            let weight = entry.weight as f32 * normal_weight;
            if weight > 0.0 {
                weighted_entries.push((entry.shape_ref, weight));
                total_blend_weight += weight;
            }
        }
        // Add entries from HAPPY_POOL, scaled by happy_weight
        for entry in HAPPY_POOL {
            let weight = entry.weight as f32 * happy_weight;
            if weight > 0.0 {
                weighted_entries.push((entry.shape_ref, weight));
                total_blend_weight += weight;
            }
        }
        
        // Select a shape based on the blended weights
        let base_shape_cells = if total_blend_weight <= 0.0 {
            SHAPE_DOT // Fallback if weights are zero
        } else {
            let mut roll = macroquad::rand::gen_range(0.0, total_blend_weight);
            let mut selected_shape = SHAPE_DOT; // Default fallback
            for (shape_ref, weight) in weighted_entries {
                if roll < weight {
                    selected_shape = shape_ref;
                    break;
                }
                roll -= weight;
            }
            selected_shape
        };

        // --- Reuse existing rotation and color logic --- 
        let mut current_cells = base_shape_cells.to_vec(); 

        // Apply random rotation
        let num_rotations = macroquad::rand::gen_range(0, 4);
        for _ in 0..num_rotations {
            current_cells = rotate_90_clockwise(&current_cells);
        }

        // Normalize coordinates after rotation
        let final_cells = normalize_cells(current_cells);

        // Select a random color
        BlockShape {
            cells: final_cells,
            color: get_random_block_color(), // 使用新的公共颜色函数
        }
    }

    // Keep generate_for_mode for now, maybe for specific scenarios or testing
    pub fn generate_for_mode(mode: GameMode) -> Self {
        let pool = match mode {
            GameMode::Easy => EASY_POOL,
            GameMode::Normal => NORMAL_POOL,
            GameMode::Happy => HAPPY_POOL,
        };
        let base_shape_cells = Self::select_weighted_shape(pool);

        let mut current_cells = base_shape_cells.to_vec(); // Clone the base shape

        // Apply random rotation
        let num_rotations = macroquad::rand::gen_range(0, 4);
        for _ in 0..num_rotations {
            current_cells = rotate_90_clockwise(&current_cells);
        }

        // Normalize coordinates after rotation
        let final_cells = normalize_cells(current_cells);

        // Select a random color (same logic as before)
        BlockShape {
            cells: final_cells,
            color: get_random_block_color(), // 使用新的公共颜色函数
        }
    }
} 