use std::{collections::{HashMap, VecDeque, HashSet}, fmt, hash::Hash};
use std::convert::TryInto;

// https://www.redblobgames.com/grids/hexagons/

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
		write!(f, "{:>2}", self.index)
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

		let mut paths: Vec<HashSet<usize>> = vec![HashSet::from([base.index])];
		let (mut s, mut j) = (0.0, 0);
	
		for i in 0.. {
			if let Some(path) = bfs(&cells, &paths[i], |cell| {
				cell.resources > 0 && !paths[i].contains(&cell.index)
			}) {
				let last = paths.last().unwrap().clone();
				let extended: HashSet<usize> = last.into_iter().chain(path.into_iter()).collect();
				let score = calc_score(&cells, &extended, ants, crystals);
				
				// {
				// 	let mut tmp: Vec<usize> = extended.clone().into_iter().collect();
				// 	tmp.sort_by(|lhs, rhs| 
				// 		cells.get(lhs).unwrap().ally_distance.cmp(&cells.get(rhs).unwrap().ally_distance)
				// 	);
				
				// 	eprint!("{score:.2}: ");
				// 	for hex in tmp {
				// 		eprint!("{hex:>2} ");
				// 	}
				// 	eprintln!("");
				// }
			
				if score > s {
					s = score;
					j = i + 1;
				}

				paths.push(extended);

			} else {
				break ;
			}
		}

		let best = &paths[j];

		actions.push_str(&format!("MESSAGE {};", best.len()));
	
		for hex in best {
			actions.push_str(&format!("BEACON {hex} 1;"));
		}
	
		println!("{actions}");
	}
}

fn calc_score(cells: &HashMap<usize, Cell>, path: &HashSet<usize>, ants: usize, crystals: usize) -> f32 {
	let richness: usize = path.iter()
		.map(|hex| cells.get(hex).unwrap())
		.map(|cell| {
			let modifier = if cell.cell_type == CellType::Crystal { 1 } else { crystals / ants / 5 };

			cell.resources.clamp(0, 1) * modifier
		})
		.sum();

	let score = ants as f32 * richness as f32 / path.len() as f32;

	score
}

fn bfs(cells: &HashMap<usize, Cell>, path: &HashSet<usize>, f: impl Fn(&Cell) -> bool) -> Option<Vec<usize>> {
	let mut edges: VecDeque<Vec<usize>> = path.iter().map(|hex| vec![*hex]).collect();
	let mut visited: Vec<usize> = path.clone().into_iter().collect();

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
