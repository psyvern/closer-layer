use std::collections::HashMap;
use std::process::Command;

use gtk::gdk;
use gtk::prelude::GestureExt;
use hyprland::event_listener::EventListener;
use hyprland::shared::HyprData;
use relm4::AsyncComponentSender;
use relm4::prelude::{AsyncComponent, AsyncComponentParts};

use gtk::{Window, prelude::WidgetExt};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use relm4::RelmApp;

#[derive(Debug)]
enum AppMsg {
    Hide,
    OpenLayer(String),
    CloseLayer(String),
}

struct AppModel {
    visible: bool,
    layers: HashMap<&'static str, &'static str>,
}

#[relm4::component(async)]
impl AsyncComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    type Init = HashMap<&'static str, &'static str>;

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

            gesture.connect_pressed(move |gesture, _, _, _| {
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

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.update(message, sender.clone(), root).await;
        self.update_view(widgets, sender);
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
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

fn main() {
    let layers = maplit::hashmap! {
        "audio" => "ags -t audio",
        "jogger" => "jogger --toggle",
        "players" => "ags -t players",
        "sidebar" => "ags -t sidebar",
    };
    let app = RelmApp::new("com.psyvern.closer");

    // The CSS "magic" happens here.
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("../style.css"));
    // We give the CssProvided to the default screen so the CSS rules we added
    // can be applied to our window.
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );

    app.run_async::<AppModel>(layers);
    // app.run::<AppModel>(layers);
}
