use std::vec;

use macroquad::prelude::*;

const W: f32 = 20.0;
const ROWS: usize = 20;
const COLS: usize = 20;
const INITIAL_SPEED: f32 = 0.5;

fn random_position() -> Vec2 {
    let x = rand::gen_range(0, ROWS) as f32 * W;
    let y = rand::gen_range(0, COLS) as f32 * W;
    Vec2::new(x, y)
}

fn grid(x: f32) -> f32 {
    (x / W).floor() * W
}

struct Snake {
    body: Vec<Vec2>,
    velocity: Vec2,
    speed: f32,
}

impl Snake {
    fn new() -> Self {
        Self {
            body: vec![random_position()],
            velocity: Vec2::new(0.01, 0.0),
            speed: INITIAL_SPEED,
        }
    }

    fn update(&mut self) {
        // Update the snake's position based on its velocity
        for i in (1..self.body.len()).rev() {
            self.body[i] = self.body[i - 1];
        }
        self.body[0] += self.velocity;
    }

    fn draw(&self) {
        for segment in &self.body {
            draw_rectangle(grid(segment.x), grid(segment.y), W, W, BLACK);
        }
    }
}

#[macroquad::main("Snake Game")]
async fn main() {
    let mut food = Vec2::new(W * 10.0, W * 10.0);
    let mut snake = Snake::new();

    loop {
        // Check for input to change snake position
        if is_key_pressed(KeyCode::Up) && snake.velocity.x != 0.0 {
            snake.velocity = Vec2::new(0.0, -snake.speed);
        } else if is_key_pressed(KeyCode::Down) && snake.velocity.x != 0.0 {
            snake.velocity = Vec2::new(0.0, snake.speed);
        } else if is_key_pressed(KeyCode::Left) && snake.velocity.y != 0.0 {
            snake.velocity = Vec2::new(-snake.speed, 0.0);
        } else if is_key_pressed(KeyCode::Right) && snake.velocity.y != 0.0 {
            snake.velocity = Vec2::new(snake.speed, 0.0);
        }

        // Draw the grid
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

        // Draw the food
        draw_rectangle(food.x.floor(), food.y.floor(), W, W, GREEN);

        // Draw the snake
        snake.update();
        snake.draw();
        
        next_frame().await;
    }
}