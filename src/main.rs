use std::ops::AddAssign;
use std::vec;
use macroquad::color::*;
use macroquad::shapes::draw_rectangle;
use macroquad::text::draw_text;
use macroquad::time::get_time;
use macroquad::window::{clear_background, next_frame};
use neural_network_study::{NeuralNetwork, ActivationFunction};
use rand::prelude::*;
use rand::rngs::StdRng;

// Constants for the game
/// Number of grid rows
const ROWS: usize = 20;
/// Number of grid columns
const COLS: usize = 20;

/// Size of the population
const POPULATION_SIZE: usize = 100;
/// Number of generations
const GENERATIONS: usize = 100;
/// Number of steps before the game ends
const MAX_STEPS: usize = 1000;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn random(rng: &mut StdRng) -> Self {
        let x = rng.random_range(0..COLS as i32);
        let y = rng.random_range(0..ROWS as i32);
        Self::new(x, y)
    }
}

impl AddAssign<Dir> for Pos {
    fn add_assign(&mut self, dir: Dir) {
        match dir {
            Dir::Horizontal(x) => self.x = self.x + x,
            Dir::Vertical(y) => self.y = self.y + y,
        }
    }
}

impl Dir {
    fn up() -> Self {
        Self::Vertical(-1)
    }

    fn down() -> Self {
        Self::Vertical(1)
    }

    fn left() -> Self {
        Self::Horizontal(-1)
    }

    fn right() -> Self {
        Self::Horizontal(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    Horizontal(i32),
    Vertical(i32),
}

struct Snake {
    body: Vec<Pos>,
    direction: Dir,
}

impl Snake {
    fn new() -> Self {
        Self {
            body: vec![Pos::new(0, 2)],
            direction: Dir::Horizontal(1),
        }
    }

    fn head(&self) -> Pos {
        self.body[0]
    }

    fn len(&self) -> usize {
        self.body.len()
    }

    fn grow(&mut self) {
        // Add a new segment to the snake's body
        let last_segment = self.body.last().unwrap();
        self.body.push(*last_segment);
    }

    fn update(&mut self) {
        // Move the snake by updating the position of the head
        self.body[0] += self.direction;
        // Move the rest of the body
        for i in (1..self.body.len()).rev() {
            self.body[i] = self.body[i - 1];
        }
    }

    fn eat(&mut self, food: Pos) -> bool {
        // Check if the snake's head is on the food
        if self.head() == food {
            // If so, grow the snake and return true
            self.grow();
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

#[derive(Clone)]
struct Brain {
    nn: NeuralNetwork,
}

impl Brain {
    fn new() -> Self {
        let mut nn = NeuralNetwork::new(4, 6, 4);

        nn.set_activation_function(ActivationFunction::Tanh);

        Self {
            nn
        }
    }

    fn make_move(&self, input: &Vec<f64>) -> Vec<f64> {
        // Use the neural network to make a move
        let output = self.nn.predict(input.clone());
        output
    }

    fn mutate(&mut self) {
        self.nn.mutate(0.1);
    }
}

struct Game {
    state: GameState,
    food: Pos,
    steps: usize,
    snake: Snake,
    brain: Brain,
    rng: StdRng,
}

impl Game {
    fn new(brain: Brain) -> Self {
        let mut rng = StdRng::from_seed([42u8; 32]);
        Self {
            state: GameState::Running,
            food: Pos::random(&mut rng),
            steps: 0,
            snake: Snake::new(),
            brain,
            rng,
        }
    }

    fn update(&mut self) {
        self.steps += 1;
        match self.state {
            GameState::GameOver => {}
            GameState::Running => {
                self.snake.update();
                // Check for collision with itself
                for i in 2..self.snake.len() {
                    if self.snake.head() == self.snake.body[i] {
                        // Game over
                        self.state = GameState::GameOver;
                        break;
                    }
                }

                // Get the next move from the brain
                let input = vec![
                    self.snake.head().x as f64 / COLS as f64,
                    self.snake.head().y as f64 / ROWS as f64,
                    self.food.x as f64 / COLS as f64,
                    self.food.y as f64 / ROWS as f64,
                ];
                let output = self.brain.make_move(&input);
                // Update the snake's direction based on the brain's output
                if output[0] > 0.5 {
                    self.snake.direction = Dir::up();
                } else if output[1] > 0.5 {
                    self.snake.direction = Dir::down();
                } else if output[2] > 0.5 {
                    self.snake.direction = Dir::left();
                } else if output[3] > 0.5 {
                    self.snake.direction = Dir::right();
                }

                // Check for collision with food
                if self.snake.eat(self.food) {
                    self.food = Pos::random(&mut self.rng);
                }

                // Check for collision with walls
                if self.snake.head().x < 0
                    || self.snake.head().x >= COLS as i32
                    || self.snake.head().y < 0
                    || self.snake.head().y >= ROWS as i32
                {
                    // Game over
                    self.state = GameState::GameOver;
                }
            }
        }
    }

    fn evaluate(&self) -> f32 {
        // Evaluate the snake's performance
        let head = self.snake.head();
        let food = self.food;
        //let length = self.snake.len();
        let dx = head.x - food.x;
        let dy = head.y - food.y;
        let distance_to_food = dx.abs() + dy.abs();
        //let length_bonus = length as f32 * 100.0;        
        //self.steps as f32 + length_bonus - distance_to_food as f32
        (COLS as i32 * ROWS as i32 - distance_to_food) as f32
    }
}

fn train() -> Option<Brain> {
    let mut rng = StdRng::from_seed([42u8; 32]);
    let mut generation = 0;
    let mut population = vec![];
    for _ in 0..POPULATION_SIZE {
        population.push(Game::new(Brain::new()));
    }
    let mut champion = None;

    loop {
        generation += 1;
        println!("Generation: {}", generation);

        // DEBUG
        if generation > GENERATIONS {
            break;
        }

        // Run the simulation with the current population
        // until all games are over
        let mut steps = 0;
        loop {
            steps += 1;
            if steps > MAX_STEPS {
                break;
            }

            let mut alive = false;
            for game in &mut population.iter_mut().filter(|g| g.state == GameState::Running) {
                alive = true;
                game.update();
            }
            if !alive {
                break;
            }
        }

        // Evaluate the population and create a mating pool
        let mut mating_pool = vec![];
        let mut scored_games = population.iter().map(|g| (g.evaluate(), g)).collect::<Vec<_>>();
        scored_games.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let mut score_sum = 0.0;
        let mut best_score = 0.0;
        for (score, game) in &scored_games {
            assert!(score >= &0.0);
            score_sum += score;
            if score > &best_score {
                best_score = *score;
                champion = Some(game.brain.clone());
            }
            mating_pool.push(game);
        }

        println!("Best score: {}", best_score);
        println!("Average score: {}", score_sum / POPULATION_SIZE as f32);

        // Create a new generation
        let mut new_population = vec![];
        for _ in 0..POPULATION_SIZE {
            // Randomly select a parent from the mating pool
            let r = rng.random_range(0.0..score_sum);
            let mut cumulative_score = 0.0;
            let mut selected_parent = None;
            for (score, game) in &scored_games {
                cumulative_score += score;
                if cumulative_score >= r {
                    selected_parent = Some(game);
                    break;
                }
            }
            let selected_parent = selected_parent.unwrap();
            // Apply some mutation to the parent's brain
            let mut child_brain = selected_parent.brain.clone();
            child_brain.mutate();
            // Create a new child by combining the parents' brains
            new_population.push(Game::new(child_brain));
        }
        population = new_population;
    }

    match &champion {
        Some(champion) => {
            println!("{}", serde_json::to_string(&champion.nn).unwrap());
        }
        None => {
            println!("No champion found");
        }
    }

    champion
}

const W: f32 = 20.0;

#[macroquad::main("Snake")]
async fn main() {
    // Train the neural network
    let brain = train().expect("Failed to train the neural network");
    // Create a new game with the trained brain
    let mut game = Game::new(brain);
    // Run the game until it's over
    let update_time = 0.5;
    let mut last_update = get_time();
    loop {
        // Update the game at a fixed interval
        if get_time() - last_update > update_time {
            game.update();
            last_update = get_time();
        }

        clear_background(BLACK);

        // Draw the grid
        draw_rectangle(0.0, 0.0, COLS as f32 * W, ROWS as f32 * W, WHITE);

        // Draw the snake
        for segment in &game.snake.body {
            draw_rectangle(
                segment.x as f32 * W,
                segment.y as f32 * W,
                W,
                W,
                BLACK,
            );
        }
        // Draw the food
        draw_rectangle(
            game.food.x as f32 * W,
            game.food.y as f32 * W,
            W,
            W,
            GREEN,
        );

        // Update the game state
        if game.state == GameState::GameOver {
            draw_text("Game Over", 10.0, W, W, BLACK);
        } else {
            draw_text("Running", 10.0, W, W, BLACK);
        }
        draw_text(
            &format!("Score: {}", game.evaluate()),
            10.0,
            40.0,
            W,
            BLACK,
        );

        next_frame().await;
    }
}