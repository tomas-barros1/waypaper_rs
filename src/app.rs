use crate::services::{
    cache::Cache, thumbnail_cache::ThumbnailCache, wallpaper_service::WallpaperService,
};
use adw::prelude::*;
use gtk::{
    Align, Box, Button, Entry, FileChooserAction, FileChooserNative, FlowBox, Label, Orientation,
    Picture, ScrolledWindow, SelectionMode, gdk_pixbuf::Pixbuf, glib,
};
use std::{
    cell::{Cell, RefCell},
    io::Cursor,
    path::PathBuf,
    rc::Rc,
    sync::mpsc,
    thread,
    time::Duration,
};

pub fn build_ui(application: &adw::Application) {
    let cache = Cache::load_default();
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("Waypaper RS")
        .default_width(900)
        .default_height(620)
        .build();
    let root = Box::new(Orientation::Vertical, 0);
    let header = adw::HeaderBar::new();
    let choose = Button::with_label(&rust_i18n::t!("choose_folder"));
    header.pack_start(&choose);
    let search = Entry::new();
    search.set_placeholder_text(Some(&rust_i18n::t!("search_placeholder")));
    search.set_width_chars(32);
    search.set_hexpand(false);
    header.set_title_widget(Some(&search));
    root.append(&header);
    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(18);
    content.set_margin_bottom(18);
    content.set_margin_start(18);
    content.set_margin_end(18);
    let status = Label::new(Some(&rust_i18n::t!("select_wallpaper")));
    status.set_halign(Align::Start);
    content.append(&status);
    let flow = FlowBox::new();
    flow.set_selection_mode(SelectionMode::None);
    flow.set_min_children_per_line(4);
    flow.set_max_children_per_line(5);
    flow.set_column_spacing(12);
    flow.set_row_spacing(12);
    content.append(&ScrolledWindow::builder().vexpand(true).child(&flow).build());
    root.append(&content);
    window.set_content(Some(&root));
    let flow_for_folder = flow.clone();
    let status_for_folder = status.clone();
    let window_for_folder = window.clone();
    let folder_state = Rc::new(RefCell::new(cache.folder.clone()));
    let load_generation = Rc::new(Cell::new(0_u64));
    let folder_state_for_choose = folder_state.clone();
    let generation_for_choose = load_generation.clone();
    choose.connect_clicked(move |_| {
        let folder_state_for_response = folder_state_for_choose.clone();
        let dialog = FileChooserNative::new(
            Some(&rust_i18n::t!("choose_folder")),
            Some(&window_for_folder),
            FileChooserAction::SelectFolder,
            Some("Choose"),
            Some("Cancel"),
        );
        let flow = flow_for_folder.clone();
        let status = status_for_folder.clone();
        let generation = generation_for_choose.clone();
        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = dialog.file().and_then(|f| f.path()) {
                    let _ = WallpaperService::new(Cache::load_default()).set_folder(file.clone());
                    *folder_state_for_response.borrow_mut() = Some(file.clone());
                    populate_async(&flow, &status, file, "", &generation);
                }
            }
            dialog.destroy();
        });
        dialog.show();
    });
    let flow_for_search = flow.clone();
    let status_for_search = status.clone();
    let folder_state_for_search = folder_state.clone();
    let generation_for_search = load_generation.clone();
    search.connect_changed(move |entry| {
        if let Some(folder) = folder_state_for_search.borrow().clone() {
            populate_async(
                &flow_for_search,
                &status_for_search,
                folder,
                &entry.text(),
                &generation_for_search,
            );
        }
    });
    window.present();
    if let Some(folder) = cache.folder {
        populate_async(&flow, &status, folder, "", &load_generation);
    }
}

fn populate_async(
    flow: &FlowBox,
    status: &Label,
    folder: PathBuf,
    query: &str,
    generation: &Rc<Cell<u64>>,
) {
    generation.set(generation.get().wrapping_add(1));
    let current_generation = generation.get();
    while let Some(child) = flow.first_child() {
        flow.remove(&child);
    }
    let service = WallpaperService::new(Cache::load_default());
    let query = query.to_lowercase();
    let wallpapers: Vec<_> = service
        .wallpapers_in(&folder)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_lowercase().contains(&query))
                .unwrap_or(false)
        })
        .collect();
    status.set_text(&rust_i18n::t!(
        "wallpapers_found",
        count = wallpapers.len(),
        folder = folder.display()
    ));
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for path in wallpapers {
            let thumbnail = ThumbnailCache::png(&path, 176, 110);
            if sender.send((path, thumbnail)).is_err() {
                break;
            }
        }
    });
    let flow = flow.clone();
    let status = status.clone();
    let generation = generation.clone();
    glib::timeout_add_local(Duration::from_millis(16), move || {
        if generation.get() != current_generation {
            return glib::ControlFlow::Break;
        }
        // Decoding happens off the GTK thread. Only lightweight widget
        // insertions happen per frame, keeping input and rendering responsive.
        for _ in 0..8 {
            match receiver.try_recv() {
                Ok((path, bytes)) => {
                    let pixbuf = bytes.and_then(|bytes| Pixbuf::from_read(Cursor::new(bytes)).ok());
                    add_thumbnail(&flow, &status, path, pixbuf);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return glib::ControlFlow::Break,
            }
        }
        glib::ControlFlow::Continue
    });
}

fn add_thumbnail(flow: &FlowBox, status: &Label, path: PathBuf, pixbuf: Option<Pixbuf>) {
    let button = Button::new();
    button.set_tooltip_text(Some(&path.display().to_string()));
    // Decode thumbnails, never the original wallpaper dimensions. A folder
    // with many 8K images must stay cheap in memory.
    let picture = pixbuf
        .map(|pixbuf| Picture::for_paintable(&gtk::gdk::Texture::for_pixbuf(&pixbuf)))
        .unwrap_or_else(Picture::new);
    button.set_size_request(180, 130);
    picture.set_size_request(176, 110);
    button.set_child(Some(&picture));
    let status = status.clone();
    button.connect_clicked(move |_| {
        let result = WallpaperService::new(Cache::load_default()).set_wallpaper(path.clone());
        let message = if result.is_ok() {
            rust_i18n::t!("wallpaper_applied").to_string()
        } else {
            rust_i18n::t!("could_not_apply").to_string()
        };
        status.set_text(&message);
    });
    flow.insert(&button, -1);
}
