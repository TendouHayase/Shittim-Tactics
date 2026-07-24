pub struct ConfigPanel {
    boss_selected: usize,
    difficulty_selected: usize,
    attack_type_selected: usize,
    terrain_selected: usize,
    algorithm_selected: usize,
    threshold: f64,
    is_searching: bool,
    search_progress: f32,
}

impl Default for ConfigPanel {
    fn default() -> Self {
        Self {
            boss_selected: 0,
            difficulty_selected: 4,
            attack_type_selected: 0,
            terrain_selected: 0,
            algorithm_selected: 0,
            threshold: 1.0,
            is_searching: false,
            search_progress: 0.0,
        }
    }
}

impl ConfigPanel {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        ui.group(|ui| {
            self.show_boss_config(ui);
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            self.show_team_config(ui);
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            self.show_search_control(ui);
        });

        ui.add_space(8.0);
    }

    fn show_boss_config(&mut self, ui: &mut egui::Ui) {
        ui.heading("Boss Configuration");

        egui::Grid::new("boss_config_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Boss:");
                egui::ComboBox::from_id_salt("boss_combo")
                    .selected_text(BOSS_NAMES[self.boss_selected])
                    .show_ui(ui, |ui| {
                        for (i, name) in BOSS_NAMES.iter().enumerate() {
                            ui.selectable_value(&mut self.boss_selected, i, *name);
                        }
                    });
                ui.end_row();

                ui.label("Difficulty:");
                egui::ComboBox::from_id_salt("difficulty_combo")
                    .selected_text(DIFFICULTY_NAMES[self.difficulty_selected])
                    .show_ui(ui, |ui| {
                        for (i, name) in DIFFICULTY_NAMES.iter().enumerate() {
                            ui.selectable_value(&mut self.difficulty_selected, i, *name);
                        }
                    });
                ui.end_row();

                ui.label("Attack Type:");
                egui::ComboBox::from_id_salt("attack_type_combo")
                    .selected_text(ATTACK_TYPE_NAMES[self.attack_type_selected])
                    .show_ui(ui, |ui| {
                        for (i, name) in ATTACK_TYPE_NAMES.iter().enumerate() {
                            ui.selectable_value(&mut self.attack_type_selected, i, *name);
                        }
                    });
                ui.end_row();

                ui.label("Terrain:");
                egui::ComboBox::from_id_salt("terrain_combo")
                    .selected_text(TERRAIN_NAMES[self.terrain_selected])
                    .show_ui(ui, |ui| {
                        for (i, name) in TERRAIN_NAMES.iter().enumerate() {
                            ui.selectable_value(&mut self.terrain_selected, i, *name);
                        }
                    });
                ui.end_row();

                ui.label("Goal Threshold:");
                ui.add(
                    egui::DragValue::new(&mut self.threshold)
                        .speed(0.01)
                        .range(0.0..=1.0),
                );
                ui.end_row();
            });
    }

    fn show_team_config(&mut self, ui: &mut egui::Ui) {
        ui.heading("Team Configuration");

        ui.horizontal(|ui| {
            ui.label("Algorithm:");
            egui::ComboBox::from_id_salt("algorithm_combo")
                .selected_text(ALGORITHM_NAMES[self.algorithm_selected])
                .show_ui(ui, |ui| {
                    for (i, name) in ALGORITHM_NAMES.iter().enumerate() {
                        ui.selectable_value(&mut self.algorithm_selected, i, *name);
                    }
                });
        });

        ui.add_space(4.0);
        ui.label(format!(
            "Configure up to {} students for the raid.",
            MAX_STUDENT_COUNT
        ));

        egui::Grid::new("team_grid")
            .num_columns(4)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Slot");
                ui.strong("Student");
                ui.strong("Position");
                ui.strong("Skill Levels (Ex / Basic / Enhanced / Sub)");
                ui.end_row();

                for slot in 0..MAX_STUDENT_COUNT {
                    ui.label(format!("#{}", slot + 1));
                    ui.colored_label(egui::Color32::GRAY, "(select student)");
                    ui.colored_label(egui::Color32::GRAY, "(x, y)");
                    ui.colored_label(egui::Color32::GRAY, "? / ? / ? / ?");
                    ui.end_row();
                }
            });
    }

    fn show_search_control(&mut self, ui: &mut egui::Ui) {
        ui.heading("Search Control");

        ui.horizontal(|ui| {
            let button_text = if self.is_searching {
                "Searching..."
            } else {
                "Start Search"
            };

            if ui
                .add_enabled(!self.is_searching, egui::Button::new(button_text))
                .clicked()
            {
                self.is_searching = true;
                self.search_progress = 0.0;
            }

            if ui
                .add_enabled(self.is_searching, egui::Button::new("Cancel"))
                .clicked()
            {
                self.is_searching = false;
                self.search_progress = 0.0;
            }
        });

        if self.is_searching {
            self.search_progress = (self.search_progress + 0.002).min(0.99);
            ui.add_space(4.0);
            ui.add(
                egui::ProgressBar::new(self.search_progress)
                    .text("Searching...")
                    .animate(true),
            );
        }
    }
}

const MAX_STUDENT_COUNT: usize = 6;

const BOSS_NAMES: &[&str] = &["Binah (Decagrammaton)", "Goz (Slumpia)"];

const DIFFICULTY_NAMES: &[&str] = &[
    "Normal",
    "Hard",
    "Very Hard",
    "Hardcore",
    "Extreme",
    "Insane",
    "Torment",
    "Lunatic",
];

const ATTACK_TYPE_NAMES: &[&str] = &[
    "Normal",
    "Explosive",
    "Piercing",
    "Corrosive",
    "Mystic",
    "Sonic",
];

const TERRAIN_NAMES: &[&str] = &["Street", "Outdoor", "Indoor"];

const ALGORITHM_NAMES: &[&str] = &["A*", "Beam Search (WIP)"];
