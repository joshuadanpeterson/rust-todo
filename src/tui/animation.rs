// src/tui/animation.rs - Animation utilities for smooth UI transitions
// Provides spinner, progress indicators, and transition effects

use std::time::Instant;

/// Spinner animation for loading states
pub struct Spinner {
    frames: Vec<&'static str>,
    current_frame: usize,
    last_update: Instant,
    frame_duration_ms: u64,
}

impl Spinner {
    /// Create a modern spinner
    pub fn modern() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            last_update: Instant::now(),
            frame_duration_ms: 80,
        }
    }
    
    /// Create a dots spinner
    pub fn dots() -> Self {
        Self {
            frames: vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            current_frame: 0,
            last_update: Instant::now(),
            frame_duration_ms: 100,
        }
    }
    
    /// Create a circular spinner
    pub fn circle() -> Self {
        Self {
            frames: vec!["◐", "◓", "◑", "◒"],
            current_frame: 0,
            last_update: Instant::now(),
            frame_duration_ms: 120,
        }
    }
    
    /// Get the current frame and advance if needed
    pub fn tick(&mut self) -> &str {
        let elapsed = self.last_update.elapsed().as_millis() as u64;
        if elapsed >= self.frame_duration_ms {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = Instant::now();
        }
        self.frames[self.current_frame]
    }
}

/// Progress bar characters for smooth transitions
pub struct ProgressBar;

impl ProgressBar {
    /// Get a smooth progress bar string
    pub fn render(progress: f64, width: usize) -> String {
        let filled = (progress * width as f64) as usize;
        let partial = ((progress * width as f64) - filled as f64) * 8.0;
        
        let partial_char = match partial as usize {
            0 => ' ',
            1 => '▏',
            2 => '▎',
            3 => '▍',
            4 => '▌',
            5 => '▋',
            6 => '▊',
            7 => '▉',
            _ => '█',
        };
        
        let mut bar = String::new();
        for i in 0..width {
            if i < filled {
                bar.push('█');
            } else if i == filled && partial > 0.0 {
                bar.push(partial_char);
            } else {
                bar.push('░');
            }
        }
        
        bar
    }
}

/// Smooth scroll indicator
pub struct ScrollIndicator;

impl ScrollIndicator {
    /// Get scroll position indicator
    pub fn render(current: usize, total: usize, height: usize) -> Vec<String> {
        if total <= height {
            return vec![];
        }
        
        let ratio = current as f64 / (total - 1) as f64;
        let indicator_pos = (ratio * (height - 1) as f64) as usize;
        
        let mut indicators = Vec::new();
        for i in 0..height {
            if i == indicator_pos {
                indicators.push("◆".to_string());
            } else {
                indicators.push("│".to_string());
            }
        }
        
        indicators
    }
    
    /// Get a mini scroll bar
    pub fn mini(current: usize, total: usize) -> String {
        if total == 0 {
            return String::new();
        }
        
        let blocks = vec!["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
        let ratio = current as f64 / (total - 1).max(1) as f64;
        let index = (ratio * (blocks.len() - 1) as f64) as usize;
        
        blocks[index].to_string()
    }
}

/// Transition effects for smooth UI changes
pub struct Transition {
    start_time: Instant,
    duration_ms: u64,
}

impl Transition {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            start_time: Instant::now(),
            duration_ms,
        }
    }
    
    /// Get the current progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        (elapsed as f64 / self.duration_ms as f64).min(1.0)
    }
    
    /// Check if transition is complete
    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }
    
    /// Apply easing function for smooth animation
    pub fn ease_in_out(&self) -> f64 {
        let t = self.progress();
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
}
