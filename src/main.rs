use std::ops::AddAssign;
use std::vec;
use neural_network_study::NeuralNetwork;
use rand::prelude::*;
use rand::rngs::StdRng;

const ROWS: usize = 20;
const COLS: usize = 20;

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
        Self {
            nn: NeuralNetwork::new(4, 6, 4),
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
        let dx = self.snake.head().x - self.food.x;
        let dy = self.snake.head().y - self.food.y;
        let distance_to_food = dx.abs() + dy.abs();
        let length_bonus = self.snake.len() as f32 * 100.0;        
        self.steps as f32 + length_bonus - distance_to_food as f32
    }
}

fn main() {
    let mut rng = StdRng::from_seed([42u8; 32]);
    let mut generation = 0;
    let population_size = 100;
    let mut population = vec![];
    for _ in 0..population_size {
        population.push(Game::new(Brain::new()));
    }

    loop {
        generation += 1;
        println!("Generation: {}", generation);

        // DEBUG
        if generation > 10 {
            break;
        }

        // Run the simulation with the current population
        // until all games are over
        let max_steps = 1000;
        let mut steps = 0;
        loop {
            steps += 1;
            if steps > max_steps {
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
            score_sum += score;
            if score > &best_score {
                best_score = *score;
            }
            mating_pool.push(game);
        }

        println!("Best score: {}", best_score);
        println!("Average score: {}", score_sum / population_size as f32);

        // Create a new generation
        let mut new_population = vec![];
        for _ in 0..population_size {
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
}
