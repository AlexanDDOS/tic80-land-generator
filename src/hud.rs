use crate::tic80::*;
use crate::print;

/// TIC-80 styled notifier
#[derive(Default)]
pub struct Notifier {
    msg: String,
    timer: i32,
    timer_start: i32
}

/// Duration of notifier animations
const NOTIFIER_ANIM_DURATION: f32 = 30.0;

impl Notifier {
    pub fn notify(&mut self, msg: &str, time: i32) {
        self.msg = msg.to_string();
        self.timer_start = time;
        self.timer = time;
    }

    pub fn draw(&self) {
        if self.timer > 0 {
            let elapsed = (self.timer_start - self.timer) as f32;
            let left = self.timer as f32;
            let mut y = 0;
            if elapsed < NOTIFIER_ANIM_DURATION {
                y -= (8.0 * (1.0 - elapsed / NOTIFIER_ANIM_DURATION)) as i32;
            } else if left < NOTIFIER_ANIM_DURATION {
                y -= (8.0 * (1.0 - left / NOTIFIER_ANIM_DURATION)) as i32;
            }
            let text_w = print!(self.msg.clone(), 0, -6, PrintOptions::default());
            rect(0, y, 240, 8, 2);
            print!(self.msg.clone(), (240 - text_w) / 2, y + 1,
                PrintOptions{color: 12, ..PrintOptions::default()});
        }
    }

    /// Count down the timer by one frame
    pub fn countdown(&mut self) {
        self.timer = std::cmp::max(self.timer - 1, 0);
    }
}