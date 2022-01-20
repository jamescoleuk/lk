use termion::color::{self, Rgb};

const BLUE: Rgb = Rgb(113, 190, 242);
const DARK_BLUE: Rgb = Rgb(21, 147, 232);
const DARK_GREEN: Rgb = Rgb(141, 188, 105);
const DARK_GREY: Rgb = Rgb(50, 50, 50);
const GREEN: Rgb = Rgb(168, 204, 140);
const GREY: Rgb = Rgb(185, 191, 202);
const ORANGE: Rgb = Rgb(219, 171, 121);
const PINK: Rgb = Rgb(210, 144, 228);
const RED: Rgb = Rgb(232, 131, 136);
const TEAL: Rgb = Rgb(102, 194, 205);

pub const BLUE_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(BLUE);
pub const BLUE_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(BLUE);
pub const DARK_BLUE_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(DARK_BLUE);
pub const DARK_BLUE_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(DARK_BLUE);
pub const DARK_GREEN_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(DARK_GREEN);
pub const DARK_GREEN_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(DARK_GREEN);
pub const DARK_GREY_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(DARK_GREY);
pub const DARK_GREY_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(DARK_GREY);
pub const GREEN_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(GREEN);
pub const GREEN_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(GREEN);
pub const GREY_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(GREY);
pub const GREY_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(GREY);
pub const ORANGE_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(ORANGE);
pub const ORANGE_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(ORANGE);
pub const PINK_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(PINK);
pub const PINK_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(PINK);
pub const RED_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(RED);
pub const RED_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(RED);
pub const TEAL_BG: termion::color::Bg<termion::color::Rgb> = color::Bg(TEAL);
pub const TEAL_FG: termion::color::Fg<termion::color::Rgb> = color::Fg(TEAL);

pub const RESET_BG: termion::color::Bg<termion::color::Reset> = color::Bg(color::Reset);
pub const RESET_FG: termion::color::Fg<termion::color::Reset> = color::Fg(color::Reset);
