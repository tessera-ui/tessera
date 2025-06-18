//! 定义布局组件的对齐方式

/// 主轴对齐方式（沿着布局方向的对齐）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    /// 起始位置对齐（左对齐或顶对齐）
    Start,
    /// 居中对齐
    Center,
    /// 结束位置对齐（右对齐或底对齐）
    End,
    /// 均匀分布，首尾留白
    SpaceEvenly,
    /// 均匀分布，首尾不留白
    SpaceBetween,
    /// 均匀分布，首尾留半白
    SpaceAround,
}

impl Default for MainAxisAlignment {
    fn default() -> Self {
        Self::Start
    }
}

/// 交叉轴对齐方式（垂直于布局方向的对齐）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    /// 起始位置对齐（左对齐或顶对齐）
    Start,
    /// 居中对齐
    Center,
    /// 结束位置对齐（右对齐或底对齐）
    End,
    /// 拉伸填充整个交叉轴
    Stretch,
}

impl Default for CrossAxisAlignment {
    fn default() -> Self {
        Self::Start
    }
}
