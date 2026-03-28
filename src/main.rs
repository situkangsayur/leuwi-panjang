use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Label, Notebook, Orientation, CssProvider};
use gdk4 as gdk;
use vte4::prelude::*;
use vte4::Terminal;

const APP_ID: &str = "com.situkangsayur.leuwi-panjang";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        r#"
        window {
            background-color: rgba(10, 20, 16, 0.85);
            color: #B8D4CC;
            border-radius: 12px;
        }
        window > box {
            border-radius: 12px;
        }
        .header-custom {
            background-color: #060F0B;
            min-height: 34px;
            border-bottom: 1px solid #1A3A28;
            border-radius: 12px 12px 0 0;
        }
        .header-custom button {
            color: #5C8A72;
            background: transparent;
            border: none;
            min-height: 28px;
            padding: 2px 10px;
            border-radius: 6px;
        }
        .header-custom button:hover {
            background-color: #0D1F17;
            color: #B8D4CC;
        }
        .close-btn:hover {
            background-color: rgba(233, 69, 96, 0.5);
            border-radius: 6px;
        }
        notebook header {
            background-color: #060F0B;
            border: none;
        }
        notebook header tab {
            background-color: #060F0B;
            color: #5C8A72;
            border: none;
            padding: 4px 14px;
            border-radius: 8px 8px 0 0;
            margin: 2px 1px 0 1px;
        }
        notebook header tab:checked {
            background-color: #0A1410;
            color: #B8D4CC;
            border-bottom: 2px solid #00FF88;
        }
        notebook header tab:hover {
            background-color: #0D1F17;
        }
        notebook header tab button {
            min-height: 16px;
            min-width: 16px;
            padding: 0;
            border-radius: 4px;
        }
        .status-bar {
            background-color: #060F0B;
            color: #5C8A72;
            padding: 2px 12px;
            font-size: 10px;
            border-top: 1px solid #1A3A28;
            border-radius: 0 0 12px 12px;
        }
        .status-green { color: #00FF88; }
        "#,
    );
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Leuwi Panjang")
        .default_width(1200)
        .default_height(800)
        .decorated(false)    // NO TITLE BAR
        .build();

    // Set empty titlebar to kill GTK4 CSD header bar completely
    let empty_titlebar = GtkBox::new(Orientation::Horizontal, 0);
    empty_titlebar.set_size_request(-1, 0);
    window.set_titlebar(Some(&empty_titlebar));

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Our custom header (replaces CSD)
    let header = build_header(&window);
    main_box.append(&header);

    // Tabs
    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_show_border(false);
    notebook.set_vexpand(true);

    add_terminal_tab(&notebook, "Terminal 1");

    // + button
    let nb_clone = notebook.clone();
    let add_btn = Button::with_label("+");
    add_btn.connect_clicked(move |_| {
        let n = nb_clone.n_pages() + 1;
        add_terminal_tab(&nb_clone, &format!("Terminal {n}"));
    });
    let end_box = GtkBox::new(Orientation::Horizontal, 0);
    end_box.append(&add_btn);
    notebook.set_action_widget(&end_box, gtk::PackType::End);

    main_box.append(&notebook);
    main_box.append(&build_status_bar());

    window.set_child(Some(&main_box));

    // Keyboard shortcuts
    let nb_keys = notebook.clone();
    let win_keys = window.clone();
    let key_ctrl = gtk::EventControllerKey::new();
    key_ctrl.connect_key_pressed(move |_, key, _code, mods| {
        let cs = mods.contains(gdk::ModifierType::CONTROL_MASK)
            && mods.contains(gdk::ModifierType::SHIFT_MASK);
        if cs {
            match key {
                gdk::Key::T | gdk::Key::t => {
                    let n = nb_keys.n_pages() + 1;
                    add_terminal_tab(&nb_keys, &format!("Terminal {n}"));
                    return gtk::glib::Propagation::Stop;
                }
                gdk::Key::W | gdk::Key::w => {
                    if let Some(p) = nb_keys.current_page() {
                        if nb_keys.n_pages() > 1 { nb_keys.remove_page(Some(p)); }
                    }
                    return gtk::glib::Propagation::Stop;
                }
                _ => {}
            }
        }
        if mods.contains(gdk::ModifierType::CONTROL_MASK) && key == gdk::Key::Tab {
            let c = nb_keys.current_page().unwrap_or(0);
            nb_keys.set_current_page(Some((c + 1) % nb_keys.n_pages()));
            return gtk::glib::Propagation::Stop;
        }
        if key == gdk::Key::F11 {
            if win_keys.is_fullscreen() { win_keys.unfullscreen(); }
            else { win_keys.fullscreen(); }
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });
    window.add_controller(key_ctrl);

    // Window drag via header
    let drag = gtk::GestureDrag::new();
    let win_drag = window.clone();
    drag.connect_drag_begin(move |gesture, x, y| {
        if y < 34.0 { // header area
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let surface = win_drag.surface().unwrap();
            surface.downcast_ref::<gdk::Toplevel>().unwrap()
                .begin_move(&gesture.device().unwrap(),
                    gesture.current_button() as i32,
                    x, y,
                    gdk::CURRENT_TIME);
        }
    });
    window.add_controller(drag);

    window.present();
}

fn build_header(window: &ApplicationWindow) -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 0);
    header.add_css_class("header-custom");

    let title = Label::new(Some("  Leuwi Panjang"));
    title.add_css_class("status-green");
    header.append(&title);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    header.append(&spacer);

    let menu_btn = Button::with_label("≡");
    header.append(&menu_btn);

    let min_btn = Button::with_label("─");
    let w = window.clone();
    min_btn.connect_clicked(move |_| { w.minimize(); });
    header.append(&min_btn);

    let max_btn = Button::with_label("□");
    let w = window.clone();
    max_btn.connect_clicked(move |_| {
        if w.is_maximized() { w.unmaximize(); } else { w.maximize(); }
    });
    header.append(&max_btn);

    let close_btn = Button::with_label("✕");
    close_btn.add_css_class("close-btn");
    let w = window.clone();
    close_btn.connect_clicked(move |_| { w.close(); });
    header.append(&close_btn);

    header
}

fn add_terminal_tab(notebook: &Notebook, title: &str) {
    let terminal = Terminal::new();

    // Colors
    let fg = gdk::RGBA::new(0.722, 0.831, 0.796, 1.0);
    let bg = gdk::RGBA::new(0.039, 0.078, 0.063, 0.85);
    let palette: Vec<gdk::RGBA> = vec![
        gdk::RGBA::new(0.157, 0.196, 0.176, 1.0),
        gdk::RGBA::new(1.0, 0.333, 0.333, 1.0),
        gdk::RGBA::new(0.0, 1.0, 0.533, 1.0),
        gdk::RGBA::new(1.0, 0.839, 0.4, 1.0),
        gdk::RGBA::new(0.392, 0.627, 1.0, 1.0),
        gdk::RGBA::new(0.824, 0.549, 1.0, 1.0),
        gdk::RGBA::new(0.314, 0.863, 0.941, 1.0),
        gdk::RGBA::new(0.784, 0.843, 0.804, 1.0),
        gdk::RGBA::new(0.353, 0.431, 0.392, 1.0),
        gdk::RGBA::new(1.0, 0.471, 0.471, 1.0),
        gdk::RGBA::new(0.392, 1.0, 0.667, 1.0),
        gdk::RGBA::new(1.0, 0.902, 0.588, 1.0),
        gdk::RGBA::new(0.549, 0.745, 1.0, 1.0),
        gdk::RGBA::new(0.902, 0.706, 1.0, 1.0),
        gdk::RGBA::new(0.51, 0.941, 1.0, 1.0),
        gdk::RGBA::new(0.941, 0.961, 0.941, 1.0),
    ];
    terminal.set_color_foreground(&fg);
    terminal.set_color_background(&bg);
    let palette_refs: Vec<&gdk::RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg), Some(&bg), &palette_refs);

    // Font
    let font = gtk4::pango::FontDescription::from_string("JetBrainsMono Nerd Font 13");
    terminal.set_font(Some(&font));

    terminal.set_scrollback_lines(10000);
    terminal.set_scroll_on_output(false);
    terminal.set_scroll_on_keystroke(true);
    terminal.set_mouse_autohide(true);

    // Spawn shell via PTY
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let pty = vte4::Pty::new_sync(vte4::PtyFlags::DEFAULT, gtk::gio::Cancellable::NONE).unwrap();

    pty.spawn_async(
        None,
        &[&shell],
        &[],
        gtk::glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        gtk::gio::Cancellable::NONE,
        |result| {
            match result {
                Ok(_pid) => {}
                Err(e) => eprintln!("Failed to spawn shell: {e}"),
            }
        },
    );
    terminal.set_pty(Some(&pty));

    terminal.set_vexpand(true);
    terminal.set_hexpand(true);

    // Close tab on shell exit
    let nb = notebook.clone();
    terminal.connect_child_exited(move |term, _status| {
        if let Some(parent) = term.parent() {
            if let Some(p) = nb.page_num(&parent) {
                if nb.n_pages() > 1 { nb.remove_page(Some(p)); }
            }
        }
    });

    // Tab label
    let tab_box = GtkBox::new(Orientation::Horizontal, 4);
    tab_box.append(&Label::new(Some(title)));
    let close = Button::with_label("×");
    close.set_has_frame(false);
    let nb2 = notebook.clone();
    let term_clone = terminal.clone();
    close.connect_clicked(move |_| {
        if let Some(parent) = term_clone.parent() {
            if let Some(p) = nb2.page_num(&parent) {
                if nb2.n_pages() > 1 { nb2.remove_page(Some(p)); }
            }
        }
    });
    tab_box.append(&close);

    notebook.append_page(&terminal, Some(&tab_box));
    notebook.set_tab_reorderable(&terminal, true);
    notebook.set_current_page(Some(notebook.n_pages() - 1));
}

fn build_status_bar() -> GtkBox {
    let bar = GtkBox::new(Orientation::Horizontal, 6);
    bar.add_css_class("status-bar");
    let dot = Label::new(Some("●"));
    dot.add_css_class("status-green");
    bar.append(&dot);
    bar.append(&Label::new(Some("leuwi-panjang v0.1.0")));
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bar.append(&spacer);
    bar
}
