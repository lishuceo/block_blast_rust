# 连通区域分析改进

## 背景问题

原有系统使用简单的填充率（filled_ratio）来判断是否需要为玩家提供帮助。这种方法存在明显缺陷：

- 即使填充率不高，如果空白格子被分割成多个孤立的小区域，玩家仍然会处于困难境地
- 无法识别狭长的空白区域（难以放置方块）和方正的空白区域（容易放置方块）的区别
- 可能导致玩家在看似空旷但实际难以操作的局面下突然失败

## 解决方案

实现了基于连通区域分析的困难度评估系统：

### 1. 连通区域分析 (`Grid::analyze_connected_empty_regions`)

- 使用洪水填充算法找出所有连通的空白区域
- 返回按大小排序的 `RegionInfo` 数组
- 每个区域包含详细信息：大小、位置、形状分数等

### 2. 形状评分系统 (`RegionInfo::shape_score`)

- 综合考虑长宽比和填充率
- 分数越接近 1.0 表示区域越方正、紧凑
- 用于区分狭长区域（难放置）和方正区域（易放置）

### 3. 困难度评估 (`Grid::get_difficulty_score`)

基于四个维度评估游戏困难度（0.0-1.0）：

1. **最大连通区域大小**：最大区域越小，困难度越高
2. **区域形状质量**：狭长形状增加困难度
3. **碎片化程度**：区域数量越多，说明空间越碎片化，困难度越高
4. **大块放置能力**：是否有能容纳 4x4 方块的区域

### 4. 改进的帮助机制 (`WaveManager::should_offer_helpful_block_v2`)

- 基于困难度分数而非填充率决定是否提供帮助
- 不同游戏阶段有不同的帮助概率曲线：
  - **缓和阶段**：最容易获得帮助
  - **积累阶段**：中等帮助概率
  - **挑战阶段**：较少帮助，但极度困难时仍会支援

## 测试验证

实现了三个单元测试验证功能正确性：

1. `test_connected_regions_simple`：测试基本的区域识别和分析
2. `test_difficulty_score_extreme_cases`：测试极端情况的困难度评分
3. `test_fragmented_regions`：测试高度碎片化场景

## 使用示例

```rust
// 在生成方块时使用新的困难度分析
let difficulty_score = self.grid.get_difficulty_score();
let offer_help = self.wave_manager.should_offer_helpful_block_v2(difficulty_score);

// 获取详细的区域信息用于调试
let regions = self.grid.analyze_connected_empty_regions();
log_info!("Found {} regions, largest has {} cells", regions.len(), 
          regions.first().map(|r| r.cell_count).unwrap_or(0));
```

## 优势

1. **更准确的困难度判断**：能识别空间碎片化等复杂情况
2. **更智能的帮助时机**：在真正需要时提供帮助，避免游戏过于简单
3. **可扩展性**：易于添加新的评估维度或调整权重
4. **调试友好**：提供详细的区域信息，便于分析游戏状态 