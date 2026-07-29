use nalgebra::Point2;

#[derive(Clone, Debug)]
pub struct Atom {
    pub element: String,
    pub pos: Point2<f32>,
    pub charge: i32,
}

impl Atom {
    pub fn new(element: &str, x: f32, y: f32) -> Self {
        Self {
            element: element.to_string(),
            pos: Point2::new(x, y),
            charge: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BondOrder {
    Single,
    Double,
    Triple,
}

#[derive(Clone, Debug)]
pub struct Bond {
    pub from: usize,
    pub to: usize,
    pub order: BondOrder,
}

impl Bond {
    pub fn new(from: usize, to: usize, order: BondOrder) -> Self {
        Self { from, to, order }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Molecule {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
}

impl Molecule {
    pub fn new() -> Self {
        Self {
            atoms: Vec::new(),
            bonds: Vec::new(),
        }
    }

    pub fn add_atom(&mut self, atom: Atom) -> usize {
        let idx = self.atoms.len();
        self.atoms.push(atom);
        idx
    }

    pub fn add_bond(&mut self, from: usize, to: usize, order: BondOrder) {
        if from == to {
            return;
        }
        // Ensure from < to for consistency
        let (u, v) = if from < to { (from, to) } else { (to, from) };
        
        // If bond exists, just update order
        for bond in &mut self.bonds {
            if bond.from == u && bond.to == v {
                bond.order = order;
                return;
            }
        }
        self.bonds.push(Bond::new(u, v, order));
    }

    pub fn remove_atom(&mut self, index: usize) {
        if index >= self.atoms.len() {
            return;
        }
        self.atoms.remove(index);
        
        // Remove bonds involving this atom and shift indices
        self.bonds.retain(|b| b.from != index && b.to != index);
        for bond in &mut self.bonds {
            if bond.from > index {
                bond.from -= 1;
            }
            if bond.to > index {
                bond.to -= 1;
            }
        }
    }

    pub fn get_bonds_for_atom(&self, idx: usize) -> Vec<&Bond> {
        self.bonds.iter().filter(|b| b.from == idx || b.to == idx).collect()
    }
    
    pub fn clear(&mut self) {
        self.atoms.clear();
        self.bonds.clear();
    }
}
