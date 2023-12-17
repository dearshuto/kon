mod application;
mod mock;

use application::{ContentType, Workspace};
use egui_extras::{Column, TableBuilder};
use kon_rs::InstrumentType;
use mock::MockClient;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "My App",
        native_options,
        Box::new(|cc| {
            let mut font_definitions = eframe::egui::FontDefinitions::default();
            let font_data = eframe::egui::FontData::from_static(include_bytes!(
                "../../resources/fonts/NotoSansJP-Regular.ttf"
            ));

            font_definitions
                .font_data
                .insert("jp_font".to_owned(), font_data);
            font_definitions
                .families
                .get_mut(&eframe::egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "jp_font".to_owned());
            cc.egui_ctx.set_fonts(font_definitions);

            Box::new(App::new())
        }),
    )
    .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|_cc| Box::new(App::<SampleClient>::new())),
            )
            .await
            .expect("failed to start eframe");
    });
}
struct App {
    workspace: Workspace<MockClient>,
    content_type: ContentType,
    instrument_filter: InstrumentType,
}

impl App {
    pub fn new() -> Self {
        let mock = MockClient::new();
        let workspace = Workspace::new(mock);

        App {
            workspace,
            content_type: ContentType::Members,
            instrument_filter: InstrumentType::empty(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.workspace.update();

        let mut content_type = self.content_type;
        eframe::egui::TopBottomPanel::top("Top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut content_type, ContentType::Members, "Members");
                ui.selectable_value(&mut content_type, ContentType::Schedule, "Schedule");
            });
        });

        match content_type {
            ContentType::Members => self.draw_members(ctx, frame),
            ContentType::Schedule => self.draw_schedule(ctx, frame),
        }
        self.content_type = content_type;
    }
}

impl App {
    fn draw_members(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            // チェックボックスで担当楽器のフィルターを表示
            let filter_list = vec![
                ("Vocal", InstrumentType::VOCAL),
                ("AcounsticGuitar", InstrumentType::ACOUSTIC_GUITAR),
                ("ElectricGuitar", InstrumentType::ELECTRIC_GUITAR),
                ("ElectricBass", InstrumentType::ELECTRIC_BASS),
                ("Keyboard", InstrumentType::KEYBOARD),
            ];

            for (label, filter) in filter_list {
                let mut is_enabled = self.instrument_filter.contains(filter);
                if ui.checkbox(&mut is_enabled, label).changed() {
                    if is_enabled {
                        self.instrument_filter.insert(filter);
                    } else {
                        self.instrument_filter.remove(filter);
                    }
                }
            }

            let filter = self.instrument_filter;
            self.workspace
                .for_each_user_with_filter(filter, |_id, user| {
                    let Some(user) = user else {
                        return;
                    };
                    ui.label(&user.name);
                });
        });
    }

    #[allow(dead_code)]
    fn draw_schedule(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto());
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Time");
                    });
                    header.col(|ui| {
                        ui.strong("Room1");
                    });
                    header.col(|ui| {
                        ui.strong("Room2");
                    });
                })
                .body(|mut body| {
                    body.row(50.0, |mut row| {
                        row.col(|ui| {
                            ui.label("12:00-13:00");
                        });
                        row.col(|ui| {
                            ui.label("Cool Band");
                        });
                        row.col(|ui| {
                            ui.label("Pank Band");
                        });
                    });
                });
        });

        eframe::egui::SidePanel::right("Band List").show(ctx, |ui| {
            ui.label("Gt. Member");
            ui.label("Ba. Member");
        });
    }
}
