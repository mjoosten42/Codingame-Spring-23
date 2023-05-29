use std::{collections::{HashMap, VecDeque}, fmt};
use std::convert::TryInto;

// https://www.redblobgames.com/grids/usizeagons/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CellType {
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
	cell_type: CellType,
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
			cell_type: CellType::None,
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
		
			if cell.cell_type == CellType::Crystal {
				crystals += cell.resources;
			}

			if cell.cell_type == CellType::Egg {
				eggs += cell.resources / (cell.ally_distance + 1);
			}

			ants += cell.ally_ants;
		}

		let mut path: Vec<usize> = vec![base.index];
		let mut used: Vec<usize> = vec![base.index];
	
		while let Some(p) = bfs(&cells, *used.last().unwrap(), |cell| {
			cell.resources > 0 && !used.contains(&cell.index)
		}) {
			if let Some(last) = path.last() {
				used.push(*last);
			}

			path.extend(&p[1..]);
		};

		path.dedup();

		let mut scores: Vec<usize> = Vec::new();

		eprintln!("{path:?}");
		for i in 1..path.len() {
			let partial: Vec<&Cell> = path[..i].iter()
				.map(|hex| cells.get(hex).unwrap())
				.collect();

			let richness: usize = partial.iter()
				.map(|cell| cell.resources.clamp(0, 1))
				.sum();

			eprintln!("{richness}: {partial:?}");

			scores.push(richness / i);
		}

		let best = *scores.iter().max().unwrap();

		for hex in &path[0..best] {
			actions.push_str(&format!("BEACON {hex} 1"));
		}
	
		println!("{actions}");
	}
}

fn calc_paths(cells: &HashMap<usize, Cell>, base: &Cell, end: &Cell) -> Vec<Vec<usize>> {
	let mut paths: VecDeque<Vec<usize>> = vec![vec![base.index]].into();
	let mut answers: Vec<Vec<usize>> = Vec::new();

	while !paths.is_empty() {
		let path = paths.pop_front().unwrap();
		let last = path.last().unwrap();
		let cell = cells.get(last).unwrap();

		if cell == end {
			answers.push(path.clone());
		}

		if path.len() > end.ally_distance {
			continue;
		}

		for adj in adjacent(cell) {
			if cells.get(&adj).unwrap().ally_distance > cell.ally_distance {
				paths.push_back(path.clone().into_iter().chain([adj]).collect());
			}
		}
	}

	answers
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

fn bfs(cells: &HashMap<usize, Cell>, start: usize, f: impl Fn(&Cell) -> bool) -> Option<Vec<usize>> {
	let mut edges: VecDeque<Vec<usize>> = vec![vec![start]].into();
	let mut visited: Vec<usize> = Vec::new();

	while !edges.is_empty() {
		let path = edges.pop_front().unwrap();
		let last = path.last().unwrap();
		let cell = cells.get(last).unwrap();

		if f(cell) {
			return Some(path);
		}
	
		for u in adjacent(cell) {
			if !visited.contains(&u) {
				edges.push_back(path.clone().into_iter().chain([u]).collect());
			}
		}

		visited.push(*last);
	}

	None
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
		cell.cell_type = match inputs[0] {
			0 => CellType::None,
			1 => CellType::Egg,
			2 => CellType::Crystal,
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
