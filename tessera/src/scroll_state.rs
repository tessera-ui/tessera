use crate::{cursor::ScrollEventType, px::{Px, PxPosition}};

/// 滚动状态管理器
#[derive(Debug, Clone)]
pub struct ScrollState {
    /// 当前滚动偏移量（支持负值）
    pub offset: PxPosition,
    /// 内容区域大小
    pub content_size: PxPosition,
    /// 视口大小
    pub viewport_size: PxPosition,
    /// 滚动速度（用于惯性滚动）
    pub velocity: [f32; 2],
    /// 滚动边界
    pub bounds: ScrollBounds,
    /// 滚动灵敏度
    pub scroll_sensitivity: f32,
}

/// 滚动边界定义
#[derive(Debug, Clone)]
pub struct ScrollBounds {
    /// 最小滚动位置（通常为负值）
    pub min: PxPosition,
    /// 最大滚动位置（通常为 [0, 0]）
    pub max: PxPosition,
}

impl ScrollState {
    /// 创建新的滚动状态
    pub fn new(viewport_size: PxPosition, content_size: PxPosition) -> Self {
        let bounds = ScrollBounds::from_sizes(viewport_size, content_size);
        Self {
            offset: PxPosition::ZERO,
            content_size,
            viewport_size,
            velocity: [0.0, 0.0],
            bounds,
            scroll_sensitivity: 50.0, // 默认滚动灵敏度
        }
    }
    
    /// 创建具有自定义滚动灵敏度的滚动状态
    pub fn with_sensitivity(viewport_size: PxPosition, content_size: PxPosition, sensitivity: f32) -> Self {
        let mut state = Self::new(viewport_size, content_size);
        state.scroll_sensitivity = sensitivity;
        state
    }
    
    /// 处理滚动事件
    pub fn handle_scroll_event(&mut self, event: &ScrollEventType) {
        // 转换滚动增量为像素，考虑滚动灵敏度
        let delta_x = Px((event.delta_x * self.scroll_sensitivity) as i32);
        let delta_y = Px((event.delta_y * self.scroll_sensitivity) as i32);
        
        // 更新偏移量并应用边界约束
        self.offset.x = Px((self.offset.x.0 - delta_x.0).clamp(
            self.bounds.min.x.0,
            self.bounds.max.x.0
        ));
        self.offset.y = Px((self.offset.y.0 - delta_y.0).clamp(
            self.bounds.min.y.0,
            self.bounds.max.y.0
        ));
        
        // 更新速度（用于后续的惯性滚动）
        self.velocity[0] = -event.delta_x;
        self.velocity[1] = -event.delta_y;
    }
    
    /// 更新视口大小
    pub fn update_viewport_size(&mut self, viewport_size: PxPosition) {
        self.viewport_size = viewport_size;
        self.bounds = ScrollBounds::from_sizes(viewport_size, self.content_size);
        self.clamp_offset_to_bounds();
    }
    
    /// 更新内容大小并重新计算边界
    pub fn update_content_size(&mut self, content_size: PxPosition) {
        self.content_size = content_size;
        self.bounds = ScrollBounds::from_sizes(self.viewport_size, content_size);
        self.clamp_offset_to_bounds();
    }
    
    /// 将偏移量约束在边界内
    pub fn clamp_offset_to_bounds(&mut self) {
        self.offset.x = Px(self.offset.x.0.clamp(
            self.bounds.min.x.0,
            self.bounds.max.x.0
        ));
        self.offset.y = Px(self.offset.y.0.clamp(
            self.bounds.min.y.0,
            self.bounds.max.y.0
        ));
    }
    
    /// 获取滚动百分比 (0.0 到 1.0)
    pub fn scroll_percentage(&self) -> [f32; 2] {
        let x_range = (self.bounds.max.x.0 - self.bounds.min.x.0) as f32;
        let y_range = (self.bounds.max.y.0 - self.bounds.min.y.0) as f32;
        
        let x_percent = if x_range > 0.0 {
            (self.offset.x.0 - self.bounds.min.x.0) as f32 / x_range
        } else {
            0.0
        };
        
        let y_percent = if y_range > 0.0 {
            (self.offset.y.0 - self.bounds.min.y.0) as f32 / y_range
        } else {
            0.0
        };
        
        [x_percent, y_percent]
    }
    
    /// 设置滚动位置到指定百分比
    pub fn set_scroll_percentage(&mut self, x_percent: f32, y_percent: f32) {
        let x_range = (self.bounds.max.x.0 - self.bounds.min.x.0) as f32;
        let y_range = (self.bounds.max.y.0 - self.bounds.min.y.0) as f32;
        
        self.offset.x = Px(self.bounds.min.x.0 + (x_percent.clamp(0.0, 1.0) * x_range) as i32);
        self.offset.y = Px(self.bounds.min.y.0 + (y_percent.clamp(0.0, 1.0) * y_range) as i32);
    }
    
    /// 滚动到指定位置
    pub fn scroll_to(&mut self, position: PxPosition) {
        self.offset = position;
        self.clamp_offset_to_bounds();
    }
    
    /// 相对滚动
    pub fn scroll_by(&mut self, delta: PxPosition) {
        self.offset = self.offset + delta;
        self.clamp_offset_to_bounds();
    }
    
    /// 检查是否可以向指定方向滚动
    pub fn can_scroll(&self, direction: ScrollDirection) -> bool {
        match direction {
            ScrollDirection::Up => self.offset.y.0 > self.bounds.min.y.0,
            ScrollDirection::Down => self.offset.y.0 < self.bounds.max.y.0,
            ScrollDirection::Left => self.offset.x.0 > self.bounds.min.x.0,
            ScrollDirection::Right => self.offset.x.0 < self.bounds.max.x.0,
        }
    }
    
    /// 获取内容在视口中的可见区域
    pub fn visible_content_rect(&self) -> ContentRect {
        ContentRect {
            x: -self.offset.x.0,
            y: -self.offset.y.0,
            width: self.viewport_size.x.0,
            height: self.viewport_size.y.0,
        }
    }
}

impl ScrollBounds {
    /// 根据视口和内容大小计算边界
    pub fn from_sizes(viewport_size: PxPosition, content_size: PxPosition) -> Self {
        let min_x = if content_size.x.0 > viewport_size.x.0 {
            viewport_size.x.0 - content_size.x.0
        } else {
            0
        };
        let min_y = if content_size.y.0 > viewport_size.y.0 {
            viewport_size.y.0 - content_size.y.0
        } else {
            0
        };
        
        Self {
            min: PxPosition::new(Px(min_x), Px(min_y)),
            max: PxPosition::ZERO,
        }
    }
    
    /// 检查位置是否在边界内
    pub fn contains(&self, position: PxPosition) -> bool {
        position.x.0 >= self.min.x.0 && position.x.0 <= self.max.x.0 &&
        position.y.0 >= self.min.y.0 && position.y.0 <= self.max.y.0
    }
}

/// 滚动方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// 内容区域矩形（用于视口裁剪）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ContentRect {
    /// 检查点是否在矩形内
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
    
    /// 检查另一个矩形是否与此矩形相交
    pub fn intersects(&self, other: &ContentRect) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
    
    /// 计算与另一个矩形的交集
    pub fn intersection(&self, other: &ContentRect) -> Option<ContentRect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        
        if left < right && top < bottom {
            Some(ContentRect {
                x: left,
                y: top,
                width: right - left,
                height: bottom - top,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state_creation() {
        let viewport = PxPosition::new(Px(400), Px(300));
        let content = PxPosition::new(Px(1000), Px(800));
        let state = ScrollState::new(viewport, content);
        
        assert_eq!(state.offset, PxPosition::ZERO);
        assert_eq!(state.viewport_size, viewport);
        assert_eq!(state.content_size, content);
        
        // 边界应该允许向左和向上滚动
        assert_eq!(state.bounds.min, PxPosition::new(Px(-600), Px(-500)));
        assert_eq!(state.bounds.max, PxPosition::ZERO);
    }

    #[test]
    fn test_scroll_bounds() {
        let viewport = PxPosition::new(Px(400), Px(300));
        let content = PxPosition::new(Px(200), Px(150)); // 内容小于视口
        let bounds = ScrollBounds::from_sizes(viewport, content);
        
        // 内容小于视口时，不应该滚动
        assert_eq!(bounds.min, PxPosition::ZERO);
        assert_eq!(bounds.max, PxPosition::ZERO);
    }

    #[test]
    fn test_scroll_event_handling() {
        let mut state = ScrollState::new(
            PxPosition::new(Px(400), Px(300)),
            PxPosition::new(Px(1000), Px(800))
        );
        
        let scroll_event = ScrollEventType {
            delta_x: 1.0,
            delta_y: 1.0,
        };
        
        state.handle_scroll_event(&scroll_event);
        
        // 滚动后偏移量应该改变
        assert_ne!(state.offset, PxPosition::ZERO);
        assert!(state.offset.x.0 < 0);
        assert!(state.offset.y.0 < 0);
    }

    #[test]
    fn test_scroll_percentage() {
        let mut state = ScrollState::new(
            PxPosition::new(Px(400), Px(300)),
            PxPosition::new(Px(1000), Px(800))
        );
        
        // 滚动到中间位置
        state.scroll_to(PxPosition::new(Px(-300), Px(-250)));
        
        let percent = state.scroll_percentage();
        assert!((percent[0] - 0.5).abs() < 0.01);
        assert!((percent[1] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_content_rect() {
        let rect1 = ContentRect { x: 10, y: 10, width: 100, height: 100 };
        let rect2 = ContentRect { x: 50, y: 50, width: 100, height: 100 };
        
        assert!(rect1.intersects(&rect2));
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 50);
        assert_eq!(intersection.y, 50);
        assert_eq!(intersection.width, 60);
        assert_eq!(intersection.height, 60);
    }
}