use raylib::{color::Color, prelude::RaylibDraw};
mod game_of_life;
//use game_of_life;
fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1920, 1080)
        .resizable()
        .title("Hello")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::GOLD);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
        println!("{}", d.get_fps());
    }
}
