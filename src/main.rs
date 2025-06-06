use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use gtk::gdk;
use gtk::prelude::GestureExt;
use hyprland::event_listener::EventListener;
use hyprland::shared::HyprData;
use relm4::AsyncComponentSender;
use relm4::prelude::{AsyncComponent, AsyncComponentParts};

use gtk::{Window, prelude::WidgetExt};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use relm4::RelmApp;
use xdg::BaseDirectories;

#[derive(Debug)]
enum AppMsg {
    Hide,
    OpenLayer(String),
    CloseLayer(String),
}

struct AppModel {
    visible: bool,
    layers: HashMap<String, String>,
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    type Init = HashMap<String, String>;

    view! {
        Window {
            #[watch]
            set_visible: model.visible,

            init_layer_shell: (),
            set_namespace: "closer0",
            set_layer: Layer::Overlay,
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),
            set_margin: (Edge::Top, -100),

            add_controller: gesture,
        }
    }

    async fn init(
        layers: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AppModel {
            visible: false,
            layers,
        };

        let gesture = gtk::GestureClick::new();
        {
            let sender = sender.clone();

            gesture.connect_released(move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                sender.input(AppMsg::Hide);
            });
        }

        let widgets = view_output!();

        let _sender = sender.clone();

        tokio::spawn(async move {
            let mut event_listener = EventListener::new();

            {
                let _sender = _sender.clone();
                event_listener.add_layer_opened_handler(move |data| {
                    _sender.input(AppMsg::OpenLayer(data));
                });
            }

            event_listener.add_layer_closed_handler(move |data| {
                _sender.input(AppMsg::CloseLayer(data));
            });

            event_listener.start_listener().unwrap();
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            AppMsg::Hide => {
                for (_, x) in hyprland::data::Layers::get().unwrap() {
                    for (_, x) in x {
                        for x in x {
                            if let Some(command) = self.layers.get(x.namespace.as_str()) {
                                let _ = Command::new("sh").arg("-c").arg(command).output();
                            }
                        }
                    }
                }

                self.visible = false;
            }
            AppMsg::OpenLayer(layer) => {
                if self.layers.contains_key(layer.as_str()) {
                    self.visible = true;
                }
            }
            AppMsg::CloseLayer(layer) => {
                for (_, x) in hyprland::data::Layers::get().unwrap() {
                    for (_, x) in x {
                        for x in x {
                            if self.layers.contains_key(x.namespace.as_str())
                                && x.namespace != layer
                            {
                                return;
                            }
                        }
                    }
                }
                self.visible = false;
            }
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[arg(short = 'C', long, default_value = "#1F1F1F3F")]
    color: String,
}

fn main() {
    let args = Args::parse();
    let layers = match args.config {
        None => {
            let base_dirs = BaseDirectories::new();
            let path = base_dirs.place_config_file("closer-layer.toml").unwrap();
            if std::fs::exists(&path).unwrap() {
                let layers = std::fs::read_to_string(path).unwrap();
                toml::from_str::<HashMap<String, String>>(&layers).unwrap()
            } else {
                std::fs::File::create(path).unwrap();
                HashMap::new()
            }
        }
        Some(path) => {
            if std::fs::exists(&path).unwrap() {
                let layers = std::fs::read_to_string(path).unwrap();
                toml::from_str::<HashMap<String, String>>(&layers).unwrap()
            } else {
                HashMap::new()
            }
        }
    };

    let app = RelmApp::new("com.psyvern.closer");

    // The CSS "magic" happens here.
    let provider = gtk::CssProvider::new();
    let css = format!("window {{ background-color: {}; }}", args.color);
    provider.load_from_string(&css);
    // We give the CssProvided to the default screen so the CSS rules we added
    // can be applied to our window.
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );

    app.with_args(Vec::new()).run_async::<AppModel>(layers);
}
