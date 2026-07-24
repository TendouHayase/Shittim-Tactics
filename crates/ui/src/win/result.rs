#[derive(Default)]
#[allow(dead_code)]
pub struct ResultPanel {
    skill_sequence: Vec<SkillStep>,
    total_frames: u64,
    has_result: bool,
}

#[allow(dead_code)]
pub struct SkillStep {
    pub order: usize,
    pub student_name: String,
    pub skill_name: String,
    pub skill_type: String,
    pub cost: u8,
    pub frame: u64,
}

impl ResultPanel {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("Search Results");
        });

        ui.add_space(8.0);

        if !self.has_result {
            ui.colored_label(
                egui::Color32::GRAY,
                "No search results yet. Configure and run a search in the Configuration tab.",
            );
            return;
        }

        ui.horizontal(|ui| {
            ui.label(format!("Total Frames: {}", self.total_frames));
            ui.separator();
            ui.label(format!("Total Skills Used: {}", self.skill_sequence.len()));
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                egui::Grid::new("result_grid")
                    .num_columns(5)
                    .striped(true)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Order");
                        ui.strong("Student");
                        ui.strong("Skill");
                        ui.strong("Type");
                        ui.strong("Frame");
                        ui.end_row();

                        for step in &self.skill_sequence {
                            ui.label(format!("#{}", step.order));
                            ui.label(&step.student_name);
                            ui.label(&step.skill_name);
                            ui.label(&step.skill_type);
                            ui.label(format!("{}", step.frame));
                            ui.end_row();
                        }
                    });
            });
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.skill_sequence.clear();
        self.total_frames = 0;
        self.has_result = false;
    }
}
