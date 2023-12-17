mod application;
mod mock;

use application::Workspace;
use mock::MockClient;

use kon_players::{InstrumentType, MemberList};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "My App",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx
                .set_fonts(eframe::egui::FontDefinitions::default());
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
    #[allow(dead_code)]
    workspace: Workspace<MockClient>,
    member_list: MemberList,
}

impl App {
    pub fn new() -> Self {
        let mock = MockClient::new();
        let workspace = Workspace::new(mock);

        App {
            workspace,
            // TODO: Workspace から取得するように修正する
            member_list: MemberList::from_csv(
                "name,property_name,value
            shikama_shuto,instrument,ElectricBass
            edogawa_conan,instrument,Vocal
            edogawa_conan,instrument,Keyboard
            akai_shuichi,instrument,Vocal
            akai_shuichi,instrument,ElectricGuitar
            okiya_subaru,instrument,Drums
            hattori_heiji,instrument,TenorSaxphone
            hattori_heiji,instrument,Tronbone",
            ),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.workspace.update();

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let _ = ui.radio(true, "Members");
                let _ = ui.radio(false, "Scheduler");
            });

            // 名簿表示
            self.draw_members(ctx, frame);
        });
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
            for item in filter_list {
                let label = item.0;
                let filter = item.1;
                let mut is_enabled = self.member_list.filter().contains(filter);
                if ui.checkbox(&mut is_enabled, label).changed() {
                    if is_enabled {
                        self.member_list.add_filter(filter);
                    } else {
                        self.member_list.remove_filter(filter);
                    }
                }
            }

            // フィルターが指定されてなければ全部表示
            if self.member_list.filter().is_empty() {
                for member in self.member_list.members() {
                    ui.label(member.name());
                }
            } else {
                for member in self.member_list.members() {
                    if (member.instruments() & self.member_list.filter()).bits() != 0 {
                        ui.label(member.name());
                    }
                }
            }
        });
    }
}
