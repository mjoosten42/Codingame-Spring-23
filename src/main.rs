use std::{collections::{VecDeque, HashMap}, fmt, hash::Hash, ops::Add, format, eprintln};

// https://www.redblobgames.com/grids/hexagons/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Player {
	Ally,
	Enemy,
}

#[derive(Copy, Clone, Debug)]
struct Cell {
	index: usize,

	base: Option<Player>,
	cell_type: i32, // TODO
	resources: usize,
	ally_ants: usize,

    ally_distance: usize,
    enemy_distance: usize,
}

impl Cell {
	fn new() -> Self {
		Self {
			index: 0,
			base: None,
			cell_type: 0,
			resources: 0,
			ally_ants: 0,
			ally_distance: 0,
			enemy_distance: 0,
		}
	}
}

impl fmt::Display for Cell {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.index)
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Hex {
	q: i32,
	r: i32,
	s: i32,
}

impl Hex {
	fn new() -> Self {
		Self { q: 0, r: 0, s: 0 }
	}

	fn from(q: i32, r: i32, s: i32) -> Self {
		Self { q, r, s }
	}
}

impl fmt::Display for Hex {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[{:>2} {:>2} {:>2}]", self.r, self.q, self.s)
	}
}

impl Add for Hex {
    type Output = Hex;

    fn add(self, rhs: Self) -> Self::Output {
        Self { q: self.q + rhs.q, r: self.r + rhs.r, s: self.s + rhs.s }
    }
}

fn main() {
	let number_of_cells = input()[0] as usize;
    let (mut cells, indices) = parse(number_of_cells);
    
	let (&ally_base, &ally_base_cell) = cells.iter().find(|(_, cell)| cell.base == Some(Player::Ally)).unwrap();
	let (&enemy_base, _) = cells.iter().find(|(_, cell)| cell.base == Some(Player::Enemy)).unwrap();

    flood_fill(&mut cells, &ally_base, |cell, dist| cell.ally_distance = dist);
    flood_fill(&mut cells, &enemy_base, |cell, dist| cell.enemy_distance = dist);

	loop {
		let mut actions: String = String::from("WAIT;");
	
		parse_turn(&mut cells, &indices, number_of_cells);

        let targets: Vec<&Cell> = cells
            .values()
            .filter(|cell| cell.ally_distance <= cell.enemy_distance && cell.resources > 0)
            .collect();
    
        let dist = targets
            .iter()
            .min_by(|lhs, rhs| lhs.ally_distance.cmp(&rhs.ally_distance))
            .unwrap().ally_distance;

        for target in targets.into_iter().filter(|cell| cell.ally_distance == dist) {
            let strength = target.resources / (target.ally_distance + 1);

            actions.push_str(&format!("LINE {} {target} {strength};", ally_base_cell.index));
        }
    
		println!("{actions}");
	}
}

fn flood_fill(cells: &mut HashMap<Hex, Cell>, start: &Hex, f: impl Fn(&mut Cell, usize)) {
	let mut edges: Vec<Hex> = vec![*start].into();
	let mut visited: Vec<Hex> = edges.clone();
	let mut distance = 0;

	while !edges.is_empty() {
		let mut adding = Vec::new();
	
		for edge in edges {
			let cell = cells.get_mut(&edge).unwrap();

            f(cell, distance);
		
			for hex in adjacent(cells, &edge) {
				if !visited.contains(&hex) {
					adding.push(hex);
					visited.push(hex);
				}
			}
		}

		distance += 1;

		edges = adding;
	}
}

fn parse_turn(cells: &mut HashMap<Hex, Cell>, indices: &HashMap<usize, Hex>, n: usize) {
    for i in 0..n {
        let inputs = input();
        let resources = inputs[0]; // the current amount of eggs/crystals on this cell
        let allies = inputs[1]; // the amount of your ants on this cell
        let _enemies = inputs[2]; // the amount of opponent ants on this cell

        let hex = indices.get(&i).unwrap();
        let cell = cells.get_mut(hex).unwrap();

        cell.resources = resources as usize;
        cell.ally_ants = allies as usize;
    }
}

fn parse(n: usize) -> (HashMap<Hex, Cell>, HashMap<usize, Hex>) {
	let mut tmp_cells: HashMap<usize, Cell> = HashMap::new();
	let mut indices: Vec<(Cell, [i32; 6])> = Vec::new();

	for i in 0..n {
		let mut cell = Cell::new();
		let inputs = input();

		cell.index = i;
		cell.cell_type = inputs[0];
		cell.resources = inputs[1] as usize;

		tmp_cells.insert(i, cell);
	
		let mut tmp = [0; 6];

		tmp.clone_from_slice(&inputs[2..]);

		indices.push((cell, tmp));
	}

	let mut cells: HashMap<Hex, Cell> = hex_grid(&indices);

    let index_map: HashMap<usize, Hex> = cells.iter().map(|(hex, cell)| (cell.index, *hex)).collect();

	let _number_of_bases = input()[0];

	for i in input() {
		cells.get_mut(index_map.get(&(i as usize)).unwrap()).unwrap().base = Some(Player::Ally);
	}

	for i in input() {
		cells.get_mut(index_map.get(&(i as usize)).unwrap()).unwrap().base = Some(Player::Enemy);
	}

    (cells, index_map)
}

fn hex_grid(indices: &Vec<(Cell, [i32; 6])>) -> HashMap<Hex, Cell> {
	let zero = *indices.iter().find(|(c, _)| c.index == 0).unwrap();
	let mut cells: HashMap<Hex, Cell> = HashMap::from([(Hex::new(), zero.0)]);
	let mut edges: VecDeque<(Cell, [i32; 6])> = vec![zero].into();
	let mut visited: Vec<usize> = vec![zero.0.index];

	while !edges.is_empty() {
		let (cell, links) = edges.pop_front().unwrap();

		let adjacent: Vec<(Cell, [i32; 6])> = links
			.iter()
			.filter(|i| **i != -1 && !visited.contains(&(**i as usize)))
			.map(|link| indices[*link as usize])
			.collect();

		for (c, l) in adjacent {
			let (offset, _) = links
				.iter()
				.enumerate()
				.filter(|(_, l)| **l != -1)
				.find(|&(_, d)| *d == c.index as i32)
				.unwrap();

			let (h, _) = cells.iter().find(|(_, d)| d.index == cell.index).unwrap();
			let hex = *h + hexes()[offset];

			cells.insert(hex, c);
			edges.push_back((c, l));
			visited.push(c.index);
		}
	}

	cells
}

fn adjacent(cells: &HashMap<Hex, Cell>, hex: &Hex) -> Vec<Hex> {
    let adj = hexes().map(|a| *hex + a);

    let mut answer = Vec::new();
    
    for a in adj {
        if cells.contains_key(&a) {
            answer.push(a);
        }
    }

    answer
}

fn hexes() -> [Hex; 6] {
    [
        Hex::from(1, 0, -1),
        Hex::from(1, -1, 0),
        Hex::from(0, -1, 1),
        Hex::from(-1, 0, 1),
        Hex::from(-1, 1, 0),
        Hex::from(0, 1, -1),
    ]
}

fn input() -> Vec<i32> {
	let mut input_line = String::new();
	std::io::stdin().read_line(&mut input_line).unwrap();
	let inputs = input_line.split(" ").collect::<Vec<_>>();
	inputs.into_iter().map(|s| s.trim().parse::<i32>().unwrap()).collect()
}
