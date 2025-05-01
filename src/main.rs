use macroquad::prelude::*;

const W: f32 = 20.0;
const ROWS: usize = 20;
const COLS: usize = 20;

#[macroquad::main("Snake Game")]
async fn main() {
    loop {
        clear_background(WHITE);

        for y in 0..ROWS {
            for x in 0..COLS {
                let color = if (x + y) % 2 == 0 {
                    LIGHTGRAY
                } else {
                    WHITE
                };
                draw_rectangle(x as f32 * W, y as f32 * W, W, W, color);
            }
        }

        next_frame().await;
    }
}