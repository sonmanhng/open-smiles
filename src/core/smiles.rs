use super::molecule::{BondOrder, Molecule};
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::Dfs;
use std::collections::{HashMap, HashSet};

pub fn generate_smiles(mol: &Molecule) -> String {
    if mol.atoms.is_empty() {
        return String::new();
    }

    // Build undirected graph
    let mut graph = UnGraph::<usize, BondOrder>::new_undirected();
    let mut node_indices = Vec::new();

    for (i, _) in mol.atoms.iter().enumerate() {
        node_indices.push(graph.add_node(i));
    }

    for bond in &mol.bonds {
        graph.add_edge(node_indices[bond.from], node_indices[bond.to], bond.order.clone());
    }

    // Find connected components
    let mut visited = HashSet::new();
    let mut components = Vec::new();

    for i in 0..mol.atoms.len() {
        if visited.contains(&i) {
            continue;
        }

        let mut comp = Vec::new();
        let mut dfs = Dfs::new(&graph, node_indices[i]);
        while let Some(nx) = dfs.next(&graph) {
            let idx = graph[nx];
            if visited.insert(idx) {
                comp.push(idx);
            }
        }
        components.push(comp);
    }

    // Generate SMILES for each component
    let mut smiles_parts = Vec::new();
    for comp in components {
        smiles_parts.push(generate_component_smiles(mol, &graph, &node_indices, comp[0]));
    }

    smiles_parts.join(".")
}

fn generate_component_smiles(
    mol: &Molecule,
    graph: &UnGraph<usize, BondOrder>,
    node_indices: &[NodeIndex],
    start_idx: usize,
) -> String {
    let mut smiles = String::new();
    let mut visited_nodes = HashSet::new();
    let mut ring_closures: HashMap<(usize, usize), usize> = HashMap::new(); // edge (u, v) -> ring number
    let mut ring_counter = 1;

    // First pass: identify ring closure bonds using a DFS tree
    // Any edge that leads to an already visited node (and is not the parent) is a ring closure.
    let mut dfs_visited = HashSet::new();
    let mut stack = vec![(start_idx, start_idx)];
    
    while let Some((curr, parent)) = stack.pop() {
        if dfs_visited.contains(&curr) {
            continue;
        }
        dfs_visited.insert(curr);

        let curr_node = node_indices[curr];
        for neighbor in graph.neighbors(curr_node) {
            let next = graph[neighbor];
            if next == parent {
                continue;
            }
            if dfs_visited.contains(&next) {
                // Ring closure detected
                let edge_key = if curr < next { (curr, next) } else { (next, curr) };
                if !ring_closures.contains_key(&edge_key) {
                    ring_closures.insert(edge_key, ring_counter);
                    ring_counter += 1;
                }
            } else {
                stack.push((next, curr));
            }
        }
    }

    // Second pass: recursive traversal to build string
    build_smiles_dfs(
        mol,
        graph,
        node_indices,
        start_idx,
        start_idx,
        &mut visited_nodes,
        &ring_closures,
        &mut smiles,
    );

    smiles
}

fn build_smiles_dfs(
    mol: &Molecule,
    graph: &UnGraph<usize, BondOrder>,
    node_indices: &[NodeIndex],
    curr: usize,
    parent: usize,
    visited: &mut HashSet<usize>,
    ring_closures: &HashMap<(usize, usize), usize>,
    smiles: &mut String,
) {
    visited.insert(curr);

    // If there is a bond from parent to curr, append bond symbol
    if curr != parent {
        let parent_node = node_indices[parent];
        let curr_node = node_indices[curr];
        if let Some(edge_idx) = graph.find_edge(parent_node, curr_node) {
            let bond_order = &graph[edge_idx];
            match bond_order {
                BondOrder::Double => smiles.push('='),
                BondOrder::Triple => smiles.push('#'),
                BondOrder::Single => {} // usually omitted
            }
        }
    }

    // Append atom symbol
    smiles.push_str(&mol.atoms[curr].element);

    // Append any ring closures for this atom
    let curr_node = node_indices[curr];
    let mut closures_for_curr = Vec::new();
    for neighbor in graph.neighbors(curr_node) {
        let next = graph[neighbor];
        let edge_key = if curr < next { (curr, next) } else { (next, curr) };
        if let Some(&ring_num) = ring_closures.get(&edge_key) {
            closures_for_curr.push(ring_num);
        }
    }
    closures_for_curr.sort_unstable();
    for num in closures_for_curr {
        if num < 10 {
            smiles.push_str(&num.to_string());
        } else {
            smiles.push_str(&format!("%{}", num));
        }
    }

    // Collect children to traverse (exclude parent and ring closure neighbors)
    let mut children = Vec::new();
    for neighbor in graph.neighbors(curr_node) {
        let next = graph[neighbor];
        if next == parent {
            continue;
        }
        let edge_key = if curr < next { (curr, next) } else { (next, curr) };
        if ring_closures.contains_key(&edge_key) {
            continue;
        }
        if !visited.contains(&next) {
            children.push(next);
        }
    }

    // Traverse children
    for (i, &child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        if !is_last {
            smiles.push('(');
        }
        build_smiles_dfs(mol, graph, node_indices, child, curr, visited, ring_closures, smiles);
        if !is_last {
            smiles.push(')');
        }
    }
}

pub fn parse_smiles(smiles: &str) -> Option<Molecule> {
    let mut mol = Molecule::new();
    let mut branch_stack: Vec<usize> = Vec::new();
    let mut ring_map: HashMap<u32, (usize, BondOrder)> = HashMap::new();
    let mut current_atom: Option<usize> = None;
    let mut next_bond: Option<BondOrder> = None;

    let chars: Vec<char> = smiles.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            '(' => {
                if let Some(curr) = current_atom {
                    branch_stack.push(curr);
                }
                i += 1;
            }
            ')' => {
                current_atom = branch_stack.pop();
                i += 1;
            }
            '=' => {
                next_bond = Some(BondOrder::Double);
                i += 1;
            }
            '#' => {
                next_bond = Some(BondOrder::Triple);
                i += 1;
            }
            '-' => {
                next_bond = Some(BondOrder::Single);
                i += 1;
            }
            '1'..='9' => {
                let digit = c.to_digit(10).unwrap();
                if let Some(curr) = current_atom {
                    if let Some((target_atom, bond_order)) = ring_map.remove(&digit) {
                        let order = next_bond.take().unwrap_or(bond_order);
                        mol.add_bond(curr, target_atom, order);
                    } else {
                        ring_map.insert(digit, (curr, next_bond.take().unwrap_or(BondOrder::Single)));
                    }
                }
                i += 1;
            }
            '%' => {
                if i + 2 < chars.len() {
                    let d1 = chars[i+1].to_digit(10);
                    let d2 = chars[i+2].to_digit(10);
                    if let (Some(d1), Some(d2)) = (d1, d2) {
                        let digit = d1 * 10 + d2;
                        if let Some(curr) = current_atom {
                            if let Some((target_atom, bond_order)) = ring_map.remove(&digit) {
                                let order = next_bond.take().unwrap_or(bond_order);
                                mol.add_bond(curr, target_atom, order);
                            } else {
                                ring_map.insert(digit, (curr, next_bond.take().unwrap_or(BondOrder::Single)));
                            }
                        }
                        i += 3;
                        continue;
                    }
                }
                i += 1;
            }
            '[' => {
                // Skip bracket contents for now, just extract the first alphabetic element inside
                let mut elem = String::new();
                i += 1;
                while i < chars.len() && chars[i] != ']' {
                    if chars[i].is_alphabetic() && elem.is_empty() {
                        elem.push(chars[i].to_ascii_uppercase());
                        if i + 1 < chars.len() && chars[i+1].is_ascii_lowercase() && chars[i+1] != ']' {
                            elem.push(chars[i+1]);
                            i += 1;
                        }
                    }
                    i += 1;
                }
                if i < chars.len() && chars[i] == ']' {
                    i += 1;
                }
                if !elem.is_empty() {
                    let new_atom_idx = mol.atoms.len();
                    mol.add_atom(crate::core::molecule::Atom {
                        element: elem,
                        pos: nalgebra::Point2::new(0.0, 0.0),
                        charge: 0,
                    });
                    if let Some(curr) = current_atom {
                        mol.add_bond(curr, new_atom_idx, next_bond.take().unwrap_or(BondOrder::Single));
                    }
                    current_atom = Some(new_atom_idx);
                }
            }
            _ if c.is_alphabetic() => {
                let mut elem = String::new();
                elem.push(c.to_ascii_uppercase());
                if i + 1 < chars.len() && chars[i+1].is_ascii_lowercase() {
                    // Check valid two letter elements
                    let next_c = chars[i+1];
                    let potential = format!("{}{}", elem, next_c);
                    if ["Cl", "Br", "Na", "Mg", "Ca", "Fe", "Cu", "Zn", "As", "Se", "Si", "Li", "K"].contains(&potential.as_str()) {
                        elem.push(next_c);
                        i += 1;
                    }
                }
                
                let new_atom_idx = mol.atoms.len();
                mol.add_atom(crate::core::molecule::Atom {
                    element: elem,
                    pos: nalgebra::Point2::new(0.0, 0.0),
                    charge: 0,
                });
                if let Some(curr) = current_atom {
                    mol.add_bond(curr, new_atom_idx, next_bond.take().unwrap_or(BondOrder::Single));
                }
                current_atom = Some(new_atom_idx);
                i += 1;
            }
            _ => {
                i += 1; // ignore other chars
            }
        }
    }

    if mol.atoms.is_empty() {
        None
    } else {
        Some(mol)
    }
}
