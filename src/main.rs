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
            body: vec![Vec2::new(0.0, W * 2.0)],
            velocity: Vec2::new(INITIAL_SPEED, 0.0),
            speed: INITIAL_SPEED,
        }
    }

    fn head(&self) -> Vec2 {
        self.body[0]
    }

    fn len(&self) -> usize {
        self.body.len()
    }

    fn grow(&mut self) {
        // Add a new segment to the snake's body
        let last_segment = self.body.last().unwrap();
        self.body.push(snapv(last_segment));
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

struct Brain {}

impl Brain {
    fn make_move(&self, input: &Vec<f64>) -> Vec<f64> {
        let snake_x = input[0];
        let snake_y = input[1];
        let snake_vx = input[2];
        let snake_vy = input[3];
        let food_x = input[4];
        let food_y = input[5];
        let dx = food_x - snake_x;
        let dy = food_y - snake_y;
        // move towards the food
        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                return vec![0.0, 0.0, 0.0, 1.0]; // Move right
            } else {
                return vec![0.0, 0.0, 1.0, 0.0]; // Move left
            }
        } else {
            if dy > 0.0 {
                return vec![0.0, 1.0, 0.0, 0.0]; // Move down
            } else {
                return vec![1.0, 0.0, 0.0, 0.0]; // Move up
            }
        }
    }
}

struct Game {
    state: GameState,
    food: Vec2,
    snake: Snake,
    brain: Brain,
}

impl Game {
    fn new(brain: Brain) -> Self {
        Self {
            state: GameState::Running,
            food: random_position(),
            snake: Snake::new(),
            brain,
        }
    }

    fn update(&mut self) {
        match self.state {
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) {
                    // Restart the game
                    self.snake = Snake::new();
                    self.food = random_position();
                    self.state = GameState::Running;
                }
            }
            GameState::Running => {
                if self.snake.update() {
                    // Check for collision with itself
                    for i in 2..self.snake.len() {
                        if snapv(&self.snake.head()) == snapv(&self.snake.body[i]) {
                            // Game over
                            self.state = GameState::GameOver;
                            break;
                        }
                    }

                    // Get the next move from the brain
                    let input = vec![
                        snap(self.snake.head().x) as f64,
                        snap(self.snake.head().y) as f64,
                        self.snake.velocity.x as f64,
                        self.snake.velocity.y as f64,
                        self.food.x as f64,
                        self.food.y as f64,
                    ];
                    let output = self.brain.make_move(&input);
                    // Update the snake's velocity based on the brain's output
                    if output[0] > 0.5 {
                        self.snake.velocity = Vec2::new(0.0, -self.snake.speed);
                    } else if output[1] > 0.5 {
                        self.snake.velocity = Vec2::new(0.0, self.snake.speed);
                    } else if output[2] > 0.5 {
                        self.snake.velocity = Vec2::new(-self.snake.speed, 0.0);
                    } else if output[3] > 0.5 {
                        self.snake.velocity = Vec2::new(self.snake.speed, 0.0);
                    }
                }

                // Check for collision with food
                if self.snake.eat(&self.food) {
                    self.food = random_position();
                }

                // Check for collision with walls
                if self.snake.head().x < 0.0
                    || self.snake.head().x >= W * COLS as f32
                    || self.snake.head().y < 0.0
                    || self.snake.head().y >= W * ROWS as f32
                {
                    // Game over
                    self.state = GameState::GameOver;
                }
            }
        }
    }

    fn draw(&self) {
        let food_color = Color::from_hex(0x355834);
        let grid_colors = vec![Color::from_hex(0xDEC0F1), Color::from_hex(0xB79CED)];
        let font_size = 40.0;
    
        // Draw the grid
        for y in 0..ROWS {
            for x in 0..COLS {
                let color = grid_colors[(x + y) % 2];
                draw_rectangle(x as f32 * W, y as f32 * W, W, W, color);
            }
        }

        // Draw the food
        draw_rectangle(self.food.x.floor(), self.food.y.floor(), W, W, food_color);

        // Draw the snake
        self.snake.draw();

        // Print score (snake length)
        let score = self.snake.body.len();
        draw_text(&format!("Score: {}", score), 10.0, 20.0, font_size, BLACK);

        match self.state {
            GameState::GameOver => {
                let game_over_message = format!("Game Over! Score: {}. Press Space to Restart", self.evaluate());
                draw_text(
                    &game_over_message,
                    10.0,
                    50.0,
                    font_size,
                    RED,
                );
            }
            _ => {}
        }
    }

    fn evaluate(&self) -> f32 {
        // Evaluate the snake's performance
        let dx = self.snake.head().x - self.food.x;
        let dy = self.snake.head().y - self.food.y;
        let distance_to_food = dx.abs() + dy.abs();
        let score = self.snake.len() as f32  * 100.0;
        (score - distance_to_food + (ROWS * COLS) as f32) / 10000.0
    }
}

#[macroquad::main("Snake Game")]
async fn main() {
    let mut game = Game::new(Brain {});

    loop {
        game.update();

        clear_background(BLACK);
        game.draw();

        next_frame().await;
    }
}
