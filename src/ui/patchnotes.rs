use egui::{Color32, RichText, Stroke, Ui};

pub enum PatchNotesAction {
    Confirm,
    Back,
}

pub fn show(ui: &mut Ui, size_delta: Option<i64>, all_up_to_date: bool, kadr_version: Option<&str>, _patchnotes: Option<&str>) -> Option<PatchNotesAction> {
    let mut action = None;

    ui.add_space(16.0);

    ui.vertical_centered(|ui| {
        let ver = kadr_version.unwrap_or("…");
        let title = if all_up_to_date {
            format!("kadr v{ver} — Already up to date")
        } else {
            format!("Update to v{ver}")
        };
        ui.label(
            RichText::new(title)
                .size(20.0)
                .color(Color32::from_gray(225))
                .strong(),
        );
        if !all_up_to_date {
            ui.add_space(4.0);
            let delta_str = match size_delta {
                Some(d) if d > 0  => format!("Installation size change: +{}", fmt_delta(d as u64)),
                Some(d) if d < 0  => format!("Installation size change: -{}", fmt_delta(d.unsigned_abs())),
                Some(_)            => "Installation size unchanged".to_owned(),
                None               => "Calculating size…".to_owned(),
            };
            ui.label(
                RichText::new(delta_str)
                    .size(11.0)
                    .color(Color32::from_gray(70)),
            );
        }
    });

    ui.add_space(24.0);

    let btn_w = 400.0;
    let left_pad = (ui.available_width() - btn_w) / 2.0;

    ui.horizontal(|ui| {
        ui.add_space(left_pad);
        let half = (btn_w - 8.0) / 2.0;

        if small_btn(ui, half, "Back", Color32::from_rgb(80, 70, 100), Color32::from_rgba_premultiplied(80, 70, 100, 40)) {
            action = Some(PatchNotesAction::Back);
        }
        if !all_up_to_date {
            ui.add_space(8.0);
            if small_btn(ui, half, "Confirm Update", Color32::from_rgb(99, 155, 255), Color32::from_rgba_premultiplied(99, 155, 255, 40)) {
                action = Some(PatchNotesAction::Confirm);
            }
        }
    });

    action
}

fn fmt_delta(bytes: u64) -> String {
    if bytes < 1_000_000 {
        format!("{} KB", (bytes + 500) / 1_000)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    }
}

fn small_btn(ui: &mut Ui, width: f32, label: &str, accent: Color32, bg: Color32) -> bool {
    let height = 40.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    let fill = if response.hovered() {
        Color32::from_rgba_premultiplied(
            bg.r().saturating_add(20),
            bg.g().saturating_add(20),
            bg.b().saturating_add(20),
            bg.a().saturating_add(15),
        )
    } else {
        bg
    };

    let stroke_alpha = if response.hovered() { 160u8 } else { 60u8 };
    let stroke_color = Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), stroke_alpha);

    ui.painter().rect(rect, 6.0, fill, Stroke::new(1.0, stroke_color), egui::StrokeKind::Outside);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        Color32::from_gray(210),
    );

    response.clicked()
}
