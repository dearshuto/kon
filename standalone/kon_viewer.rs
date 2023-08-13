use egui::TextureHandle;
use kon_players::{
    clients::{IClient, SampleClient},
    InstrumentType, Member, MemberList,
};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
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
    texture_handle: Option<TextureHandle>,
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
            texture_handle: None,
            _marker: Default::default(),
        }
    }
}

impl<TClient: kon_players::clients::IClient + Default> eframe::App for App<TClient> {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
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

            if self.texture_handle.is_none() {
                let texture_handle = Self::load_texture(
                    ctx,
                    "AG".to_owned(),
                    include_bytes!("../resources/images/acoustic_guitar.png"),
                );
                self.texture_handle = Some(texture_handle);
            }

            // フィルターが指定されてなければ全部表示
            if self.member_list.filter().is_empty() {
                for member in self.member_list.members() {
                    self.draw_member_item(ui, member);
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

impl<TClient: kon_players::clients::IClient + Default> App<TClient> {
    pub fn draw_member_item<'a>(&self, ui: &mut eframe::egui::Ui, member: &Member) {
        ui.horizontal(|ui| {
            ui.label(member.name());

            if let Some(icon) = &self.texture_handle {
                ui.image(icon.id(), icon.size_vec2());
                ui.image(icon.id(), icon.size_vec2());
            }
        });
    }

    fn load_texture(ctx: &eframe::egui::Context, id: String, byte_data: &[u8]) -> TextureHandle {
        let image = image::load_from_memory(byte_data).unwrap();
        let image_buffer = image.to_rgba8();
        let image_data = eframe::egui::ImageData::Color(
            eframe::egui::ColorImage::from_rgba_unmultiplied([32, 32], &image_buffer),
        );

        let texture_handle = ctx.load_texture(id, image_data, Default::default());
        texture_handle
    }
}
