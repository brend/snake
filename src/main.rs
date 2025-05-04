use macroquad::color::*;
use macroquad::shapes::draw_rectangle;
use macroquad::text::draw_text;
use macroquad::time::get_time;
use macroquad::window::{clear_background, next_frame};
use neural_network_study::{ActivationFunction, NeuralNetwork};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::ops::AddAssign;
use std::vec;

// Constants for the game
/// Number of grid rows
const ROWS: i32 = 20;
/// Number of grid columns
const COLS: i32 = 20;

/// Size of the population
const POPULATION_SIZE: usize = 250;
/// Number of generations
const GENERATIONS: usize = 200;
/// Number of steps before the game ends
const MAX_STEPS: usize = 500;
/// Probability of mutation of a gene (weight) of the neural network
const MUTATION_RATE: f64 = 0.1;

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
        let x = rng.random_range(0..COLS);
        let y = rng.random_range(0..ROWS);
        Self::new(x, y)
    }
}

impl AddAssign<Dir> for Pos {
    fn add_assign(&mut self, dir: Dir) {
        match dir {
            Dir::Horizontal(x) => self.x += x,
            Dir::Vertical(y) => self.y += y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    Horizontal(i32),
    Vertical(i32),
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

    fn hval(&self) -> i32 {
        match self {
            Self::Horizontal(i) => *i,
            _ => 0,
        }
    }

    fn vval(&self) -> i32 {
        match self {
            Self::Vertical(i) => *i,
            _ => 0,
        }
    }
}

struct Snake {
    body: Vec<Pos>,
    direction: Dir,
}

impl Snake {
    fn new(x: i32, y: i32) -> Self {
        Self {
            body: vec![Pos::new(x, y)],
            direction: Dir::Horizontal(1),
        }
    }

    fn head(&self) -> Pos {
        self.body[0]
    }

    fn len(&self) -> usize {
        self.body.len()
    }

    fn can_turn(&self, new_direction: Dir) -> bool {
        match (self.direction, new_direction) {
            (Dir::Horizontal(_), Dir::Horizontal(_)) => false,
            (Dir::Vertical(_), Dir::Vertical(_)) => false,
            _ => true,
        }
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
    Over,
}

#[derive(Clone)]
struct Brain {
    nn: NeuralNetwork,
}

impl Brain {
    fn new(rng: Option<&mut StdRng>) -> Self {
        let mut nn = NeuralNetwork::new(14, 16, 4, rng);

        nn.set_activation_function(ActivationFunction::Tanh);

        Self { nn }
    }

    fn make_move(&self, input: &Vec<f64>) -> Vec<f64> {
        // Use the neural network to make a move
        self.nn.predict(input.clone());
    }

    fn mutate(&mut self, rng: &mut StdRng, mutation_rate: f64) {
        self.nn.mutate(rng, mutation_rate);
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
        let mut rng = StdRng::from_os_rng();
        Self {
            state: GameState::Running,
            food: Pos::random(&mut rng),
            steps: 0,
            snake: Snake::new(COLS / 2, ROWS / 2),
            brain,
            rng,
        }
    }

    fn update(&mut self) {
        match self.state {
            GameState::Over => {}
            GameState::Running => {
                self.steps += 1;
                self.snake.update();
                // Check for collision with itself
                for i in 2..self.snake.len() {
                    if self.snake.head() == self.snake.body[i] {
                        // Game over
                        self.state = GameState::Over;
                        break;
                    }
                }

                // Get the next move from the brain
                let input = self.input();
                let output = self.brain.make_move(&input);
                // Update the snake's direction based on the brain's output
                let max_index = output
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(idx, _)| idx)
                    .unwrap();
                let desired_direction = match max_index {
                    0 => Dir::up(),
                    1 => Dir::down(),
                    2 => Dir::left(),
                    3 => Dir::right(),
                    _ => self.snake.direction,
                };
                if self.snake.can_turn(desired_direction) {
                    self.snake.direction = desired_direction;
                }

                // Check for collision with food
                if self.snake.eat(self.food) {
                    self.food = Pos::random(&mut self.rng);
                }

                // Check for collision with walls
                if self.snake.head().x < 0
                    || self.snake.head().x >= COLS
                    || self.snake.head().y < 0
                    || self.snake.head().y >= ROWS
                {
                    // Game over
                    self.state = GameState::Over;
                }
            }
        }
    }

    fn input(&self) -> Vec<f64> {
        let head = self.snake.head();
        vec![
            // snake head x position
            head.x as f64 / COLS as f64,
            // snake head y position
            head.y as f64 / ROWS as f64,
            // snake horizontal speed
            self.snake.direction.hval() as f64,
            // snake vertical speed
            self.snake.direction.vval() as f64,
            // horizontal distance from food
            (self.food.x - head.x) as f64 / COLS as f64,
            // vertical distance from food
            (self.food.y - head.y) as f64 / ROWS as f64,
            self.look_in_direction(Dir::up()).0, // Wall distance up
            self.look_in_direction(Dir::up()).1, // Body hit up
            self.look_in_direction(Dir::down()).0,
            self.look_in_direction(Dir::down()).1,
            self.look_in_direction(Dir::left()).0,
            self.look_in_direction(Dir::left()).1,
            self.look_in_direction(Dir::right()).0,
            self.look_in_direction(Dir::right()).1,
        ]
    }

    fn evaluate(&self) -> f32 {
        let len = self.snake.len();

        if len > 1 {
            100.0 * len as f32 / self.steps as f32
        } else {
            0.01 * self.steps as f32
        }
    }

    fn look_in_direction(&self, direction: Dir) -> (f64, f64) {
        let mut pos = self.snake.head();
        let mut distance = 0.0;
        let mut body_hit = 0.0;
        loop {
            pos += direction;
            distance += 1.0;
            if pos.x < 0 || pos.x >= COLS || pos.y < 0 || pos.y >= ROWS {
                break; // Hit a wall
            }
            if self.snake.body.contains(&pos) {
                body_hit = 1.0; // Hit body
                break;
            }
        }
        (distance / COLS as f64, body_hit) // Normalized distance, body hit indicator
    }
}

fn train() -> Option<Brain> {
    let mut rng = StdRng::from_os_rng();
    let mut generation = 0;
    let mut population = vec![];
    for _ in 0..POPULATION_SIZE {
        population.push(Game::new(Brain::new(Some(&mut rng))));
    }
    let mut champion = None;

    loop {
        generation += 1;
        println!("Generation: {}", generation);

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
            for game in &mut population
                .iter_mut()
                .filter(|g| g.state == GameState::Running)
            {
                alive = true;
                game.update();
            }
            if !alive {
                break;
            }
        }

        // Evaluate the population and create a mating pool
        let mut mating_pool = vec![];
        let mut scored_games = population
            .iter()
            .map(|g| (g.evaluate(), g))
            .collect::<Vec<_>>();
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
        let len = scored_games.len();
        while new_population.len() < POPULATION_SIZE {
            // Randomly select a parent from the mating pool
            let selected_parent = if score_sum > 0.0 {
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
                selected_parent
            } else if score_sum == 0.0 {
                let i = rng.random_range(0..len);
                Some(&scored_games[i].1)
            } else {
                panic!();
            };
            let selected_parent = selected_parent.unwrap();
            // Apply some mutation to the parent's brain
            let mut child_brain = selected_parent.brain.clone();
            child_brain.mutate(&mut rng, MUTATION_RATE);
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
            draw_rectangle(segment.x as f32 * W, segment.y as f32 * W, W, W, BLACK);
        }
        // Draw the food
        draw_rectangle(game.food.x as f32 * W, game.food.y as f32 * W, W, W, GREEN);

        // Update the game state
        if game.state != GameState::Running {
            draw_text("Game Over", 10.0, W, W, BLACK);
        } else {
            draw_text("Running", 10.0, W, W, BLACK);
        }
        draw_text(&format!("Score: {}", game.evaluate()), 10.0, 40.0, W, BLACK);

        next_frame().await;
    }
}
