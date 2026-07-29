use super::molecule::Molecule;
use nalgebra::{Point2, Vector2};

pub fn generate_2d_layout(mol: &mut Molecule) {
    let n = mol.atoms.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        mol.atoms[0].pos = Point2::new(0.0, 0.0);
        return;
    }

    // Initialize positions in a circle to avoid overlapping at 0,0
    let mut lcg_state: u32 = 12345;
    let mut next_f32 = || -> f32 {
        lcg_state = lcg_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let float = (lcg_state as f32) / (u32::MAX as f32);
        (float * 2.0) - 1.0 // Map to -1.0 .. 1.0
    };

    let radius = (n as f32) * 10.0;
    for i in 0..n {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / (n as f32);
        mol.atoms[i].pos = Point2::new(
            radius * angle.cos() + next_f32(),
            radius * angle.sin() + next_f32(),
        );
    }

    let iterations = 500;
    let ideal_length = 40.0_f32;
    let k_repel = 1000.0_f32; // Repulsion constant
    let k_spring = 0.5_f32;  // Spring constant

    let mut velocities = vec![Vector2::new(0.0, 0.0); n];

    for _ in 0..iterations {
        let mut forces = vec![Vector2::new(0.0, 0.0); n];

        // Repulsion forces (all pairs)
        for i in 0..n {
            for j in (i + 1)..n {
                let diff = mol.atoms[i].pos - mol.atoms[j].pos;
                let mut d = diff.norm();
                if d < 0.1 {
                    d = 0.1; // avoid division by zero
                }
                let f = k_repel / (d * d);
                let dir = diff / d;
                
                forces[i] += dir * f;
                forces[j] -= dir * f;
            }
        }

        // Attraction forces (bonds)
        for bond in &mol.bonds {
            let u = bond.from;
            let v = bond.to;
            if u >= n || v >= n { continue; }
            
            let diff = mol.atoms[v].pos - mol.atoms[u].pos;
            let mut d = diff.norm();
            if d < 0.1 {
                d = 0.1;
            }
            // Spring force
            let f = k_spring * (d - ideal_length);
            let dir = diff / d;
            
            forces[u] += dir * f;
            forces[v] -= dir * f;
        }

        // Apply forces and damping
        for i in 0..n {
            velocities[i] = (velocities[i] + forces[i]) * 0.7; // 0.7 is damping factor
            mol.atoms[i].pos += velocities[i] * 0.1; // 0.1 is time step
        }
    }

    // Center the molecule
    let mut center = Point2::new(0.0, 0.0);
    for atom in &mol.atoms {
        center.coords += atom.pos.coords;
    }
    center.coords /= n as f32;

    for atom in &mut mol.atoms {
        atom.pos -= center.coords;
    }
}
