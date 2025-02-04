use std::cell::RefCell;

use adw::subclass::prelude::*;
use adw::traits::ComboRowExt;
use ashpd::desktop::account::UserInformationRequest;
use ashpd::WindowIdentifier;
use gettextrs::gettext;
use gtk::gio;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::model;
use crate::podman;
use crate::utils;
use crate::view;

const ACTION_FETCH_USERNAME: &str = "container-commit-page.fetch-username";
const ACTION_ADD_CHANGE: &str = "container-commit-page.add-change";
const ACTION_COMMIT: &str = "container-commit-page.commit";

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/container/commit-page.ui")]
    pub(crate) struct CommitPage {
        pub(super) container: WeakRef<model::Container>,
        pub(super) changes: RefCell<gio::ListStore>,
        #[template_child]
        pub(super) author_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) comment_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) repo_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) tag_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) format_combo_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub(super) format_list: TemplateChild<gtk::StringList>,
        #[template_child]
        pub(super) pause_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub(super) changes_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) commit_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) leaflet_overlay: TemplateChild<view::LeafletOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CommitPage {
        const NAME: &'static str = "PdsContainerCommitPage";
        type Type = super::CommitPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(ACTION_FETCH_USERNAME, None, |widget, _, _| {
                widget.fetch_user_information();
            });
            klass.install_action(ACTION_ADD_CHANGE, None, |widget, _, _| {
                widget.add_change();
            });
            klass.install_action(ACTION_COMMIT, None, |widget, _, _| {
                widget.commit();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CommitPage {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<model::Container>("container")
                        .flags(
                            glib::ParamFlags::READWRITE
                                | glib::ParamFlags::CONSTRUCT
                                | glib::ParamFlags::EXPLICIT_NOTIFY,
                        )
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "container" => self.instance().set_container(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "container" => self.instance().container().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.changes_list_box
                .bind_model(Some(&*self.changes.borrow()), |item| {
                    view::ValueRow::new(item.downcast_ref().unwrap(), gettext("Change")).upcast()
                });
            self.changes_list_box.append(
                &gtk::ListBoxRow::builder()
                    .action_name(ACTION_ADD_CHANGE)
                    .selectable(false)
                    .child(
                        &gtk::Image::builder()
                            .icon_name("list-add-symbolic")
                            .margin_top(12)
                            .margin_bottom(12)
                            .build(),
                    )
                    .build(),
            );
        }

        fn dispose(&self) {
            utils::ChildIter::from(&*self.instance()).for_each(|child| child.unparent());
        }
    }

    impl WidgetImpl for CommitPage {
        fn root(&self) {
            self.parent_root();

            let widget = &*self.instance();

            glib::idle_add_local(
                clone!(@weak widget => @default-return glib::Continue(false), move || {
                    widget.imp().author_entry_row.grab_focus();
                    glib::Continue(false)
                }),
            );
            utils::root(widget).set_default_widget(Some(&*self.commit_button));
        }

        fn unroot(&self) {
            utils::root(&*self.instance()).set_default_widget(gtk::Widget::NONE);
            self.parent_unroot()
        }
    }
}

glib::wrapper! {
    pub(crate) struct CommitPage(ObjectSubclass<imp::CommitPage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl From<&model::Container> for CommitPage {
    fn from(container: &model::Container) -> Self {
        glib::Object::new::<Self>(&[("container", &container)])
    }
}

impl CommitPage {
    fn container(&self) -> Option<model::Container> {
        self.imp().container.upgrade()
    }

    fn set_container(&self, value: Option<&model::Container>) {
        if self.container().as_ref() == value {
            return;
        }
        self.imp().container.set(value);
        self.notify("container");
    }

    fn fetch_user_information(&self) {
        glib::MainContext::default().block_on(async move {
            match UserInformationRequest::default()
                .identifier(WindowIdentifier::from_native(&self.native().unwrap()).await)
                .build()
                .await
            {
                Ok(user_info) => self.imp().author_entry_row.set_text(user_info.name()),
                Err(e) => {
                    if let ashpd::Error::Portal(ashpd::PortalError::Cancelled(_)) = e {
                        utils::show_error_toast(
                            self,
                            &gettext("Error on fetching user name"),
                            &e.to_string(),
                        );
                    }
                }
            }
        });
    }

    fn add_change(&self) {
        let change = model::Value::default();

        change.connect_remove_request(clone!(@weak self as obj => move |change| {
            let imp = obj.imp();

            let port_mappings = imp.changes.borrow();
            if let Some(pos) = port_mappings.find(change) {
                port_mappings.remove(pos);
            }
        }));

        self.imp().changes.borrow().append(&change);
    }

    fn commit(&self) {
        if let Some(container) = self.container() {
            if let Some(api) = container.api() {
                if let Some(client) = container
                    .container_list()
                    .and_then(|container_list| container_list.client())
                {
                    let imp = self.imp();

                    let opts = podman::opts::ContainerCommitOpts::builder();

                    let opts = set_opts_builder_field(
                        opts,
                        imp.author_entry_row.text().trim(),
                        |opts, field| opts.author(field),
                    );
                    let opts = set_opts_builder_field(
                        opts,
                        imp.comment_entry_row.text().trim(),
                        |opts, field| opts.comment(field),
                    );

                    let repo = imp.repo_entry_row.text();
                    let repo = repo.trim();
                    let opts = set_opts_builder_field(opts, repo, |opts, field| opts.repo(field));

                    let tag = imp.tag_entry_row.text();
                    let tag = tag.trim();
                    let opts = set_opts_builder_field(opts, tag, |opts, field| opts.tag(field));

                    let opts = opts
                        .format(
                            imp.format_list
                                .get()
                                .string(imp.format_combo_row.selected())
                                .unwrap(),
                        )
                        .pause(imp.pause_switch.is_active());

                    let page = view::ActionPage::from(
                        &client.action_list().commit_container(
                            if repo.is_empty() {
                                None
                            } else {
                                Some(format!(
                                    "{}:{}",
                                    repo,
                                    if tag.is_empty() { "latest" } else { tag }
                                ))
                            }
                            .as_deref(),
                            &container.name(),
                            api,
                            opts.build(),
                        ),
                    );

                    imp.leaflet_overlay.show_details(&page);
                }
            }
        }
    }
}

fn set_opts_builder_field<F>(
    opts: podman::opts::ContainerCommitOptsBuilder,
    field: &str,
    op: F,
) -> podman::opts::ContainerCommitOptsBuilder
where
    F: FnOnce(
        podman::opts::ContainerCommitOptsBuilder,
        &str,
    ) -> podman::opts::ContainerCommitOptsBuilder,
{
    if field.is_empty() {
        opts
    } else {
        op(opts, field)
    }
}
