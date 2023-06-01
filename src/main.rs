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
	ants: usize,
	enemy_ants: usize,

    distance: usize,
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
			ants: 0,
			enemy_ants: 0,
			distance: 0,
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
    
	let bases: Vec<usize> = cells.values()
		.filter(|cell| cell.base == Some(Player::Ally))
		.map(|cell| cell.index)
		.collect();

	let enemy_bases: Vec<usize> = cells.values()
		.filter(|cell| cell.base == Some(Player::Enemy))
		.map(|cell| cell.index)
		.collect();

    flood_fill(&mut cells, &bases, |cell, dist| cell.distance = dist);
    flood_fill(&mut cells, &enemy_bases, |cell, dist| cell.enemy_distance = dist);

	let goal: usize = cells.values()
		.filter(|cell| cell.resource == Resource::Crystal)
		.map(|cell| cell.resources)
		.sum::<usize>() / 2;

	let magic_ratio = f64::powf(cells.len() as f64, 1.0 / 3.0) as usize;

	loop {
		let harvested = parse_turn(&mut cells, number_of_cells);

		let mut resources: Vec<&Cell> = Vec::new();
		let mut crystals: Vec<&Cell> = Vec::new();
		let mut eggs: Vec<&Cell> = Vec::new();
		let mut ants: Vec<&Cell> = Vec::new();
		let mut _total_crystals = 0;
		let mut _total_eggs = 0;
		let mut total_ants = 0;
		let mut _enemies = 0;

		for cell in cells.values() {
			if cell.ants > 0 {
				ants.push(cell);
			}
		
			if cell.resources > 0 {
				resources.push(cell);
			
				if cell.resource == Resource::Crystal {
					crystals.push(cell);
				}

				if cell.resource == Resource::Egg {
					eggs.push(cell);
				}
			}

			if cell.resource == Resource::Crystal {
				_total_crystals += cell.resources * cell.distance;
			}

			if cell.resource == Resource::Egg {
				_total_eggs += cell.resources;
			}

			total_ants += cell.ants;
			_enemies += cell.enemy_ants;
		}

		let needed = goal - harvested;
		let ratio = needed / total_ants;
		let mut claimed: Vec<usize> = bases.clone().into_iter().collect();
		let mut budget = total_ants;

		resources.sort_by_key(|cell| cell.distance);

		if ratio > magic_ratio && !eggs.is_empty() {
			resources.retain(|cell| cell.resource == Resource::Egg);
			print!("MESSAGE Eggs only;")
		}

		if ratio <= magic_ratio && ratio > 0 {
			print!("MESSAGE Both;")
		}

		if ratio == 0 {
			resources.retain(|cell| cell.resource == Resource::Crystal && cell.distance <= cell.enemy_distance);
			print!("MESSAGE Crystals only;")
		}

		// Ensure the ant population is increasing
		if let Some(egg) = resources.iter().find(|cell| cell.resource == Resource::Egg) {
			let paths = paths(&cells, &bases, |cell| claimed.contains(&cell.index));

			line(&cells, egg.index, &claimed, 1);
			
			if paths.len() == 1 {
				claimed.extend(paths[0].iter());
			}

			claimed.push(egg.index);

			budget -= paths[0].len();
		}

		for resource in resources {
			let paths = paths(&cells, &vec![resource.index], |cell| claimed.contains(&cell.index));
			let distance = paths[0].len();

			if budget > distance {
				budget -= distance;

				line(&cells, resource.index, &claimed, 1);
			
				if paths.len() == 1 {
					claimed.extend(paths[0].iter());
				}
	
				claimed.push(resource.index);
			}
		}

		for base in &bases {
			print!("BEACON {base} 1;");
		}

		println!("WAIT;");
	}
}

fn line(cells: &HashMap<usize, Cell>, target: usize, from: &Vec<usize>, strength: usize) {
	let closest = bfs(&cells, &vec![target], |cell| from.contains(&cell.index));

	print!("LINE {closest} {target} {strength};");
}

fn paths(cells: &HashMap<usize, Cell>, start: &Vec<usize>, f: impl Fn(&Cell) -> bool) -> Vec<Vec<usize>> {
	let mut paths: VecDeque<Option<Vec<usize>>> = vec![Some(start.clone()), None].into();
	let mut answer: Vec<Vec<usize>> = Vec::new();
	let mut visited: Vec<usize> = start.clone();

	while !paths.is_empty() {
		let opt = paths.pop_front().unwrap();

		if opt.is_none() {
			if !answer.is_empty() {
				return answer;
			}
		
			paths.push_back(None);
			continue;
		}

		let path = opt.unwrap();
		let last = path.last().unwrap();
		let cell = cells.get(&last).unwrap();

		if f(cell) {
			answer.push(path.clone());
		}
	
		for u in adjacent(cell) {
			if !visited.contains(&u) {
				paths.push_back(Some(path.clone().into_iter().chain([u]).collect()));
			}
		}

		visited.push(*last);
	}

	panic!("No paths found!");
}

fn bfs(cells: &HashMap<usize, Cell>, start: &Vec<usize>, f: impl Fn(&Cell) -> bool) -> usize {
	let mut edges: VecDeque<usize> = start.clone().into();
	let mut visited: HashSet<usize> = HashSet::new();

	while !edges.is_empty() {
		let edge = edges.pop_front().unwrap();
		let cell = cells.get(&edge).unwrap();

		if f(cell) {
			return edge;
		}

		for adj in adjacent(cell) {
			if !visited.contains(&adj) {
				edges.push_back(adj);
			}
		}

		visited.insert(edge);
	}

	panic!("No cell found!");
}

fn flood_fill(cells: &mut HashMap<usize, Cell>, start: &Vec<usize>, mut f: impl FnMut(&mut Cell, usize)) {
	let mut edges: Vec<usize> = start.clone();
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

fn parse_turn(cells: &mut HashMap<usize, Cell>, n: usize) -> usize {
	let harvested = input()[0] as usize;

    for i in 0..n {
        let cell = cells.get_mut(&i).unwrap();
        let inputs = input();

		cell.resources = inputs[0] as usize;
		cell.ants = inputs[1] as usize;
		cell.enemy_ants = inputs[2] as usize;
    }

	harvested
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
	cell.adj.iter()
		.filter(|a| **a != -1)
		.map(|a| *a as usize)
		.collect()
}

fn input() -> Vec<i32> {
	let mut input_line = String::new();
	std::io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	inputs.into_iter().map(|s| s.trim().parse::<i32>().unwrap()).collect()
}
