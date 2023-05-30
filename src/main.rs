use std::{collections::{HashMap, VecDeque, HashSet}, fmt, hash::Hash};
use std::convert::TryInto;

// https://www.redblobgames.com/grids/hexagons/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Resource {
	None,
	Egg,
	Crystal,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Player {
	Ally,
	Enemy,
}

#[derive(Copy, Clone, Debug)]
struct Cell {
	index: usize,
	adj: [i32; 6],

	base: Option<Player>,
	resource: Resource,
	resources: usize,
	ally_ants: usize,
	enemy_ants: usize,

    ally_distance: usize,
    enemy_distance: usize,
}

impl Cell {
	fn new() -> Self {
		Self {
			index: 0,
			adj: [-1; 6],
			base: None,
			resource: Resource::None,
			resources: 0,
			ally_ants: 0,
			enemy_ants: 0,
			ally_distance: 0,
			enemy_distance: 0,
		}
	}
}

impl PartialEq for Cell {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index
	}
}

impl Eq for Cell {}

impl PartialOrd for Cell {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.index.cmp(&other.index))
	}
}

impl Ord for Cell {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.index.cmp(&other.index)
	}
}

impl Hash for Cell {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.index.hash(state)
	}
}

impl fmt::Display for Cell {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.index)
	}
}

fn main() {
	let number_of_cells = input()[0] as usize;
    let mut cells: HashMap<usize, Cell> = parse(number_of_cells);
    
	let base = *cells.values().find(|cell| cell.base == Some(Player::Ally)).unwrap();
	let enemy_base = *cells.values().find(|cell| cell.base == Some(Player::Enemy)).unwrap();

    flood_fill(&mut cells, &base, |cell, dist| cell.ally_distance = dist);
    flood_fill(&mut cells, &enemy_base, |cell, dist| cell.enemy_distance = dist);

	loop {
		let mut actions: String = String::from("WAIT;");
	
		parse_turn(&mut cells, number_of_cells);

		let mut resources: Vec<&Cell> = Vec::new();
		let mut crystals = 0;
		let mut ants = 0;
		let mut eggs = 0;

		for cell in cells.values() {
			if cell.resources > 0 {
				resources.push(cell);
			}
		
			if cell.resource == Resource::Crystal {
				crystals += cell.resources;
			}

			if cell.resource == Resource::Egg {
				eggs += cell.resources / (cell.ally_distance + 1);
			}

			ants += cell.ally_ants;
		}

		let mut paths: HashMap<usize, Vec<usize>> = HashMap::new();

		for resource in &resources {
			let path = bfs(&cells, resource, |cell| cell != *resource && cell.resources > 0).unwrap();

			paths.insert(resource.index, path);
		}

		for (r, p) in paths {
			eprintln!("{r:>2}: {p:?}");
		}
		
		let mut remaining = (crystals / ants) as i32;

		actions.push_str(&format!("MESSAGE {remaining};"));
	
		resources.retain(|cell| {
			if cell.resource == Resource::Egg {
				remaining -= 1;

				remaining > 0
			} else {
				true
			}
		});

		for hex in resources {
			actions.push_str(&format!("LINE {base} {hex} 1;"));
		}
	
		println!("{actions}");
	}
}

fn calc_score(cells: &HashMap<usize, Cell>, path: &HashSet<usize>, ants: usize, crystals: usize) -> f32 {
	let richness: usize = path.iter()
		.map(|hex| cells.get(hex).unwrap())
		.map(|cell| {
			let modifier = if cell.resource == Resource::Crystal { 1 } else { crystals / ants / 5 };

			cell.resources.clamp(0, 1) * modifier
		})
		.sum();

	let score = ants as f32 * richness as f32 / path.len() as f32;

	score
}

fn bfs(cells: &HashMap<usize, Cell>, start: &Cell, f: impl Fn(&Cell)) -> Vec<usize> {
	let mut edges: Vec<usize> = vec![start.index];
	let mut visited: Vec<usize> = edges.clone();
	let mut answer: Vec<usize> = Vec::new();

	while !edges.is_empty() && answer.is_empty() {
		let mut adding = Vec::new();

		for edge in edges {
			let cell = cells.get_mut(&edge).unwrap();

			if cell.resources > 0 {
				answer.push(edge);
			}

			for u in adjacent(cell) {
				if cells.get(&u).unwrap().ally_distance > cell.ally_distance {
					adding.push(u);
				}
			}

			visited.push(edge);
		}

		distance += 1;

		edges = adding;
	}

	answer
}

fn flood_fill(cells: &mut HashMap<usize, Cell>, start: &Cell, mut f: impl FnMut(&mut Cell, usize)) {
	let mut edges: Vec<usize> = vec![start.index];
	let mut visited: Vec<usize> = edges.clone();
	let mut distance = 0;

	while !edges.is_empty() {
		let mut adding = Vec::new();
	
		for edge in edges {
			let cell = cells.get_mut(&edge).unwrap();

            f(cell, distance);
		
			for u in adjacent(cell) {
				if !visited.contains(&u) {
					adding.push(u);
					visited.push(u);
				}
			}
		}

		distance += 1;

		edges = adding;
	}
}

fn parse_turn(cells: &mut HashMap<usize, Cell>, n: usize) {
    for i in 0..n {
        let cell = cells.get_mut(&i).unwrap();
        let inputs = input();

		cell.resources = inputs[0] as usize;
		cell.ally_ants = inputs[1] as usize;
		cell.enemy_ants = inputs[2] as usize;
    }
}

fn parse(n: usize) -> HashMap<usize, Cell> {
	let mut cells: HashMap<usize, Cell> = HashMap::new();

	for i in 0..n {
		let mut cell = Cell::new();
		let inputs = input();

		cell.index = i;
		cell.resource = match inputs[0] {
			0 => Resource::None,
			1 => Resource::Egg,
			2 => Resource::Crystal,
			_ => panic!(),
		};
		cell.resources = inputs[1] as usize;
		cell.adj = inputs[2..].try_into().unwrap();

		cells.insert(i, cell);
	}

	let _number_of_bases = input()[0];

	for i in input() {
		cells.get_mut(&(i as usize)).unwrap().base = Some(Player::Ally);
	}

	for i in input() {
		cells.get_mut(&(i as usize)).unwrap().base = Some(Player::Enemy);
	}

    cells
}

fn adjacent(cell: &Cell) -> Vec<usize> {
	let mut adj = Vec::new();

	for a in cell.adj {
		if a != -1 {
			adj.push(a as usize);
		}
	}

	adj
}

fn input() -> Vec<i32> {
	let mut input_line = String::new();
	std::io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	inputs.into_iter().map(|s| s.trim().parse::<i32>().unwrap()).collect()
}
