use std::sync::mpsc;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    install::{InstallOptions, InstallProgress},
    ui::{done, options, progress, welcome},
};

pub enum Page {
    Welcome,
    Options(InstallOptions),
    Progress(ProgressState),
    Done(DoneState),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operation {
    Install,
    Update,
    Uninstall,
}

pub struct ProgressState {
    pub rx: mpsc::Receiver<InstallProgress>,
    pub log: Vec<String>,
    pub fraction: f32,
    pub finished: bool,
    pub error: Option<String>,
    pub options: InstallOptions,
    pub operation: Operation,
}

pub struct DoneState {
    pub success: bool,
    pub message: String,
    pub install_dir: std::path::PathBuf,
    pub operation: Operation,
}

pub struct InstallerApp {
    pub page: Page,
    pub existing_install: Option<crate::install::ExistingInstall>,
    pub remote_sizes: Arc<Mutex<Option<HashMap<String, u64>>>>,
    pub remote_kadr_version: Arc<Mutex<Option<String>>>,
    pub remote_installer_version: Arc<Mutex<Option<String>>>,
    pub pending_updates: Arc<Mutex<Option<Vec<String>>>>,
    pub patchnotes_text: Arc<Mutex<Option<String>>>,
    pub selected_panel: Option<welcome::Panel>,
    pub installer_dl: Arc<Mutex<welcome::InstallerDlState>>,
}

impl InstallerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let existing_install = crate::install::detect_existing_install();

        let remote_sizes: Arc<Mutex<Option<HashMap<String, u64>>>> = Arc::new(Mutex::new(None));
        let sizes_ref = Arc::clone(&remote_sizes);
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            let sizes = crate::install::fetch_remote_sizes();
            *sizes_ref.lock().unwrap() = Some(sizes);
            ctx.request_repaint();
        });

        let remote_kadr_version: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let ver_ref = Arc::clone(&remote_kadr_version);
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            if let Some(r) = crate::install::fetch_release_version() {
                *ver_ref.lock().unwrap() = Some(r);
                ctx.request_repaint();
            }
        });

        let remote_installer_version: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let inst_ver_ref = Arc::clone(&remote_installer_version);
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            if let Some(r) = crate::install::fetch_installer_release_version() {
                *inst_ver_ref.lock().unwrap() = Some(r);
                ctx.request_repaint();
            }
        });

        let pending_updates: Arc<Mutex<Option<Vec<String>>>> = Arc::new(Mutex::new(None));
        if let Some(existing) = &existing_install {
            let dir = existing.dir.clone();
            let pending_ref = Arc::clone(&pending_updates);
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let filenames = crate::install::get_pending_filenames(&dir);
                *pending_ref.lock().unwrap() = Some(filenames);
                ctx.request_repaint();
            });
        }

        let patchnotes_text: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let notes_ref = Arc::clone(&patchnotes_text);
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            let text = ureq::get("https://bomzh.fm/raw/patchnotes")
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
                .call()
                .ok()
                .and_then(|r| r.into_body().read_to_string().ok())
                .unwrap_or_else(|| "• Could not load patch notes".to_owned());
            *notes_ref.lock().unwrap() = Some(text);
            ctx.request_repaint();
        });

        Self {
            page: Page::Welcome,
            existing_install,
            remote_sizes,
            remote_kadr_version,
            remote_installer_version,
            pending_updates,
            patchnotes_text,
            selected_panel: None,
            installer_dl: Arc::new(Mutex::new(welcome::InstallerDlState::Idle)),
        }
    }
}

impl eframe::App for InstallerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        apply_theme(&ctx);

        let frame = egui::Frame::default().fill(egui::Color32::from_rgb(11, 10, 16));
        egui::CentralPanel::default()
            .frame(frame)
            .show_inside(ui, |ui| {
                let kadr_ver = self.remote_kadr_version.lock().unwrap().clone();
                draw_header(ui, kadr_ver.as_deref());
                ui.add_space(8.0);

                match &mut self.page {
                    Page::Welcome => {
                        let existing_dir = self.existing_install.as_ref().map(|e| e.dir.as_path());
                        let installed_ver = self.existing_install.as_ref().and_then(|e| e.version.as_deref());
                        let notes = self.patchnotes_text.lock().unwrap().clone();
                        let remote_inst_ver = self.remote_installer_version.lock().unwrap().clone();
                        let pending = self.pending_updates.lock().unwrap().clone();
                        let total_size = self.remote_sizes.lock().unwrap().as_ref().map(|m| m.values().sum::<u64>());
                        let installer_dl = self.installer_dl.lock().unwrap().clone();

                        if let Some(action) = welcome::show(
                            ui, existing_dir, installed_ver, total_size,
                            notes.as_deref(), remote_inst_ver.as_deref(),
                            pending.as_deref(), &mut self.selected_panel, &installer_dl,
                        ) {
                            match action {
                                welcome::WelcomeAction::RunUpdate => {
                                    if let Some(existing) = &self.existing_install {
                                        let (tx, rx) = mpsc::channel();
                                        let dir = existing.dir.clone();
                                        let dir2 = dir.clone();
                                        std::thread::spawn(move || {
                                            crate::install::run_update(&dir, tx);
                                        });
                                        let mut opts = InstallOptions::default();
                                        opts.install_dir = dir2;
                                        self.page = Page::Progress(ProgressState {
                                            rx, log: Vec::new(), fraction: 0.0,
                                            finished: false, error: None,
                                            options: opts, operation: Operation::Update,
                                        });
                                    }
                                }
                                welcome::WelcomeAction::DownloadInstaller => {
                                    let dl_ref = Arc::clone(&self.installer_dl);
                                    let ctx = ctx.clone();
                                    std::thread::spawn(move || {
                                        *dl_ref.lock().unwrap() = welcome::InstallerDlState::Downloading;
                                        ctx.request_repaint();
                                        match crate::install::download_installer_to_downloads() {
                                            Ok(path) => *dl_ref.lock().unwrap() = welcome::InstallerDlState::Done(path),
                                            Err(e)   => *dl_ref.lock().unwrap() = welcome::InstallerDlState::Error(e.to_string()),
                                        }
                                        ctx.request_repaint();
                                    });
                                }
                                welcome::WelcomeAction::LaunchInstallerAndExit(path) => {
                                    let _ = std::process::Command::new(&path).spawn();
                                    std::process::exit(0);
                                }
                                welcome::WelcomeAction::GoInstall => {
                                    self.page = Page::Options(InstallOptions::default());
                                }
                                welcome::WelcomeAction::GoUninstall => {
                                    if let Some(existing) = &self.existing_install {
                                        let (tx, rx) = mpsc::channel();
                                        let dir = existing.dir.clone();
                                        let dir2 = dir.clone();
                                        std::thread::spawn(move || {
                                            crate::uninstall::run_uninstall(&dir, tx);
                                        });
                                        let mut opts = InstallOptions::default();
                                        opts.install_dir = dir2;
                                        self.page = Page::Progress(ProgressState {
                                            rx, log: Vec::new(), fraction: 0.0,
                                            finished: false, error: None,
                                            options: opts, operation: Operation::Uninstall,
                                        });
                                    }
                                }
                            }
                        }

                        if matches!(*self.installer_dl.lock().unwrap(), welcome::InstallerDlState::Downloading) {
                            ctx.request_repaint_after(std::time::Duration::from_millis(100));
                        }
                    }

                    Page::Options(opts) => {
                        if let Some(action) = options::show(ui, opts) {
                            match action {
                                options::OptionsAction::Back => {
                                    self.page = Page::Welcome;
                                }
                                options::OptionsAction::Install(opts) => {
                                    let (tx, rx) = mpsc::channel();
                                    let opts_clone = opts.clone();
                                    std::thread::spawn(move || {
                                        crate::install::run_install(&opts_clone, tx);
                                    });
                                    self.page = Page::Progress(ProgressState {
                                        rx, log: Vec::new(), fraction: 0.0,
                                        finished: false, error: None,
                                        options: opts, operation: Operation::Install,
                                    });
                                }
                            }
                        }
                    }

                    Page::Progress(state) => {
                        while let Ok(msg) = state.rx.try_recv() {
                            match msg {
                                InstallProgress::Log(s) => state.log.push(s),
                                InstallProgress::Step(f) => state.fraction = f,
                                InstallProgress::Done => { state.fraction = 1.0; state.finished = true; }
                                InstallProgress::Error(e) => { state.error = Some(e); state.finished = true; }
                            }
                        }

                        if state.finished {
                            ctx.request_repaint();
                        } else {
                            ctx.request_repaint_after(std::time::Duration::from_millis(50));
                        }

                        if let Some(action) = progress::show(ui, state) {
                            match action {
                                progress::ProgressAction::Continue => {
                                    let success = state.error.is_none();
                                    let msg = if success {
                                        match state.operation {
                                            Operation::Install   => "Kadr was installed successfully!".to_owned(),
                                            Operation::Update    => "Kadr was updated successfully!".to_owned(),
                                            Operation::Uninstall => "Kadr was uninstalled successfully.".to_owned(),
                                        }
                                    } else {
                                        format!("{} failed:\n{}",
                                            match state.operation {
                                                Operation::Install   => "Installation",
                                                Operation::Update    => "Update",
                                                Operation::Uninstall => "Uninstall",
                                            },
                                            state.error.as_deref().unwrap_or("unknown error"))
                                    };
                                    let dir = state.options.install_dir.clone();
                                    let op = state.operation;
                                    if op == Operation::Uninstall && success {
                                        self.existing_install = None;
                                    }
                                    self.page = Page::Done(DoneState { success, message: msg, install_dir: dir, operation: op });
                                }
                            }
                        }
                    }

                    Page::Done(state) => {
                        if let Some(action) = done::show(ui, state) {
                            match action {
                                done::DoneAction::Launch => {
                                    let exe = state.install_dir.join("kadr.exe");
                                    let _ = std::process::Command::new(exe).spawn();
                                    std::process::exit(0);
                                }
                                done::DoneAction::Close => std::process::exit(0),
                            }
                        }
                    }
                }
            });
    }
}

fn draw_header(ui: &mut egui::Ui, kadr_version: Option<&str>) {
    let available_w = ui.available_width();
    let height = 52.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(available_w, height), egui::Sense::hover());
    let p = ui.painter();

    p.rect_filled(rect, 0.0, egui::Color32::from_rgb(15, 13, 22));
    p.text(rect.min + egui::vec2(24.0, 12.0), egui::Align2::LEFT_TOP,
        "kadr", egui::FontId::proportional(22.0), egui::Color32::from_rgb(99, 155, 255));
    p.text(rect.min + egui::vec2(74.0, 17.0), egui::Align2::LEFT_TOP,
        "installer", egui::FontId::proportional(13.0), egui::Color32::from_gray(85));

    let ver_text = format!("kadr v{}   installer v{}", kadr_version.unwrap_or("…"), env!("CARGO_PKG_VERSION"));
    p.text(rect.right_center() - egui::vec2(18.0, 0.0), egui::Align2::RIGHT_CENTER,
        &ver_text, egui::FontId::monospace(10.5), egui::Color32::from_gray(80));

    p.hline(rect.left()..=rect.right(), rect.bottom(),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(99, 155, 255, 50)));
}

fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = egui::Color32::from_rgb(11, 10, 16);
    style.visuals.window_fill = egui::Color32::from_rgb(16, 14, 22);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(8, 7, 12);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(22, 20, 30);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(30, 27, 42);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(40, 36, 58);
    style.visuals.override_text_color = Some(egui::Color32::from_gray(210));
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(35));
    ctx.set_global_style(style);
}
