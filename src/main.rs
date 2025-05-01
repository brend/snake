use macroquad::prelude::*;

const W: f32 = 20.0;
const ROWS: usize = 20;
const COLS: usize = 20;

fn random_position() -> Vec2 {
    let x = rand::gen_range(0, ROWS) as f32 * W;
    let y = rand::gen_range(0, COLS) as f32 * W;
    Vec2::new(x, y)
}

#[macroquad::main("Snake Game")]
async fn main() {
    let mut food = random_position();

    loop {
        // Check for input to change food position
        if is_key_pressed(KeyCode::Space) {
            food.x = rand::gen_range(0, ROWS) as f32 * W;
            food.y = rand::gen_range(0, COLS) as f32 * W;
        }

        // Clear the screen
        clear_background(WHITE);

        // Draw the grid
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

        // Draw the food
        draw_rectangle(food.x.floor(), food.y.floor(), W, W, GREEN);

        next_frame().await;
    }
}