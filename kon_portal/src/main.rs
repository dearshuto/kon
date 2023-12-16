mod application;
use application::Workspace;

use kon_players::{
    clients::{IClient, SampleClient},
    InstrumentType, MemberList,
};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let _workspace = Workspace::new();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "My App",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx
                .set_fonts(eframe::egui::FontDefinitions::default());
            Box::new(App::<SampleClient>::new())
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
struct App<TClient: kon_players::clients::IClient + Default> {
    member_list: MemberList,
    _marker: std::marker::PhantomData<TClient>,
}

impl<TClient> App<TClient>
where
    TClient: IClient + Default,
{
    pub fn new() -> Self {
        let mut client = TClient::default();
        let data = client.fetch().unwrap();
        App {
            member_list: MemberList::from_csv(&data),
            _marker: Default::default(),
        }
    }
}

impl<TClient: kon_players::clients::IClient + Default> eframe::App for App<TClient> {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let _ = ui.radio(true, "Members");
                let _ = ui.radio(false, "Scheduler");
            });

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
