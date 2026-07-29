use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, Shape, FontId, PointerButton};
use super::app::{OpenSmilesApp, Tool};
use crate::core::molecule::{Atom, BondOrder};

const ATOM_RADIUS: f32 = 12.0;

pub fn draw_canvas(ui: &mut egui::Ui, state: &mut OpenSmilesApp) -> bool {
    let mut changed = false;

    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        Sense::click_and_drag(),
    );

    let rect = response.rect;
    let center = rect.center().to_vec2();
    
    // Transforms
    let to_screen = |p: Pos2, zoom: f32, pan: Vec2| -> Pos2 {
        let v = p.to_vec2();
        let scaled = v * zoom;
        (scaled + pan + center).to_pos2()
    };
    
    let to_model = |p: Pos2, zoom: f32, pan: Vec2| -> Pos2 {
        let v = p.to_vec2();
        let unpanned = v - pan - center;
        (unpanned / zoom).to_pos2()
    };

    // Draw background
    painter.rect_filled(rect, 0.0, Color32::WHITE);

    // Zoom logic
    let scroll_delta = ui.input(|i| i.raw_scroll_delta);
    if scroll_delta.y != 0.0 && response.hovered() {
        if let Some(mouse_pos) = response.hover_pos() {
            let zoom_delta = (scroll_delta.y * 0.005).exp();
            let model_pos = to_model(mouse_pos, state.zoom, state.pan).to_vec2();
            state.zoom *= zoom_delta;
            // Limit zoom to sane values
            state.zoom = state.zoom.clamp(0.1, 10.0);
            state.pan = mouse_pos.to_vec2() - center - model_pos * state.zoom;
            changed = true;
        }
    }

    // Pan logic
    if response.dragged_by(PointerButton::Secondary) || response.dragged_by(PointerButton::Middle) {
        state.pan += response.drag_delta();
        changed = true;
    } else if response.dragged_by(PointerButton::Primary) && state.dragging_from.is_none() && state.current_tool == Tool::Select {
        state.pan += response.drag_delta();
        changed = true;
    }

    // Handle inputs
    let pointer_pos = response.hover_pos();
    
    // Find hovered atom/bond
    state.hovered_atom = None;
    state.hovered_bond = None;

    if let Some(pos) = pointer_pos {
        // Check atoms first
        let hit_radius = ATOM_RADIUS * 1.5 * state.zoom;
        for (i, atom) in state.molecule.atoms.iter().enumerate() {
            let p = Pos2::new(atom.pos.x, atom.pos.y);
            let screen_p = to_screen(p, state.zoom, state.pan);
            if screen_p.distance(pos) < hit_radius {
                state.hovered_atom = Some(i);
                break;
            }
        }
    }

    // Interaction logic
    if response.clicked() {
        if let Some(pos) = pointer_pos {
            let model_pos = to_model(pos, state.zoom, state.pan);
            match state.current_tool {
                Tool::Atom(el) => {
                    if let Some(hovered) = state.hovered_atom {
                        // Change element if clicking on existing atom
                        if state.molecule.atoms[hovered].element != el {
                            state.molecule.atoms[hovered].element = el.to_string();
                            changed = true;
                        }
                    } else {
                        // Create new atom
                        state.molecule.add_atom(Atom::new(el, model_pos.x, model_pos.y));
                        changed = true;
                    }
                }
                Tool::Erase => {
                    if let Some(hovered) = state.hovered_atom {
                        state.molecule.remove_atom(hovered);
                        changed = true;
                    }
                }
                _ => {}
            }
        }
    }

    // Dragging logic for bonds and moving
    if response.drag_started() {
        if let Some(hovered) = state.hovered_atom {
            state.dragging_from = Some(hovered);
        } else if let Some(pos) = pointer_pos {
            if response.dragged_by(PointerButton::Primary) {
                let model_pos = to_model(pos, state.zoom, state.pan);
                // Start a new bond by creating a starting atom if using a bond tool
                if let Tool::Bond(_) = state.current_tool {
                    let idx = state.molecule.add_atom(Atom::new("C", model_pos.x, model_pos.y));
                    state.dragging_from = Some(idx);
                    changed = true;
                }
            }
        }
    }

    if response.drag_stopped() {
        if let Some(from) = state.dragging_from {
            if let Tool::Bond(ref order) = state.current_tool {
                if let Some(to) = state.hovered_atom {
                    if from != to {
                        state.molecule.add_bond(from, to, order.clone());
                        changed = true;
                    }
                } else if let Some(pos) = pointer_pos {
                    let model_pos = to_model(pos, state.zoom, state.pan);
                    let from_pos = Pos2::new(state.molecule.atoms[from].pos.x, state.molecule.atoms[from].pos.y);
                    // Create new atom at release point if dragged far enough
                    if from_pos.distance(model_pos) > ATOM_RADIUS * 2.0 {
                        let to = state.molecule.add_atom(Atom::new("C", model_pos.x, model_pos.y));
                        state.molecule.add_bond(from, to, order.clone());
                        changed = true;
                    }
                }
            }
        }
        state.dragging_from = None;
    }

    if response.dragged_by(PointerButton::Primary) {
        if let Some(from) = state.dragging_from {
            if state.current_tool == Tool::Select {
                if let Some(pos) = pointer_pos {
                    let model_pos = to_model(pos, state.zoom, state.pan);
                    state.molecule.atoms[from].pos.x = model_pos.x;
                    state.molecule.atoms[from].pos.y = model_pos.y;
                    changed = true;
                }
            }
        }
    }

    // Rendering

    let bond_color = Color32::from_rgb(160, 160, 160);
    let bond_thickness = 2.5_f32 * state.zoom;

    // Draw active drag line
    if let (Some(from), Some(pos)) = (state.dragging_from, pointer_pos) {
        if let Tool::Bond(ref _order) = state.current_tool {
            let from_pos = Pos2::new(state.molecule.atoms[from].pos.x, state.molecule.atoms[from].pos.y);
            let screen_from = to_screen(from_pos, state.zoom, state.pan);
            painter.line_segment([screen_from, pos], Stroke::new(bond_thickness, bond_color));
        }
    }

    // Draw bonds
    for bond in &state.molecule.bonds {
        let p1 = Pos2::new(state.molecule.atoms[bond.from].pos.x, state.molecule.atoms[bond.from].pos.y);
        let p2 = Pos2::new(state.molecule.atoms[bond.to].pos.x, state.molecule.atoms[bond.to].pos.y);
        
        let screen_p1 = to_screen(p1, state.zoom, state.pan);
        let screen_p2 = to_screen(p2, state.zoom, state.pan);
        
        let dir = (screen_p2 - screen_p1).normalized();
        let normal = Vec2::new(-dir.y, dir.x);
        
        let stroke = Stroke::new(bond_thickness, bond_color);
        
        match bond.order {
            BondOrder::Single => {
                painter.line_segment([screen_p1, screen_p2], stroke);
            }
            BondOrder::Double => {
                let offset = normal * 3.0 * state.zoom;
                painter.line_segment([screen_p1 + offset, screen_p2 + offset], stroke);
                painter.line_segment([screen_p1 - offset, screen_p2 - offset], stroke);
            }
            BondOrder::Triple => {
                let offset = normal * 4.0 * state.zoom;
                painter.line_segment([screen_p1, screen_p2], stroke);
                painter.line_segment([screen_p1 + offset, screen_p2 + offset], stroke);
                painter.line_segment([screen_p1 - offset, screen_p2 - offset], stroke);
            }
        }
    }

    // Draw atoms
    for (i, atom) in state.molecule.atoms.iter().enumerate() {
        let p = Pos2::new(atom.pos.x, atom.pos.y);
        let screen_p = to_screen(p, state.zoom, state.pan);
        let current_radius = ATOM_RADIUS * state.zoom;
        
        let is_carbon = atom.element == "C";
        
        if state.hovered_atom == Some(i) {
            painter.circle_filled(screen_p, current_radius + (4.0 * state.zoom), Color32::from_rgba_unmultiplied(200, 200, 255, 150));
        }

        if is_carbon {
            // Draw small grey circle for carbon
            painter.circle_filled(screen_p, 4.0 * state.zoom, bond_color);
        } else {
            let key = match atom.element.as_str() {
                "O" => "O",
                "N" => "N",
                "S" => "S",
                "P" => "P",
                "F" | "Cl" | "Br" | "I" => "Halogen",
                "H" => "H",
                _ => "Default",
            };
            
            let hex = state.atom_colors.get(key).map(|s| s.as_str()).unwrap_or("#646464");
            let mut fill_color = Color32::from_rgb(100, 100, 100);
            
            let hex_clean = hex.trim_start_matches('#');
            if hex_clean.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex_clean[0..2], 16),
                    u8::from_str_radix(&hex_clean[2..4], 16),
                    u8::from_str_radix(&hex_clean[4..6], 16)
                ) {
                    fill_color = Color32::from_rgb(r, g, b);
                }
            }
            
            // Draw solid inner fill (no border, no shadow)
            painter.circle_filled(screen_p, current_radius, fill_color);
            
            // Draw text
            painter.text(
                screen_p,
                egui::Align2::CENTER_CENTER,
                &atom.element,
                FontId::proportional(14.0 * state.zoom),
                Color32::WHITE,
            );
        }
    }

    changed
}
