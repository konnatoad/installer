use crate::install::Channel;
use crate::theme;
use egui::{Color32, RichText, Stroke, Ui};
use std::path::Path;

#[derive(PartialEq, Clone, Copy)]
pub enum Panel {
    Kadr,
    Installer,
    Dependencies,
    Uninstall,
}

#[derive(Clone)]
pub enum InstallerDlState {
    Idle,
    Downloading,
    Done(std::path::PathBuf),
    Error(String),
}

pub enum WelcomeAction {
    RunUpdate,
    DownloadInstaller,
    LaunchInstallerAndExit(std::path::PathBuf),
    GoInstall,
    GoUninstall,
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut Ui,
    existing_dir: Option<&Path>,
    installed_kadr_version: Option<&str>,
    total_install_size: Option<u64>,
    patchnotes: Option<&str>,
    remote_installer_version: Option<&str>,
    pending_updates: Option<&[String]>,
    selected_panel: &mut Option<Panel>,
    installer_dl: &InstallerDlState,
    channel: &mut Channel,
    stored_channel: Channel,
) -> Option<WelcomeAction> {
    let mut action = None;

    if existing_dir.is_none() {
        return show_fresh_install(ui, total_install_size, patchnotes, &mut action);
    }

    let channel_switch_pending = *channel != stored_channel;
    let kadr_outdated = if channel_switch_pending {
        Some(true)
    } else {
        pending_updates.map(|p| p.iter().any(|f| f == "kadr.exe"))
    };
    let deps_pending: Vec<&str> = pending_updates
        .unwrap_or(&[])
        .iter()
        .filter(|f| f.as_str() != "kadr.exe")
        .map(|f| f.as_str())
        .collect();
    let deps_outdated = pending_updates.map(|_| !deps_pending.is_empty());
    let has_updates =
        pending_updates.map(|p| !p.is_empty()).unwrap_or(false) || channel_switch_pending;

    let inst_current = env!("CARGO_PKG_VERSION");
    let installer_outdated = remote_installer_version.map(|r| r != inst_current);

    let sidebar_w = 160.0;
    let avail_h = ui.available_height();

    ui.horizontal(|ui| {
        egui::Frame::new().fill(theme::SURFACE).show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.set_width(sidebar_w);
                ui.set_min_height(avail_h);
                ui.add_space(12.0);

                let btn_label = if channel_switch_pending {
                    "Switch Channel".to_owned()
                } else {
                    match pending_updates {
                        None => "Update All  …".to_owned(),
                        Some([]) => "Up to date".to_owned(),
                        Some(p) => format!("Update All  ({})", p.len()),
                    }
                };
                let btn_active = has_updates;
                let btn_color = if btn_active {
                    theme::ACCENT
                } else {
                    theme::SURFACE4
                };
                if sidebar_btn(ui, sidebar_w - 20.0, &btn_label, btn_color) && btn_active {
                    action = Some(WelcomeAction::RunUpdate);
                }

                ui.add_space(14.0);
                sidebar_divider(ui, sidebar_w);
                ui.add_space(8.0);

                let items: &[(Panel, &str)] = &[
                    (Panel::Kadr, "Kadr"),
                    (Panel::Installer, "Installer"),
                    (Panel::Dependencies, "Dependencies"),
                ];
                for (panel, label) in items {
                    let dot = status_dot(match *panel {
                        Panel::Kadr => kadr_outdated,
                        Panel::Installer => installer_outdated,
                        Panel::Dependencies => deps_outdated,
                        Panel::Uninstall => None,
                    });
                    if nav_item(ui, sidebar_w, label, dot, selected_panel == &Some(*panel)) {
                        *selected_panel = if *selected_panel == Some(*panel) {
                            None
                        } else {
                            Some(*panel)
                        };
                    }
                }

                ui.add_space(8.0);
                sidebar_divider(ui, sidebar_w);
                ui.add_space(8.0);

                if nav_item(
                    ui,
                    sidebar_w,
                    "Uninstall",
                    None,
                    *selected_panel == Some(Panel::Uninstall),
                ) {
                    *selected_panel = if *selected_panel == Some(Panel::Uninstall) {
                        None
                    } else {
                        Some(Panel::Uninstall)
                    };
                }
            });
        });

        let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, avail_h), egui::Sense::hover());
        ui.painter().rect_filled(sep_rect, 0.0, theme::BORDER);

        ui.vertical(|ui| {
            ui.add_space(16.0);
            match *selected_panel {
                None => {
                    show_patchnotes(ui, patchnotes);
                }
                Some(Panel::Kadr) => {
                    if let Some(a) = show_kadr_panel(
                        ui,
                        installed_kadr_version,
                        kadr_outdated,
                        channel,
                        stored_channel,
                    ) {
                        action = Some(a);
                    }
                }
                Some(Panel::Installer) => {
                    if let Some(a) = show_installer_panel(
                        ui,
                        inst_current,
                        remote_installer_version,
                        installer_outdated,
                        installer_dl,
                    ) {
                        action = Some(a);
                    }
                }
                Some(Panel::Dependencies) => {
                    if let Some(a) = show_deps_panel(ui, deps_outdated, &deps_pending) {
                        action = Some(a);
                    }
                }
                Some(Panel::Uninstall) => {
                    if let Some(a) = show_uninstall_panel(ui) {
                        action = Some(a);
                    }
                }
            }
        });
    });

    action
}

fn show_fresh_install(
    ui: &mut Ui,
    total_install_size: Option<u64>,
    patchnotes: Option<&str>,
    action: &mut Option<WelcomeAction>,
) -> Option<WelcomeAction> {
    let avail_w = ui.available_width();
    let total_w = (avail_w - 40.0).min(560.0);
    let left_pad = (avail_w - total_w) / 2.0;

    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("Kadr Image Viewer")
                .size(22.0)
                .color(theme::TEXT)
                .strong(),
        );
        ui.add_space(4.0);
        let size_str = match total_install_size {
            Some(s) => format!("~{} · requires internet connection", fmt_size(s)),
            None => "Requires internet connection · calculating size…".to_owned(),
        };
        ui.label(RichText::new(size_str).size(11.0).color(theme::TEXT_MUTED));
    });

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.add_space(left_pad);
        ui.vertical(|ui| {
            ui.set_width(total_w);

            // Patchnotes
            let notes_text = patchnotes.unwrap_or("Loading…");
            egui::Frame::new()
                .fill(theme::SURFACE)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(4.0)
                .inner_margin(egui::Margin {
                    left: 12,
                    right: 12,
                    top: 8,
                    bottom: 8,
                })
                .show(ui, |ui| {
                    ui.set_width(total_w - 24.0);
                    for line in notes_text.lines() {
                        let t = line.trim();
                        if t.is_empty() {
                            continue;
                        }
                        ui.label(RichText::new(t).size(11.5).color(theme::TEXT_DIM));
                    }
                });

            ui.add_space(14.0);

            if content_btn(ui, total_w, "Install Kadr", theme::ACCENT) {
                *action = Some(WelcomeAction::GoInstall);
            }
        });
    });

    action.take()
}

fn show_patchnotes(ui: &mut Ui, patchnotes: Option<&str>) {
    let notes_text = patchnotes.unwrap_or("Loading…");
    let w = ui.available_width() - 20.0;
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(4.0)
        .inner_margin(egui::Margin {
            left: 12,
            right: 12,
            top: 8,
            bottom: 8,
        })
        .show(ui, |ui| {
            ui.set_width(w - 24.0);
            for line in notes_text.lines() {
                let t = line.trim();
                if t.is_empty() {
                    continue;
                }
                ui.label(RichText::new(t).size(11.5).color(theme::TEXT_DIM));
            }
        });
}

fn show_kadr_panel(
    ui: &mut Ui,
    installed_version: Option<&str>,
    outdated: Option<bool>,
    channel: &mut Channel,
    stored_channel: Channel,
) -> Option<WelcomeAction> {
    let mut action = None;
    let w = ui.available_width() - 20.0;

    panel_title(ui, "Kadr");

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Channel").size(12.0).color(theme::TEXT_DIM));
        egui::ComboBox::from_id_salt("kadr_channel")
            .selected_text(channel.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(channel, Channel::Stable, "Stable");
                ui.selectable_value(channel, Channel::Beta, "Beta");
            });
    });

    let switching = *channel != stored_channel;
    let (status_text, status_color) = match outdated {
        None => ("Checking for updates…".to_owned(), theme::TEXT_MUTED),
        Some(false) => (
            format!("v{} — Up to date", installed_version.unwrap_or("?")),
            theme::SUCCESS,
        ),
        Some(true) if switching => (
            format!(
                "v{} installed — switch to {} available",
                installed_version.unwrap_or("?"),
                channel.label()
            ),
            theme::ERROR_TEXT,
        ),
        Some(true) => (
            format!("v{} — Update available", installed_version.unwrap_or("?")),
            theme::ERROR_TEXT,
        ),
    };
    ui.add_space(8.0);
    ui.label(RichText::new(status_text).size(13.0).color(status_color));

    if outdated == Some(true) {
        ui.add_space(16.0);
        let label = if switching {
            "Switch Channel"
        } else {
            "Update Kadr"
        };
        if content_btn(ui, w, label, theme::ACCENT) {
            action = Some(WelcomeAction::RunUpdate);
        }
    }

    action
}

fn show_installer_panel(
    ui: &mut Ui,
    current_version: &str,
    remote_version: Option<&str>,
    outdated: Option<bool>,
    dl_state: &InstallerDlState,
) -> Option<WelcomeAction> {
    let mut action = None;
    let w = ui.available_width() - 20.0;

    panel_title(ui, "Installer");

    let (status_text, status_color) = match outdated {
        None => (format!("v{current_version} — checking…"), theme::TEXT_MUTED),
        Some(false) => (format!("v{current_version} — Up to date"), theme::SUCCESS),
        Some(true) => {
            let new_ver = remote_version.unwrap_or("?");
            (
                format!("v{current_version} -> v{new_ver} available"),
                theme::ERROR_TEXT,
            )
        }
    };
    ui.add_space(8.0);
    ui.label(RichText::new(status_text).size(13.0).color(status_color));
    ui.add_space(4.0);
    ui.label(
        RichText::new("The new installer will be saved to your Downloads folder.")
            .size(11.0)
            .color(theme::TEXT_MUTED),
    );

    ui.add_space(16.0);

    match dl_state {
        InstallerDlState::Idle => {
            if outdated != Some(false) && content_btn(ui, w, "Download Installer", theme::ACCENT2) {
                action = Some(WelcomeAction::DownloadInstaller);
            }
        }
        InstallerDlState::Downloading => {
            ui.label(
                RichText::new("Downloading…")
                    .size(13.0)
                    .color(theme::TEXT_DIM),
            );
        }
        InstallerDlState::Done(path) => {
            ui.label(
                RichText::new("Download complete")
                    .size(13.0)
                    .color(theme::SUCCESS),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(path.display().to_string())
                    .size(10.5)
                    .color(theme::TEXT_MUTED)
                    .monospace(),
            );
            ui.add_space(12.0);
            if content_btn(ui, w, "Launch New Installer", theme::ACCENT) {
                action = Some(WelcomeAction::LaunchInstallerAndExit(path.clone()));
            }
        }
        InstallerDlState::Error(e) => {
            ui.label(
                RichText::new(format!("Download failed: {e}"))
                    .size(12.0)
                    .color(theme::ERROR_TEXT),
            );
            ui.add_space(12.0);
            if content_btn(ui, w, "Retry", theme::ACCENT2) {
                action = Some(WelcomeAction::DownloadInstaller);
            }
        }
    }

    action
}

fn show_deps_panel(ui: &mut Ui, outdated: Option<bool>, pending: &[&str]) -> Option<WelcomeAction> {
    let mut action = None;
    let w = ui.available_width() - 20.0;

    panel_title(ui, "Dependencies");

    ui.add_space(8.0);

    match outdated {
        None => {
            ui.label(
                RichText::new("Checking…")
                    .size(13.0)
                    .color(theme::TEXT_MUTED),
            );
        }
        Some(false) => {
            ui.label(
                RichText::new("All dependencies up to date")
                    .size(13.0)
                    .color(theme::SUCCESS),
            );
        }
        Some(true) => {
            ui.label(
                RichText::new("Updates available:")
                    .size(13.0)
                    .color(theme::ERROR_TEXT),
            );
            ui.add_space(6.0);
            for name in pending {
                ui.label(
                    RichText::new(format!("  • {name}"))
                        .size(12.0)
                        .color(theme::TEXT_DIM),
                );
            }
            ui.add_space(16.0);
            if content_btn(ui, w, "Update Dependencies", theme::ACCENT) {
                action = Some(WelcomeAction::RunUpdate);
            }
        }
    }

    action
}

fn show_uninstall_panel(ui: &mut Ui) -> Option<WelcomeAction> {
    let mut action = None;
    let w = ui.available_width() - 20.0;

    panel_title(ui, "Uninstall");

    ui.add_space(8.0);
    ui.label(
        RichText::new("This will remove Kadr and all its files from this computer.")
            .size(12.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(20.0);

    if content_btn(ui, w, "Uninstall Kadr", theme::ERROR_TEXT) {
        action = Some(WelcomeAction::GoUninstall);
    }

    action
}

fn sidebar_btn(ui: &mut Ui, width: f32, label: &str, color: Color32) -> bool {
    let height = 36.0;
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
        let bg = if resp.hovered() {
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 55)
        } else {
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 30)
        };
        let stroke_alpha = if resp.hovered() { 120u8 } else { 60u8 };
        ui.painter().rect(
            rect,
            5.0,
            bg,
            Stroke::new(
                1.0,
                Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), stroke_alpha),
            ),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(12.0),
            theme::TEXT,
        );
        resp.clicked()
    })
    .inner
}

fn nav_item(ui: &mut Ui, width: f32, label: &str, dot: Option<Color32>, selected: bool) -> bool {
    let height = 34.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    let bg = if selected {
        theme::accent_fill(18)
    } else if resp.hovered() {
        theme::white_wash(8)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 0.0, bg);

    if selected {
        let bar = egui::Rect::from_min_size(rect.min, egui::vec2(3.0, height));
        ui.painter().rect_filled(bar, 0.0, theme::ACCENT);
    }

    let text_color = if selected {
        theme::TEXT
    } else {
        theme::TEXT_DIM
    };
    ui.painter().text(
        egui::pos2(rect.min.x + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.5),
        text_color,
    );

    if let Some(dot_color) = dot {
        ui.painter().circle_filled(
            egui::pos2(rect.max.x - 14.0, rect.center().y),
            4.0,
            dot_color,
        );
    }

    resp.clicked()
}

fn sidebar_divider(ui: &mut Ui, width: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER);
}

fn panel_title(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).size(18.0).color(theme::TEXT).strong());
    let rect = ui.cursor();
    let line = egui::Rect::from_min_size(
        egui::pos2(rect.min.x, rect.min.y),
        egui::vec2(ui.available_width() - 20.0, 1.0),
    );
    ui.painter().rect_filled(line, 0.0, theme::BORDER);
    ui.add_space(2.0);
}

fn content_btn(ui: &mut Ui, width: f32, label: &str, accent: Color32) -> bool {
    let height = 38.0;
    let w = width.min(300.0);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, height), egui::Sense::click());
    let bg = if resp.hovered() {
        Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 55)
    } else {
        Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), 30)
    };
    let stroke_alpha = if resp.hovered() { 160u8 } else { 80u8 };
    ui.painter().rect(
        rect,
        5.0,
        bg,
        Stroke::new(
            1.0,
            Color32::from_rgba_premultiplied(accent.r(), accent.g(), accent.b(), stroke_alpha),
        ),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        theme::TEXT,
    );
    resp.clicked()
}

fn status_dot(outdated: Option<bool>) -> Option<Color32> {
    match outdated {
        None => None,
        Some(false) => Some(theme::SUCCESS),
        Some(true) => Some(theme::ERROR_TEXT),
    }
}

fn fmt_size(bytes: u64) -> String {
    if bytes < 1_000_000 {
        format!("{} KB", (bytes + 500) / 1_000)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    }
}
