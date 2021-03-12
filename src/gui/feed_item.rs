use crate::gui::thumbnail::{Thumbnail, ThumbnailMsg};
use crate::youtube_feed::Entry;

use std::thread;

use gtk::prelude::*;
use gtk::{Align, ImageExt, Justification, Orientation};
use pango::{AttrList, Attribute, EllipsizeMode, WrapMode};
use relm::{Relm, Widget};
use relm_derive::{widget, Msg};

#[derive(Msg)]
pub enum FeedListItemMsg {
    SetImage,
    Clicked,
    SetPlaying(bool),
}

pub struct FeedListItemModel {
    entry: Entry,
    playing: bool,
    relm: Relm<FeedListItem>,
}

#[widget]
impl Widget for FeedListItem {
    fn model(relm: &Relm<Self>, entry: Entry) -> FeedListItemModel {
        FeedListItemModel {
            entry,
            playing: false,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: FeedListItemMsg) {
        match event {
            FeedListItemMsg::SetImage => {
                self.components.thumbnail.emit(ThumbnailMsg::SetImage);
            }
            FeedListItemMsg::SetPlaying(playing) => {
                self.model.playing = playing;
            }
            FeedListItemMsg::Clicked => {
                let result = self.model.entry.play();

                if let Ok(mut child) = result {
                    let stream = self.model.relm.stream().clone();

                    stream.emit(FeedListItemMsg::SetPlaying(true));

                    let (_channel, sender) = relm::Channel::new(move |_| {
                        // TODO:
                        // Catch panic that happens when stream is dropped, i.e. the widget has been destroyed, i.e. the feed has been reloaded
                        stream.emit(FeedListItemMsg::SetPlaying(false));
                    });

                    thread::spawn(move || {
                        let _ = child.wait();
                        sender.send(()).expect("Could not send message");
                    });
                }
            }
        }
    }

    fn init_view(&mut self) {
        let title_attr_list = AttrList::new();
        title_attr_list.insert(Attribute::new_size(12 * pango::SCALE).unwrap());
        self.widgets
            .label_title
            .set_attributes(Some(&title_attr_list));

        let author_attr_list = AttrList::new();
        author_attr_list.insert(Attribute::new_size(8 * pango::SCALE).unwrap());
        self.widgets
            .label_author
            .set_attributes(Some(&author_attr_list));

        let date_attr_list = author_attr_list.clone();
        self.widgets
            .label_date
            .set_attributes(Some(&date_attr_list));

        self.widgets
            .playing
            .set_from_icon_name(Some("media-playback-start"), gtk::IconSize::LargeToolbar);
    }

    view! {
        gtk::ListBoxRow {
            gtk::Box {
                orientation: Orientation::Horizontal,

                #[name="playing"]
                gtk::Image {
                    visible: self.model.playing
                },

                #[name="thumbnail"]
                Thumbnail(self.model.entry.media.thumbnail.clone()),

                gtk::Box {
                    orientation: Orientation::Vertical,

                    #[name="label_title"]
                    gtk::Label {
                        text: &self.model.entry.title,
                        ellipsize: EllipsizeMode::End,
                        property_wrap: true,
                        property_wrap_mode: WrapMode::Word,
                        lines: 2,
                        justify: Justification::Left,
                    },
                    #[name="label_author"]
                    gtk::Label {
                        text: &self.model.entry.author.name,
                        ellipsize: EllipsizeMode::End,
                        property_wrap: true,
                        property_wrap_mode: WrapMode::Word,
                        halign: Align::Start
                    },
                    #[name="label_date"]
                    gtk::Label {
                        text: &self.model.entry.published.to_string(),
                        ellipsize: EllipsizeMode::End,
                        property_wrap: true,
                        property_wrap_mode: WrapMode::Word,
                        halign: Align::Start
                    },
                }
            }
        }
    }
}
