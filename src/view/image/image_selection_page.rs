use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::subclass::Signal;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::model;
use crate::utils;
use crate::view;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/image-selection-page.ui")]
    pub(crate) struct ImageSelectionPage {
        pub(super) client: WeakRef<model::Client>,
        #[template_child]
        pub(super) header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(super) select_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) back_navigation_controls: TemplateChild<view::BackNavigationControls>,
        #[template_child]
        pub(super) image_search_widget: TemplateChild<view::ImageSearchWidget>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageSelectionPage {
        const NAME: &'static str = "ImageSelectionPage";
        type Type = super::ImageSelectionPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("image.select", None, |widget, _, _| {
                widget.select_image();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ImageSelectionPage {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "image-selected",
                    &[glib::GString::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "client",
                    "Client",
                    "The client of this ImagePullingPage",
                    model::Client::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "client" => self.client.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "client" => obj.client().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.action_set_enabled("image.select", false);
            self.image_search_widget.connect_notify_local(
                Some("selected-image"),
                clone!(@weak obj => move |widget, _| {
                    obj.action_set_enabled("image.select", widget.selected_image().is_some());
                }),
            );
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.header_bar.unparent();
            self.image_search_widget.unparent();
        }
    }

    impl WidgetImpl for ImageSelectionPage {
        fn root(&self, widget: &Self::Type) {
            self.parent_root(widget);
            utils::root(widget).set_default_widget(Some(&*self.select_button));
        }

        fn unroot(&self, widget: &Self::Type) {
            utils::root(widget).set_default_widget(gtk::Widget::NONE);
            self.parent_unroot(widget)
        }
    }
}

glib::wrapper! {
    pub(crate) struct ImageSelectionPage(ObjectSubclass<imp::ImageSelectionPage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl From<Option<&model::Client>> for ImageSelectionPage {
    fn from(client: Option<&model::Client>) -> Self {
        glib::Object::new(&[("client", &client)]).expect("Failed to create ImageSelectionPage")
    }
}

impl ImageSelectionPage {
    fn client(&self) -> Option<model::Client> {
        self.imp().client.upgrade()
    }

    fn select_image(&self) {
        let imp = self.imp();

        if let Some(search_response) = imp.image_search_widget.selected_image() {
            let image = format!(
                "{}:{}",
                search_response.name().unwrap(),
                imp.image_search_widget.tag(),
            );

            self.emit_by_name::<()>("image-selected", &[&image]);

            imp.back_navigation_controls.navigate_back();
        }
    }

    pub(crate) fn connect_image_selected<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("image-selected", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let image = values[1].get::<String>().unwrap();
            f(&obj, image);

            None
        })
    }
}
