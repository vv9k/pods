use gtk::gio;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::Lazy;
use utils::ToTypedListModel;

use crate::model;
use crate::model::SelectableExt;
use crate::utils;

mod imp {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    pub(crate) struct SelectableList(glib::gobject_ffi::GTypeInterface);

    #[glib::object_interface]
    unsafe impl ObjectInterface for SelectableList {
        const NAME: &'static str = "SelectableList";
        type Prerequisites = (gio::ListModel,);

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("selection-mode").build(),
                    glib::ParamSpecUInt::builder("num-selected")
                        .flags(glib::ParamFlags::READABLE)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }
    }
}

glib::wrapper! {
    pub(crate) struct SelectableList(ObjectInterface<imp::SelectableList>) @requires gio::ListModel;
}

impl SelectableList {
    pub(super) fn bootstrap<L: IsA<Self> + IsA<gio::ListModel>>(list: &L)
    where
        L: glib::clone::Downgrade,
        <L as glib::clone::Downgrade>::Weak: glib::clone::Upgrade,
        <<L as glib::clone::Downgrade>::Weak as glib::clone::Upgrade>::Strong: glib::IsA<Self>,
    {
        list.connect_items_changed(|self_, position, _, added| {
            (position..position + added)
                .map(|i| self_.item(i).unwrap())
                .for_each(|item| {
                    item.connect_notify_local(
                        Some("selected"),
                        clone!(@weak self_ as obj => move |_, _| obj.notify("num-selected")),
                    );
                });
        });
    }
}

pub(crate) trait SelectableListExt: glib::IsA<SelectableList> {
    fn is_selection_mode(&self) -> bool;

    fn set_selection_mode(&self, value: bool);

    fn select_all(&self) {
        self.select(true);
    }

    fn select_none(&self) {
        self.select(false);
    }

    fn select(&self, value: bool);

    fn num_selected(&self) -> u32;

    fn selected_items(&self) -> Vec<model::Selectable>;
}

impl<T: IsA<SelectableList> + IsA<gio::ListModel>> SelectableListExt for T {
    fn is_selection_mode(&self) -> bool {
        self.property("selection-mode")
    }

    fn set_selection_mode(&self, value: bool) {
        if !value {
            self.select_none();
        }
        self.set_property("selection-mode", value);
    }

    fn select(&self, value: bool) {
        self.to_owned()
            .to_typed_list_model::<model::Selectable>()
            .into_iter()
            .for_each(|obj| obj.set_selected(value));
    }

    fn num_selected(&self) -> u32 {
        self.to_owned()
            .to_typed_list_model::<model::Selectable>()
            .into_iter()
            .filter(|obj| obj.is_selected())
            .count() as u32
    }

    fn selected_items(&self) -> Vec<model::Selectable> {
        self.to_owned()
            .to_typed_list_model::<model::Selectable>()
            .into_iter()
            .filter(|obj| obj.is_selected())
            .collect()
    }
}

unsafe impl<T: ObjectSubclass> IsImplementable<T> for SelectableList {}
