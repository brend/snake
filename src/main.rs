use std::vec;

use macroquad::prelude::*;

const W: f32 = 30.0;
const ROWS: usize = 20;
const COLS: usize = 20;
const INITIAL_SPEED: f32 = 0.5;

fn random_position() -> Vec2 {
    let x = rand::gen_range(0, ROWS) as f32 * W;
    let y = rand::gen_range(0, COLS) as f32 * W;
    Vec2::new(x, y)
}

fn snap(x: f32) -> f32 {
    (x / W).floor() * W
}

fn snapv(v: &Vec2) -> Vec2 {
    Vec2::new(snap(v.x), snap(v.y))
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

    fn head(&self) -> Vec2 {
        self.body[0]
    }

    fn grow(&mut self) {
        // Add a new segment to the snake's body
        let last_segment = *self.body.last().unwrap();
        self.body.push(last_segment);
    }

    fn speed_up(&mut self) {
        // Increase the snake's speed
        self.speed += 0.1;
        self.velocity = self.velocity.normalize() * self.speed;
    }

    fn update(&mut self) -> bool {
        // Move the snake by updating the position of the head
        let head_on_grid = snapv(&self.head());
        self.body[0] += self.velocity;
        if head_on_grid != snapv(&self.head()) {
            // If the head has moved to a new grid cell, update the body
            for i in (1..self.body.len()).rev() {
                self.body[i] = self.body[i - 1];
            }
            return true;
        }
        false
    }

    fn draw(&self) {
        for segment in &self.body {
            draw_rectangle(snap(segment.x), snap(segment.y), W, W, BLACK);
        }
    }

    fn eat(&mut self, food: &Vec2) -> bool {
        // Check if the snake's head is on the food
        if snapv(&self.head()) == snapv(food) {
            // If so, grow the snake and return true
            self.grow();
            self.speed_up();
            return true;
        }
        false
    }
}

#[derive(PartialEq)]
enum GameState {
    Running,
    GameOver,
}

fn handle_input(snake: &mut Snake) {
            // Check for input to change snake position
            if is_key_pressed(KeyCode::Up) && snake.velocity.x != 0.0 {
                snake.velocity = Vec2::new(0.0, -snake.speed);
            } else if is_key_pressed(KeyCode::Down) && snake.velocity.x != 0.0 {
                snake.velocity = Vec2::new(0.0, snake.speed);
            } else if is_key_pressed(KeyCode::Left) && snake.velocity.y != 0.0 {
                snake.velocity = Vec2::new(-snake.speed, 0.0);
            } else if is_key_pressed(KeyCode::Right) && snake.velocity.y != 0.0 {
                snake.velocity = Vec2::new(snake.speed, 0.0);
            } else if is_key_pressed(KeyCode::Space) {
                snake.grow();
                snake.speed_up();
            }    
}

#[macroquad::main("Snake Game")]
async fn main() {
    let mut food = Vec2::new(W * 10.0, W * 10.0);
    let mut snake = Snake::new();
    let mut state = GameState::Running;

    loop {
        match state {
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) {
                    // Restart the game
                    snake = Snake::new();
                    food = random_position();
                    state = GameState::Running;
                }
            }
            _ => {
                handle_input(&mut snake);
                if snake.update() {
                    // Check for collision with itself
                    for i in 2..snake.body.len() {
                        if snapv(&snake.head()) == snapv(&snake.body[i]) {
                            println!("Collision with segment {}", i);
                            // Game over
                            state = GameState::GameOver;
                            break;
                        }
                    }
                }

                // Check for collision with food
                if snake.eat(&food) {
                    food = random_position();
                }

                // Check for collision with walls
                if snake.head().x < 0.0 || snake.head().x >= W * COLS as f32 ||
                snake.head().y < 0.0 || snake.head().y >= W * ROWS as f32 {
                    // Game over
                    state = GameState::GameOver;
                }
            }
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
        snake.draw();

        // Print score (snake length)
        let score = snake.body.len();
        draw_text(&format!("Score: {}", score), 10.0, 20.0, 20.0, BLACK);

        match state {
            GameState::GameOver => {
                draw_text("Game Over! Press Space to Restart", 10.0, 50.0, 20.0, RED);
            }
            _ => {}
        }

        next_frame().await;
    }
}