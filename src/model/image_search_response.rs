use gtk::glib;
use gtk::prelude::ParamSpecBuilderExt;
use gtk::prelude::ToValue;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use crate::podman;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct ImageSearchResponse {
        pub(super) automated: OnceCell<Option<String>>,
        pub(super) description: OnceCell<Option<String>>,
        pub(super) index: OnceCell<Option<String>>,
        pub(super) name: OnceCell<Option<String>>,
        pub(super) official: OnceCell<Option<String>>,
        pub(super) stars: OnceCell<i64>,
        pub(super) tag: OnceCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageSearchResponse {
        const NAME: &'static str = "ImageSearchResponse";
        type Type = super::ImageSearchResponse;
    }

    impl ObjectImpl for ImageSearchResponse {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("automated")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                    glib::ParamSpecString::builder("description")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                    glib::ParamSpecString::builder("index")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                    glib::ParamSpecString::builder("name")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                    glib::ParamSpecString::builder("official")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                    glib::ParamSpecInt64::builder("stars")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                    glib::ParamSpecString::builder("tag")
                        .flags(glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "automated" => self.automated.set(value.get().unwrap()).unwrap(),
                "description" => self.description.set(value.get().unwrap()).unwrap(),
                "index" => self.index.set(value.get().unwrap()).unwrap(),
                "name" => self.name.set(value.get().unwrap()).unwrap(),
                "official" => self.official.set(value.get().unwrap()).unwrap(),
                "stars" => self.stars.set(value.get().unwrap()).unwrap(),
                "tag" => self.tag.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = &*self.instance();
            match pspec.name() {
                "automated" => obj.automated().to_value(),
                "description" => obj.description().to_value(),
                "index" => obj.index().to_value(),
                "name" => obj.name().to_value(),
                "official" => obj.official().to_value(),
                "stars" => obj.stars().to_value(),
                "tag" => obj.tag().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ImageSearchResponse(ObjectSubclass<imp::ImageSearchResponse>);
}

impl From<podman::models::RegistrySearchResponse> for ImageSearchResponse {
    fn from(response: podman::models::RegistrySearchResponse) -> Self {
        glib::Object::new::<Self>(&[
            ("automated", &response.automated),
            ("description", &response.description),
            ("index", &response.index),
            ("name", &response.name),
            ("official", &response.official),
            ("stars", &response.stars.unwrap_or(-1)),
            ("tag", &response.tag),
        ])
    }
}

impl ImageSearchResponse {
    pub(crate) fn automated(&self) -> Option<&str> {
        self.imp().automated.get().and_then(Option::as_deref)
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.imp().description.get().and_then(Option::as_deref)
    }

    pub(crate) fn index(&self) -> Option<&str> {
        self.imp().index.get().and_then(Option::as_deref)
    }

    pub(crate) fn name(&self) -> Option<&str> {
        self.imp().name.get().and_then(Option::as_deref)
    }

    pub(crate) fn official(&self) -> Option<&str> {
        self.imp().official.get().and_then(Option::as_deref)
    }

    pub(crate) fn stars(&self) -> i64 {
        *self.imp().stars.get().unwrap()
    }

    pub(crate) fn tag(&self) -> Option<&str> {
        self.imp().tag.get().and_then(Option::as_deref)
    }
}
