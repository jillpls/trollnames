use crate::data_processing::{Name, generate_data, NameSegment};
use crate::name_gen::{generate_names_from_parts, GeneratedName};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct NameApp {
    names: Vec<Name>,
    syllables: Vec<NameSegment>,
    parts: Vec<NameSegment>,
    #[serde(skip)]
    generated: Vec<(GeneratedName, bool)>,
    #[serde(skip)]
    omit_reserved: bool,
    #[serde(skip)]
    length: f32,
    #[serde(skip)]
    amount: usize,
}

impl Default for NameApp {
    fn default() -> Self {
        Self {
            names: vec![],
            syllables: vec![],
            parts: vec![],
            generated: vec![],
            omit_reserved: true,
            length: 2.5,
            amount: 5,
        }
    }
}

impl NameApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            let mut r: NameApp = Default::default();
            r.load_from_files();
            r
        }
    }

    fn load_from_files(&mut self) {
        let (s, p, n) = generate_data();
        self.syllables = s.iter().map(|o| o.into()).collect();
        self.parts = p.iter().map(|p| p.into()).collect();
        self.names = n;
    }
}

impl eframe::App for NameApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:

        //     egui::MenuBar::new().ui(ui, |ui| {
        //         // NOTE: no File->Quit on web pages!
        //         let is_web = cfg!(target_arch = "wasm32");
        //         if !is_web {
        //             ui.menu_button("File", |ui| {
        //                 if ui.button("Quit").clicked() {
        //                     ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        //                 }
        //             });
        //             ui.add_space(16.0);
        //         }

        //         egui::widgets::global_theme_preference_buttons(ui);
        //     });
        // });


        // if *expanded {
        //     ui.label("Derived from:");
        //     for name in r {
        //         ui.horizontal(|ui| {
        //             ui.label(format!("{} - (", name));
        //             ui.hyperlink(format!(
        //                 "https://wowpedia.fandom.com/wiki/{}",
        //                 urlencoding::encode(name)
        //             ));
        //             ui.label(")");
        //         });
        //     }
        //     ui.separator();
        // }


        egui::TopBottomPanel::top("Settings").show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Troll Name Generator");

            ui.label(format!("Names: {}", self.names.len()));
            ui.label(format!("Syllables: {}", self.syllables.len()));
            ui.label(format!("Name Parts: {}", self.parts.len()));
            if ui.button("Reload Data").clicked() {
                self.load_from_files();
            }

            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.length, 1.0..=4.0)
                    .text("Length")
                    .step_by(0.1),
            );
            ui.add(
                egui::Slider::new(&mut self.amount, 1..=50)
                    .logarithmic(true)
                    .text("Amount"),
            );

            ui.checkbox(
                &mut self.omit_reserved,
                "Omit reserved (jin, fon, zul, zen)",
            );

            if ui.button("Generate Names").clicked() {
                self.generated = generate_names_from_parts(
                    &self.parts,
                    &self.syllables,
                    &self.names,
                    self.amount,
                    self.omit_reserved,
                    self.length,
                )
                .into_iter()
                .map(|v| (v, false))
                .collect();
            }
        });
        let mut selected = None;

        if let Some((i, (n, _))) = self.generated.iter().enumerate().find(|(_, (_,e))| *e) {
        egui::SidePanel::right("test").show(ctx, | ui | {
            selected = Some(i);
            ui.heading(n.to_string());
            ui.label("Derived from:");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for name in n.elements.iter().map(|s| s.derived_names.iter()).flatten() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} - (", name));
                        ui.hyperlink(format!(
                            "https://wowpedia.fandom.com/wiki/{}",
                            urlencoding::encode(name)
                        ));
                        ui.label(")");
                    });
                }
            });
        });

        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.generated.len() > 0 {
                let mut changed = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, (n, expanded)) in self.generated.iter_mut().enumerate() {
                        let gender_text = match n.gender() {
                            ..0.05 => "female",
                            ..0.25 => "likely female",
                            ..0.35 => "slightly female",
                            ..0.65 => "neutral",
                            ..0.75 => "slightly male",
                            ..0.95 => "likely male",
                            _ => "male",
                        };
                        ui.horizontal(|ui| {
                            ui.strong(n.to_string());
                            ui.label(" - ");
                            ui.label(gender_text);
                            if ui.add(egui::Button::new("Details >>").selected(*expanded)).clicked() {
                                *expanded = !*expanded;
                                changed = Some(i);
                            }
                        });
                    }
                });
                if let (Some(prev), Some(selected)) = (selected, changed) {
                    if prev != selected {
                        self.generated[prev].1 = false;
                    }
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
