// src/wave.rs

// 修改日志宏的导入方式
use crate::{log_debug, log_info, log_warn}; 
// 导入随机数工具 (假设存在 random 模块)
// use crate::random; // 确保 random 模块存在并包含 get_rand_range
use macroquad::rand; // 使用 macroquad 的随机数生成器
use crate::GameMode;

/// 表示当前的波次阶段
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WavePhase {
    Accumulation,       // 积累阶段
    ChallengeActive(ChallengeType), // 挑战进行中 (包含具体挑战类型)
    Relief,             // 缓和阶段
}

/// 表示具体的挑战类型
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChallengeType {
    BlockFlood,         // 方块潮
    // TargetRows(u8),     // 精准消除行任务 (目标数量) - 简化：只存数量
    // TargetCols(u8),     // 精准消除列任务 (目标数量) - 简化：只存数量
}

/// 负责管理游戏波次、难度和挑战状态
#[derive(Debug, Clone)]
pub struct WaveManager {
    turn_count: u32,              // 总回合数
    current_phase: WavePhase,     // 当前所处阶段
    turns_in_phase: u32,          // 当前阶段已持续的回合数

    // --- 配置参数 ---
    accumulation_turns: u32,
    challenge_active_turns: u32,
    relief_turns: u32,
    
    // --- 动态状态 ---
    pub blocks_per_generation: usize,
    pub block_complexity_factor: f32,
    
    // --- 挑战特定状态 ---
    // active_target_lines: Vec<usize>, // 存储当前激活的目标行/列索引 - REMOVED
    // target_lines_cleared_count: u32,  // 本次挑战已清除的目标数量 - REMOVED
    // required_targets_for_success: u8, // 新增：记录当前目标挑战需要完成的数量 - REMOVED

    pending_score_bonus: u32, // <--- 新增：待处理的奖励分数
}

// 定义奖励常量
// const SCORE_BONUS_PER_TARGET_LINE: u32 = 50; // REMOVED
// const SCORE_BONUS_TARGET_CHALLENGE_COMPLETION: u32 = 200; // REMOVED
const SCORE_BONUS_BLOCK_FLOOD_SURVIVAL: u32 = 100;

impl WaveManager {
    pub fn new() -> Self {
        WaveManager {
            turn_count: 0,
            current_phase: WavePhase::Accumulation,
            turns_in_phase: 0,

            accumulation_turns: 20, // 保持原样或调整
            challenge_active_turns: 5, // BlockFlood 持续回合数
            relief_turns: 3,

            blocks_per_generation: 3,
            block_complexity_factor: 0.1,
            
            // active_target_lines: Vec::new(), // REMOVED
            // target_lines_cleared_count: 0, // REMOVED
            // required_targets_for_success: 0, // REMOVED
            pending_score_bonus: 0,
        }
    }

    /// 当玩家成功放置一个方块后调用，返回该回合产生的奖励分数
    pub fn increment_turn(&mut self) -> u32 {
        self.turn_count += 1;
        self.turns_in_phase += 1;
        
        log_debug!("Turn incremented: Total={}, PhaseTurns={}", self.turn_count, self.turns_in_phase);

        self.update_phase(); 
        self.update_difficulty(); 

        let bonus_to_award = self.pending_score_bonus;
        self.pending_score_bonus = 0;
        bonus_to_award
    }

    /// 更新当前波次阶段
    fn update_phase(&mut self) {
        let phase_duration = match self.current_phase {
            WavePhase::Accumulation => self.accumulation_turns,
            WavePhase::ChallengeActive(_) => self.challenge_active_turns,
            WavePhase::Relief => self.relief_turns,
        };

        if self.turns_in_phase >= phase_duration {
             let next_phase = match self.current_phase {
                 WavePhase::Accumulation => {
                     let next_challenge = self.select_next_challenge();
                     self.start_challenge(next_challenge); 
                     WavePhase::ChallengeActive(next_challenge)
                 }
                 WavePhase::ChallengeActive(_) => {
                      self.end_challenge(); 
                      WavePhase::Relief
                 }
                 WavePhase::Relief => WavePhase::Accumulation,
             };
             self.transition_to(next_phase);
        }
    }

    /// 平滑地切换到新阶段
    fn transition_to(&mut self, next_phase: WavePhase) {
        log_info!("Transitioning from {:?} to {:?}", self.current_phase, next_phase);
        self.current_phase = next_phase;
        self.turns_in_phase = 0;
        // 清理不再需要的挑战状态
        // self.active_target_lines.clear(); // REMOVED
        // self.target_lines_cleared_count = 0; // REMOVED
        // self.required_targets_for_success = 0; // REMOVED
    }
    
    /// 根据总回合数和当前阶段更新难度参数
    fn update_difficulty(&mut self) {
        let base_complexity: f32 = (0.15 + self.turn_count as f32 / 60.0).min(0.7);

        match self.current_phase {
            WavePhase::Accumulation => {
                self.block_complexity_factor = base_complexity * 0.5;
            }
            WavePhase::ChallengeActive(challenge_type) => {
                match challenge_type {
                     ChallengeType::BlockFlood => {
                          self.block_complexity_factor = (base_complexity * 1.3).min(0.9); 
                     }
                     // TargetRows and TargetCols removed
                }
            }
            WavePhase::Relief => {
                self.block_complexity_factor = base_complexity * 0.4;
            }
        }
        log_debug!(
            "Difficulty updated: Blocks={}, Complexity={:.2}", 
            self.blocks_per_generation,
            self.block_complexity_factor
        );
    }

    /// 选择下一个挑战类型
    fn select_next_challenge(&self) -> ChallengeType {
        // 既然只有 BlockFlood，直接返回
        ChallengeType::BlockFlood
    }
    
    /// 开始挑战时的设置逻辑
    fn start_challenge(&mut self, challenge_type: ChallengeType) {
         log_info!("Starting challenge: {:?}", challenge_type);
         match challenge_type {
             ChallengeType::BlockFlood => {
                 // BlockFlood 可能不需要特殊设置，或者可以在这里调整一些参数
                 log_info!("BlockFlood challenge started.");
             }
             // TargetRows and TargetCols removed
         }
    }

    /// 挑战结束时的清理逻辑，并计算奖励
    fn end_challenge(&mut self) {
         log_info!("Ending challenge (current phase was: {:?})", self.current_phase);
         if let WavePhase::ChallengeActive(challenge_type) = self.current_phase {
             match challenge_type {
                  ChallengeType::BlockFlood => {
                      log_info!("BlockFlood challenge survived!");
                      self.pending_score_bonus += SCORE_BONUS_BLOCK_FLOOD_SURVIVAL;
                      log_info!("Awarding survival bonus: {}. Total pending: {}", SCORE_BONUS_BLOCK_FLOOD_SURVIVAL, self.pending_score_bonus);
                  }
                  // TargetRows and TargetCols removed
             }
         }
    }

    /// 通知管理器某行/列已被清除
    pub fn notify_line_cleared(&mut self, _index: usize, _is_row: bool) {
        // 由于移除了目标挑战，此函数目前不需要做太多事情
        // 如果未来有其他基于消除行的挑战，可以在这里添加逻辑
         if let WavePhase::ChallengeActive(challenge_type) = self.current_phase {
             match challenge_type {
                 ChallengeType::BlockFlood => {
                     // BlockFlood 期间的消除可能没有特殊奖励，或者可以有通用奖励
                 }
                 // TargetRows and TargetCols removed
             }
         }
    }
    
    /// 获取当前激活的目标行/列 (用于绘制高亮)
    pub fn get_active_target_lines(&self) -> Vec<usize> { // 修改返回类型以适应移除
        // 因为不再有目标行/列，返回一个空Vec
        Vec::new()
    }

    // --- 其他 Getter 方法 ---
    pub fn get_current_phase(&self) -> WavePhase {
        self.current_phase
    }

    pub fn get_turn_count(&self) -> u32 {
        self.turn_count
    }
    
    pub fn is_challenge_active(&self) -> bool {
         matches!(self.current_phase, WavePhase::ChallengeActive(_))
    }
    
    pub fn get_active_challenge_type(&self) -> Option<ChallengeType> {
         if let WavePhase::ChallengeActive(challenge_type) = self.current_phase {
              Some(challenge_type)
         } else {
              None
         }
    }

    /// 根据当前阶段和棋盘填充率，决定是否应该提供"有用"的方块
    pub fn should_offer_helpful_block(&self, grid_filled_ratio: f32) -> bool {
        let random_chance = rand::gen_range(0.0, 1.0) as f32;
        log_info!("should_offer_helpful_block: filled_ratio: {}, random_chance: {}, phase: {:?}", grid_filled_ratio, random_chance, self.current_phase);

        match self.current_phase {
            WavePhase::Relief => {
                if grid_filled_ratio >= 0.30 { random_chance < 0.70 } 
                else if grid_filled_ratio >= 0.35 { random_chance < 0.80 } 
                else if grid_filled_ratio >= 0.45 { random_chance < 0.99 } 
                else { false }
            }
            WavePhase::Accumulation => {
                if grid_filled_ratio >= 0.30 { random_chance < 0.70 } 
                else if grid_filled_ratio >= 0.40 { random_chance < 0.85 } 
                else if grid_filled_ratio >= 0.50 { random_chance < 0.95 } 
                else { false }
            }
            WavePhase::ChallengeActive(_) => { // 现在只有 BlockFlood
                if grid_filled_ratio >= 0.45 { random_chance < 0.80 } 
                else if grid_filled_ratio >= 0.60 { random_chance < 0.95 } 
                else { false }
            }
        }
    }

    /// 根据当前阶段和棋盘困难度分数，决定是否应该提供"有用"的方块
    /// difficulty_score: 0.0-1.0 之间的值，越高表示越困难
    pub fn should_offer_helpful_block_v2(&self, difficulty_score: f32) -> bool {
        let random_chance = rand::gen_range(0.0, 1.0) as f32;
        log_info!("should_offer_helpful_block_v2: difficulty_score: {:.3}, random_chance: {:.3}, phase: {:?}", difficulty_score, random_chance, self.current_phase);

        match self.current_phase {
            WavePhase::Relief => {
                // 缓和阶段：更容易获得帮助，降低阈值
                if difficulty_score >= 0.5 { random_chance < 0.95 }      // 从0.7降到0.5
                else if difficulty_score >= 0.35 { random_chance < 0.80 } // 从0.5降到0.35
                else if difficulty_score >= 0.2 { random_chance < 0.60 }  // 从0.3降到0.2
                else if difficulty_score >= 0.1 { random_chance < 0.35 }  // 新增档位
                else { random_chance < 0.15 }  // 从0.2降到0.15
            }
            WavePhase::Accumulation => {
                // 积累阶段：适中的帮助，也降低阈值
                if difficulty_score >= 0.6 { random_chance < 0.90 }      // 从0.8降到0.6
                else if difficulty_score >= 0.45 { random_chance < 0.70 } // 从0.6降到0.45
                else if difficulty_score >= 0.3 { random_chance < 0.50 }  // 从0.4降到0.3
                else if difficulty_score >= 0.15 { random_chance < 0.30 } // 从0.25降到0.15
                else if difficulty_score >= 0.05 { random_chance < 0.10 } // 新增档位
                else { false }  // 容易时不帮助
            }
            WavePhase::ChallengeActive(_) => { 
                // 挑战阶段：较少的帮助，但也适当降低阈值
                if difficulty_score >= 0.7 { random_chance < 0.80 }      // 从0.85降到0.7
                else if difficulty_score >= 0.55 { random_chance < 0.50 } // 从0.7降到0.55
                else if difficulty_score >= 0.4 { random_chance < 0.25 }  // 从0.5降到0.4
                else if difficulty_score >= 0.25 { random_chance < 0.10 } // 新增档位
                else { false }  // 不太困难时不帮助
            }
        }
    }
} 
 