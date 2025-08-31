use std::time::{Duration, Instant};

/// 时间窗口结构
#[derive(Debug, Clone)]
pub struct TimeWindow {
    /// 窗口开始时间
    pub start_time: Instant,
    /// 窗口结束时间  
    pub end_time: Instant,
    /// 窗口内的事件计数
    pub event_count: u32,
}

impl TimeWindow {
    /// 创建新的时间窗口
    pub fn new(start_time: Instant, duration: Duration) -> Self {
        Self {
            start_time,
            end_time: start_time + duration,
            event_count: 0,
        }
    }

    /// 检查时间是否在窗口内
    pub fn contains(&self, time: Instant) -> bool {
        time >= self.start_time && time < self.end_time
    }

    /// 重置窗口到新的时间范围
    pub fn reset(&mut self, start_time: Instant, duration: Duration) {
        self.start_time = start_time;
        self.end_time = start_time + duration;
        self.event_count = 0;
    }

    /// 添加事件到窗口
    pub fn add_event(&mut self) {
        self.event_count += 1;
    }
}

/// 基于时间窗口的频率计算器
/// 实现用户需求：2秒时间窗口，500ms更新频率，4个窗口轮换
#[derive(Debug)]
pub struct FrequencyCalculator {
    /// 4个500ms的时间窗口
    time_windows: [TimeWindow; 4],
    /// 当前活跃窗口索引
    current_window_index: usize,
    /// 上次更新时间
    last_update_time: Instant,
    /// 窗口持续时间（500ms）
    window_duration: Duration,
    /// 更新间隔（500ms）
    update_interval: Duration,
    /// 总时间窗口大小（2秒）
    total_window_duration: Duration,
    /// 上次轮换时间
    last_rotation_time: Instant,
}

impl FrequencyCalculator {
    /// 创建新的频率计算器
    pub fn new() -> Self {
        let now = Instant::now();
        let window_duration = Duration::from_millis(500);
        let update_interval = Duration::from_millis(500);
        let total_window_duration = Duration::from_millis(2000);

        // 初始化4个时间窗口，从当前时间开始
        let time_windows = [
            TimeWindow::new(now, window_duration), // 当前窗口
            TimeWindow::new(now - Duration::from_millis(500), window_duration), // -500ms
            TimeWindow::new(now - Duration::from_millis(1000), window_duration), // -1000ms
            TimeWindow::new(now - Duration::from_millis(1500), window_duration), // -1500ms
        ];

        Self {
            time_windows,
            current_window_index: 0, // 从当前窗口开始
            last_update_time: now,
            window_duration,
            update_interval,
            total_window_duration,
            last_rotation_time: now,
        }
    }

    /// 添加事件到当前时间窗口
    pub fn add_event(&mut self) {
        let now = Instant::now();

        // 检查是否需要轮换窗口
        self.update_windows_if_needed(now);

        // 将事件添加到当前窗口
        self.time_windows[self.current_window_index].add_event();
    }

    /// 计算当前频率（Hz）
    pub fn calculate_frequency(&self) -> f64 {
        let total_events: u32 = self.time_windows.iter().map(|w| w.event_count).sum();

        if total_events == 0 {
            return 0.0;
        }

        // 计算频率：总事件数 / 总时间窗口（秒）
        let total_seconds = self.total_window_duration.as_secs_f64();
        let frequency = total_events as f64 / total_seconds;

        // 保留1位小数
        (frequency * 10.0).round() / 10.0
    }

    /// 检查是否需要发送频率更新
    pub fn should_send_update(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_update_time) >= self.update_interval
    }

    /// 标记已发送更新
    pub fn mark_update_sent(&mut self) {
        self.last_update_time = Instant::now();
    }

    /// 重置计算器（用于模式切换等场景）
    pub fn reset(&mut self) {
        let now = Instant::now();

        // 重置所有窗口 - 修复：正确设置窗口时间
        for (i, window) in self.time_windows.iter_mut().enumerate() {
            let start_time = now - Duration::from_millis(i as u64 * 500);
            window.reset(start_time, self.window_duration);
        }

        self.current_window_index = 0; // 从当前窗口开始
        self.last_update_time = now;
        self.last_rotation_time = now;
    }

    /// 更新时间窗口（如果需要的话）
    fn update_windows_if_needed(&mut self, now: Instant) {
        // 检查当前窗口是否已过期
        let current_window = &self.time_windows[self.current_window_index];

        if now >= current_window.end_time {
            // 需要轮换到下一个窗口
            self.rotate_windows(now);
        }
    }

    /// 轮换时间窗口
    fn rotate_windows(&mut self, now: Instant) {
        // 计算需要轮换多少个窗口
        let current_end_time = self.time_windows[self.current_window_index].end_time;
        let elapsed = now.duration_since(current_end_time);
        let windows_to_rotate = (elapsed.as_millis() / 500).max(1) as usize;

        for i in 0..windows_to_rotate {
            // 移动到下一个窗口索引
            self.current_window_index = (self.current_window_index + 1) % 4;

            // 重置新的当前窗口 - 修复：新窗口应该从当前时间开始
            let window_start = current_end_time + Duration::from_millis(i as u64 * 500);
            self.time_windows[self.current_window_index].reset(window_start, self.window_duration);
        }

        self.last_rotation_time = now;
    }

    /// 获取调试信息
    pub fn get_debug_info(&self) -> String {
        let total_events: u32 = self.time_windows.iter().map(|w| w.event_count).sum();
        let frequency = self.calculate_frequency();

        format!(
            "FrequencyCalculator: current_window={}, total_events={}, frequency={}Hz, windows=[{}, {}, {}, {}]",
            self.current_window_index,
            total_events,
            frequency,
            self.time_windows[0].event_count,
            self.time_windows[1].event_count,
            self.time_windows[2].event_count,
            self.time_windows[3].event_count,
        )
    }
}

impl Default for FrequencyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_frequency_calculator_basic() {
        let mut calc = FrequencyCalculator::new();

        // 添加一些事件
        for _ in 0..10 {
            calc.add_event();
        }

        let frequency = calc.calculate_frequency();
        assert!(frequency > 0.0);
        println!("Basic test frequency: {}", frequency);
    }

    #[test]
    fn test_frequency_calculator_time_windows() {
        let mut calc = FrequencyCalculator::new();

        // 在短时间内添加事件
        for _ in 0..30 {
            calc.add_event();
            thread::sleep(Duration::from_millis(10));
        }

        let frequency = calc.calculate_frequency();
        println!("Time window test frequency: {}", frequency);
        println!("Debug info: {}", calc.get_debug_info());

        // 频率应该大于0
        assert!(frequency > 0.0);
    }

    #[test]
    fn test_frequency_calculator_reset() {
        let mut calc = FrequencyCalculator::new();

        // 添加事件
        for _ in 0..5 {
            calc.add_event();
        }

        let frequency_before = calc.calculate_frequency();
        assert!(frequency_before > 0.0);

        // 重置
        calc.reset();
        let frequency_after = calc.calculate_frequency();
        assert_eq!(frequency_after, 0.0);
    }
}
