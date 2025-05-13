// src/wave.rs

// 修改日志宏的导入方式
use crate::{log_debug, log_info, log_warn}; 
// 导入随机数工具 (假设存在 random 模块)
use crate::random; // 确保 random 模块存在并包含 get_rand_range

/// 表示当前的波次阶段
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WavePhase {
    Accumulation,       // 积累阶段
    ChallengeIncoming,  // 挑战预告阶段
    ChallengeActive(ChallengeType), // 挑战进行中 (包含具体挑战类型)
    Relief,             // 缓和阶段
}

/// 表示具体的挑战类型
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChallengeType {
    BlockFlood,         // 方块潮
    TargetRows(u8),     // 精准消除行任务 (目标数量) - 简化：只存数量
    TargetCols(u8),     // 精准消除列任务 (目标数量) - 简化：只存数量
    // HighValueBlocks, // 待添加
}

/// 负责管理游戏波次、难度和挑战状态
#[derive(Debug, Clone)]
pub struct WaveManager {
    turn_count: u32,              // 总回合数
    current_phase: WavePhase,     // 当前所处阶段
    turns_in_phase: u32,          // 当前阶段已持续的回合数

    // --- 配置参数 ---
    accumulation_turns: u32,
    challenge_incoming_turns: u32,
    challenge_active_turns: u32,
    relief_turns: u32,
    
    // --- 动态状态 ---
    pub blocks_per_generation: usize,
    pub block_complexity_factor: f32,
    
    // --- 挑战特定状态 ---
    // 注意：实际的障碍物/目标列表由 Grid 或 Game 管理，这里只存触发状态或计数
    active_target_lines: Vec<usize>, // 存储当前激活的目标行/列索引
    target_lines_cleared_count: u32,  // 本次挑战已清除的目标数量
    required_targets_for_success: u8, // 新增：记录当前目标挑战需要完成的数量

    pending_score_bonus: u32, // <--- 新增：待处理的奖励分数
}

// 定义奖励常量
const SCORE_BONUS_PER_TARGET_LINE: u32 = 50;
const SCORE_BONUS_TARGET_CHALLENGE_COMPLETION: u32 = 200;
const SCORE_BONUS_BLOCK_FLOOD_SURVIVAL: u32 = 100;

impl WaveManager {
    pub fn new() -> Self {
        WaveManager {
            turn_count: 0,
            current_phase: WavePhase::Accumulation,
            turns_in_phase: 0,

            accumulation_turns: 15,
            challenge_incoming_turns: 1,
            challenge_active_turns: 5,
            relief_turns: 3,

            blocks_per_generation: 3,
            block_complexity_factor: 0.1,
            
            active_target_lines: Vec::new(),
            target_lines_cleared_count: 0,
            required_targets_for_success: 0, // 初始化
            pending_score_bonus: 0, // <--- 初始化
        }
    }

    /// 当玩家成功放置一个方块后调用，返回该回合产生的奖励分数
    pub fn increment_turn(&mut self) -> u32 { // <--- 修改返回值
        self.turn_count += 1;
        self.turns_in_phase += 1;
        
        log_debug!("Turn incremented: Total={}, PhaseTurns={}", self.turn_count, self.turns_in_phase);

        // 在增加回合数后立即更新阶段，这样新阶段的逻辑可以在同一帧生效
        self.update_phase(); 
        // 然后基于可能更新后的阶段来更新难度
        self.update_difficulty(); 

        let bonus_to_award = self.pending_score_bonus;
        self.pending_score_bonus = 0; // 重置待处理奖励
        bonus_to_award // <--- 返回奖励
    }

    /// 更新当前波次阶段
    fn update_phase(&mut self) {
        let phase_duration = match self.current_phase {
            WavePhase::Accumulation => self.accumulation_turns,
            WavePhase::ChallengeIncoming => self.challenge_incoming_turns,
            WavePhase::ChallengeActive(_) => self.challenge_active_turns,
            WavePhase::Relief => self.relief_turns,
        };

        if self.turns_in_phase >= phase_duration {
             let next_phase = match self.current_phase {
                 WavePhase::Accumulation => WavePhase::ChallengeIncoming,
                 WavePhase::ChallengeIncoming => {
                     let next_challenge = self.select_next_challenge();
                     // 在转换前触发挑战开始逻辑
                     self.start_challenge(next_challenge); 
                     WavePhase::ChallengeActive(next_challenge)
                 }
                 WavePhase::ChallengeActive(_) => {
                      self.end_challenge(); // 在转换前结束挑战
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
        self.turns_in_phase = 0; // 重置阶段回合计数器
        
        // 清理通用挑战状态
        self.active_target_lines.clear();
        self.target_lines_cleared_count = 0;
        self.required_targets_for_success = 0; // 重置
    }
    
    /// 根据总回合数和当前阶段更新难度参数
    fn update_difficulty(&mut self) {
        // 调整后的复杂度计算
        let base_complexity: f32 = (0.15 + self.turn_count as f32 / 60.0).min(0.7);

        match self.current_phase {
            WavePhase::Accumulation => {
                self.block_complexity_factor = base_complexity * 0.5; // 积累阶段复杂度更低
            }
            WavePhase::ChallengeIncoming => {
                self.block_complexity_factor = base_complexity;
            }
            WavePhase::ChallengeActive(challenge_type) => {
                match challenge_type {
                     ChallengeType::BlockFlood => {
                          self.block_complexity_factor = (base_complexity * 1.3).min(0.9); 
                     }
                     ChallengeType::TargetRows(_) | ChallengeType::TargetCols(_) => {
                          self.block_complexity_factor = base_complexity;
                     }
                }
            }
            WavePhase::Relief => {
                self.block_complexity_factor = base_complexity * 0.4; // 缓和阶段复杂度更低
            }
        }
        log_debug!(
            "Difficulty updated: Blocks={}, Complexity={:.2}", 
            self.blocks_per_generation, // 仍然记录，值为3
            self.block_complexity_factor
        );
    }

    /// 选择下一个挑战类型
    fn select_next_challenge(&self) -> ChallengeType {
        let cycle_len = self.accumulation_turns + self.challenge_incoming_turns + self.challenge_active_turns + self.relief_turns;
        if cycle_len == 0 { return ChallengeType::BlockFlood; } 
        // 更新: 现在只有两种类型轮流 (0 和 1)
        let challenge_index = (self.turn_count / cycle_len) % 2; 

        match challenge_index {
            0 => ChallengeType::BlockFlood,
            _ => { // 对应之前的 1 和 2 (现在是 1)
                 let target_count = random::gen_range(1, 3) as u8;
                 if random::gen_range(0, 2) == 0 { 
                     ChallengeType::TargetRows(target_count)
                 } else {
                     ChallengeType::TargetCols(target_count)
                 }
            }
        }
    }
    
    /// 开始挑战时的设置逻辑
    fn start_challenge(&mut self, challenge_type: ChallengeType) {
         log_info!("Starting challenge: {:?}", challenge_type);
         
         // 清理可能残留的状态
         self.active_target_lines.clear();
         self.target_lines_cleared_count = 0;
         self.required_targets_for_success = 0; // 重置

         match challenge_type {
             ChallengeType::TargetRows(count) | ChallengeType::TargetCols(count) => {
                 self.required_targets_for_success = count; // 记录需要完成的目标数
                 // 随机选择目标行/列索引
                 let mut available_lines: Vec<usize> = (0..8).collect();
                 for _ in 0..count { // 使用参数 count
                     if available_lines.is_empty() { break; }
                     let random_index = random::gen_range(0, available_lines.len() as i32) as usize;
                     self.active_target_lines.push(available_lines.remove(random_index));
                 }
                 log_info!("Target lines challenge. Targets: {:?}, Required: {}", self.active_target_lines, self.required_targets_for_success);
             }
             ChallengeType::BlockFlood => {} // BlockFlood 可能不需要特殊设置
         }
    }

    /// 挑战结束时的清理逻辑，并计算奖励
    fn end_challenge(&mut self) {
         log_info!("Ending challenge (current phase was: {:?})", self.current_phase);
         if let WavePhase::ChallengeActive(challenge_type) = self.current_phase {
             match challenge_type {
                  ChallengeType::TargetRows(_) | ChallengeType::TargetCols(_) => {
                      if self.target_lines_cleared_count >= self.required_targets_for_success as u32 && self.required_targets_for_success > 0 {
                          log_info!("Target line challenge SUCCESSFUL! Cleared {}/{} targets.", self.target_lines_cleared_count, self.required_targets_for_success);
                          self.pending_score_bonus += SCORE_BONUS_TARGET_CHALLENGE_COMPLETION; // 额外完成奖
                          log_info!("Awarding completion bonus: {}. Total pending: {}", SCORE_BONUS_TARGET_CHALLENGE_COMPLETION, self.pending_score_bonus);
                      } else {
                          log_info!("Target line challenge FAILED. Cleared {}/{} targets.", self.target_lines_cleared_count, self.required_targets_for_success);
                      }
                  }
                  ChallengeType::BlockFlood => {
                      log_info!("BlockFlood challenge survived!");
                      self.pending_score_bonus += SCORE_BONUS_BLOCK_FLOOD_SURVIVAL;
                      log_info!("Awarding survival bonus: {}. Total pending: {}", SCORE_BONUS_BLOCK_FLOOD_SURVIVAL, self.pending_score_bonus);
                  }
             }
         }
    }

    /// 通知管理器某行/列已被清除
    pub fn notify_line_cleared(&mut self, index: usize, is_row: bool) {
         if let WavePhase::ChallengeActive(challenge_type) = self.current_phase {
             let target_match = match challenge_type {
                 ChallengeType::TargetRows(_) => is_row,
                 ChallengeType::TargetCols(_) => !is_row,
                 _ => false,
             };

             if target_match && self.active_target_lines.contains(&index) {
                 // 防止重复计数
                 if let Some(pos) = self.active_target_lines.iter().position(|&x| x == index) {
                      self.active_target_lines.remove(pos); // 移除已完成的目标
                      self.target_lines_cleared_count += 1;
                      self.pending_score_bonus += SCORE_BONUS_PER_TARGET_LINE; // 即时奖励
                      log_info!(
                          "Target line {} cleared! ({}/{}) Pending bonus: {}. Total pending: {}", 
                          index, self.target_lines_cleared_count, self.required_targets_for_success, 
                          SCORE_BONUS_PER_TARGET_LINE, self.pending_score_bonus
                      );
                      // TODO: 可以在此触发单行/列完成的即时奖励
                      // TODO: Check if all targets are cleared to mark challenge success earlier?
                 }
             }
         }
    }
    
    /// 获取当前激活的目标行/列 (用于绘制高亮)
    pub fn get_active_target_lines(&self) -> &Vec<usize> {
        &self.active_target_lines
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
} 
 