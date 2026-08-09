use eframe::egui;
use eframe::egui::{Color32, Sense, Stroke};
use catan::{Cost, Resource};
use crate::panels::board::BuildMode;
use crate::theme;
use crate::theme::resource_color;

pub(crate) fn action_button(
    ui: &mut egui::Ui,
    mode: BuildMode,
    current: BuildMode,
    cost: &Cost,
    affordable: bool,
    color: Color32,
) -> egui::Response {
    const S: f32 = theme::BUTTON_W_H;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(S, S + 18.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool(response.id, response.hovered() && affordable);
    let painter = ui.painter_at(rect);

    let selected = mode == current;
    let bg = if selected {
        Color32::from_gray(80)
    } else {
        Color32::from_gray(38 + (14.0 * t) as u8)
    };
    let body =
        egui::Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - S), egui::vec2(S, S));
    painter.rect_filled(body, 8.0, bg);
    if selected {
        painter.rect_stroke(body, 8.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);
    }

    // La pièce elle-même, dans la couleur du joueur — la même forme que sur le plateau.
    let c = body.center();
    let piece = if affordable {
        color
    } else {
        color.gamma_multiply(0.35)
    };
    match mode {
        BuildMode::Road => {
            painter.line_segment(
                [c + egui::vec2(-16.0, 8.0), c + egui::vec2(16.0, -8.0)],
                Stroke::new(7.0, piece),
            );
            ()
        }
        BuildMode::Settlement => {
            painter.circle_filled(c, 12.0, piece);
            ()
        }
        BuildMode::City => {
            painter.rect_filled(
                egui::Rect::from_center_size(c, egui::vec2(24.0, 24.0)),
                3.0,
                piece,
            );
            ()
        }
        BuildMode::None => (),
    }

    // Le coût, en pastilles de la couleur des terrains.
    let dots: Vec<Resource> = Resource::ALL
        .iter()
        .flat_map(|&r| std::iter::repeat(r).take(cost.amount(r) as usize))
        .collect();
    let dw = 11.0;
    let start = rect.center().x - (dots.len() as f32 - 1.0) * dw * 0.5;
    for (i, &r) in dots.iter().enumerate() {
        let p = egui::pos2(start + i as f32 * dw, rect.top() + 8.0);
        let col = if affordable {
            resource_color(r)
        } else {
            resource_color(r).gamma_multiply(0.4)
        };
        painter.circle_filled(p, 4.5, col);
    }

    response
}