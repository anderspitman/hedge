use eframe::{egui,CreationContext};
use std::sync::{Arc,Mutex};
//use std::collections::HashMap;
use hedge::{InMessage,OutMessage,Widget};
//use rfd::FileDialog;

//#[cfg(target_arch = "wasm32")]
//use wasm_bindgen::prelude::*;

mod app;

//slint::include_modules!();

//const WIDTH: usize = 512;
//const HEIGHT: usize = 512;
//const BYTES_PER_PIXEL: usize = 4;


pub struct HedgeApp {
    app: Arc<Mutex<app::App>>,
    tree: Widget,
    messages: Vec<InMessage>,
    //edit_texts: HashMap<String, String>,
}

impl HedgeApp {
    pub fn new(_cc: &CreationContext, wasm_path: std::path::PathBuf) -> Self {

        let app = Arc::new(Mutex::new(app::App::new_from_path(wasm_path)));

        //app.lock().unwrap().update(vec![]);

        Self {
            app: app,
            tree: Widget::Container{
                children: vec![],
            },
            messages: vec![],
            //edit_texts: HashMap::new(),
        }
    }

    fn render_tree(&mut self, path: &str, tree: &Widget, ui: &mut egui::Ui, messages: &mut Vec<InMessage>) {

        match tree {
            Widget::Container { children } => {
                self.render_children(path, ui, children, messages);
            },
            Widget::Row{ children } => {
                ui.horizontal(|ui| {
                    self.render_children(path, ui, children, messages);
                });
            },
            Widget::Column{ children } => {
                ui.vertical(|ui| {
                    self.render_children(path, ui, children, messages);
                });
            },
            Widget::Textbox{ text } => {
                //let text = self.edit_texts.entry(path.to_string()).or_insert(init_text.to_string());
                //if ui.text_edit_singleline(text).changed() {
                //    messages.push(InMessage::TextChanged{
                //        path: path.to_string(),
                //        text: text.to_string(),
                //    });
                //}
                ui.label(text);
            },
            Widget::Button { text, name } => {
                if ui.button(text).clicked() {
                    messages.push(InMessage::WidgetPressed{
                        path: path.to_string(),
                        name: name.clone(),
                    });
                }
            },
            Widget::Label { text } => {
                ui.label(text);
            },
        }
    }

    fn render_children(&mut self, path: &str, ui: &mut egui::Ui, children: &Vec<Widget>, messages: &mut Vec<InMessage>) {
        for (i, child) in children.iter().enumerate() {
            self.render_child(path, ui, &child, i, messages);
        }
    }

    fn render_child(&mut self, path: &str, ui: &mut egui::Ui, child: &Widget, i: usize, messages: &mut Vec<InMessage>) {
        let child_path = if path == "/" {
            format!("/{}", i)
        }
        else {
            format!("{}/{}", path, i)
        };

        self.render_tree(&child_path, child, ui, messages);
    }
}

impl eframe::App for HedgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let out_messages = {
            let mut app = self.app.lock().unwrap();
            app.update(&self.messages)
        };

        self.messages.clear();

        if cfg!(target_os = "android") {
            // Reserve some space at the top so the demo ui isn't hidden behind the android status bar
            // TODO(lucasmerlin): This is a pretty big hack, should be fixed once safe_area implemented
            // for android:
            // https://github.com/rust-windowing/winit/issues/3910
            egui::TopBottomPanel::top("status_bar_space").show(ctx, |ui| {
                ui.set_height(32.0);
            });
        }

        let mut messages = vec![];
        // TODO: see if we can get rid of this clone
        let tree = self.tree.clone();

        egui::CentralPanel::default().show(&ctx, |ui| {
            //self.render_tree("/", &self.tree, ui, &mut self.messages);
            self.render_tree("/", &tree, ui, &mut messages);
        });

        self.messages = messages;

        let mut tree_update = None; 

        // TODO: having message processing down here is hacky. I think it delays things by a frame.
        // I did it so that input messages can be added inline. Has to be below
        // self.messages.clear() above.
        for message in out_messages {
            match message {
                OutMessage::SetTree { tree, .. } => {
                    tree_update = Some(tree);
                },
                OutMessage::OpenFolder => {
                    println!("open folder");
                    //if let Some(path) = FileDialog::new().pick_folder() {
                    //    //self.selected_folder = Some(path.display().to_string());
                    //    println!("{:?}", path);
                    //    
                    //    //self.messages.push(InMessage::FolderOpened{
                    //    //    path: path.to_string_lossy().replace('\\', "/"),
                    //    //});
                    //}
                },
            }
        }

        if let Some(new_tree) = tree_update {
            self.tree = new_tree;
        }

    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {

    let mut wasm_path = app.external_data_path().unwrap();
    wasm_path.push("main.wasm");

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    eframe::run_native(
        "Hedge Browser",
        options,
        Box::new(|cc| Ok(Box::new(HedgeApp::new(cc, wasm_path)))),
    )
    .unwrap()
}
