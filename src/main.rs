use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Label, Notebook, Orientation, CssProvider, WindowControls, PackType};
use gdk4 as gdk;
use vte4::prelude::*;
use vte4::Terminal;

const APP_ID: &str = "com.situkangsayur.leuwi-panjang";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(r#"
        window, window.background { background-color: #0D1117; }
        
        /* Tab bar = titlebar area */
        .tab-titlebar {
            background-color: #0D1117;
            min-height: 36px;
            padding: 0 4px;
        }
        
        /* Tabs like Chrome */
        notebook header {
            background-color: #0D1117;
            border: none;
            padding: 0;
            margin: 0;
        }
        notebook header tab {
            background-color: transparent;
            color: #8B949E;
            border: none;
            padding: 6px 16px;
            margin: 4px 1px 0 1px;
            border-radius: 10px 10px 0 0;
            min-height: 24px;
        }
        notebook header tab:checked {
            background-color: #161B22;
            color: #E6EDF3;
        }
        notebook header tab:hover:not(:checked) {
            background-color: #1C2129;
            color: #C9D1D9;
        }
        notebook header tab button {
            min-height: 16px;
            min-width: 16px;
            padding: 0;
            margin: 0;
            border-radius: 50%;
            color: #8B949E;
        }
        notebook header tab button:hover {
            background-color: rgba(255,255,255,0.1);
            color: #E6EDF3;
        }
        notebook > stack {
            background-color: #161B22;
        }
        
        /* Window control buttons */
        windowcontrols button {
            min-height: 20px;
            min-width: 20px;
            padding: 4px;
            margin: 0 1px;
            border-radius: 50%;
            background: transparent;
            color: #8B949E;
        }
        windowcontrols button:hover {
            background-color: rgba(255,255,255,0.1);
        }
        windowcontrols button.close:hover {
            background-color: #E94560;
            color: white;
        }
        
        /* + new tab button */
        .new-tab-btn {
            color: #8B949E;
            background: transparent;
            border: none;
            padding: 4px 8px;
            margin: 4px 2px;
            border-radius: 8px;
            min-height: 24px;
        }
        .new-tab-btn:hover {
            background-color: #1C2129;
            color: #E6EDF3;
        }
        
        /* Menu button */
        .menu-btn {
            color: #8B949E;
            background: transparent;
            border: none;
            padding: 4px 8px;
            border-radius: 8px;
        }
        .menu-btn:hover {
            background-color: #1C2129;
            color: #E6EDF3;
        }
        
        /* Status bar */
        .status-bar {
            background-color: #0D1117;
            color: #8B949E;
            padding: 1px 12px;
            font-size: 11px;
            min-height: 22px;
        }
        .status-green { color: #3FB950; }
    "#);
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
        .build();

    // === TITLEBAR = TAB BAR (Chrome style) ===
    let titlebar = GtkBox::new(Orientation::Horizontal, 0);
    titlebar.add_css_class("tab-titlebar");

    // Notebook for tabs
    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_show_border(false);
    notebook.set_vexpand(true);
    notebook.set_hexpand(true);

    // The notebook header (tabs) goes INTO the titlebar
    // We move notebook tab strip to titlebar by placing notebook action widgets
    
    // + button at end of tabs
    let nb_clone = notebook.clone();
    let add_btn = Button::with_label("+");
    add_btn.add_css_class("new-tab-btn");
    add_btn.connect_clicked(move |_| {
        let n = nb_clone.n_pages() + 1;
        add_terminal_tab(&nb_clone, &format!("Terminal {n}"));
    });
    
    let end_box = GtkBox::new(Orientation::Horizontal, 2);
    
    // Hamburger menu
    let menu_btn = Button::with_label("≡");
    menu_btn.add_css_class("menu-btn");
    end_box.append(&menu_btn);
    
    end_box.append(&add_btn);
    notebook.set_action_widget(&end_box, PackType::End);

    // First tab
    add_terminal_tab(&notebook, "Terminal 1");

    // Use the NOTEBOOK ITSELF as titlebar — tabs ARE the titlebar
    // We wrap notebook header area + window controls
    let tab_header = gtk::HeaderBar::new();
    tab_header.set_show_title_buttons(true);
    tab_header.set_decoration_layout(Some(":minimize,maximize,close"));
    tab_header.set_title_widget(Some(&Label::new(None))); // empty title
    tab_header.add_css_class("tab-titlebar");
    
    // This doesn't work well... let me use a different approach
    // Just set decorated=false and build our own header
    
    // Actually the BEST way for Chrome-style:
    // Use HeaderBar with notebook as title widget
    // But notebook needs to be separate...
    
    // Simplest working approach: decorated(false) + custom top bar
    window.set_decorated(false);
    
    // Top bar with tabs area + window controls
    let top_bar = GtkBox::new(Orientation::Horizontal, 0);
    top_bar.add_css_class("tab-titlebar");
    top_bar.set_hexpand(true);
    
    // Make top bar draggable for window move
    let drag_handle = gtk::WindowHandle::new();
    drag_handle.set_child(Some(&top_bar));
    drag_handle.set_hexpand(true);
    
    // Spacer (draggable area before controls)
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    top_bar.append(&spacer);
    
    // Window controls (minimize, maximize, close)
    let win_controls = WindowControls::new(PackType::End);
    top_bar.append(&win_controls);
    
    // Main layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.append(&drag_handle);  // tab bar + window controls
    main_box.append(&notebook);      // terminal content
    
    // Status bar
    let status = GtkBox::new(Orientation::Horizontal, 6);
    status.add_css_class("status-bar");
    let dot = Label::new(Some("●"));
    dot.add_css_class("status-green");
    status.append(&dot);
    status.append(&Label::new(Some("leuwi-panjang v0.1.0")));
    main_box.append(&status);
    
    window.set_child(Some(&main_box));
    
    // Move notebook tabs to top_bar
    // GTK4 Notebook places tabs inside itself. 
    // For Chrome-style we set notebook tab position to top
    // and it naturally sits below our drag handle.
    notebook.set_tab_pos(gtk::PositionType::Top);
    
    // Keyboard shortcuts
    let nb_k = notebook.clone();
    let win_k = window.clone();
    let kc = gtk::EventControllerKey::new();
    kc.connect_key_pressed(move |_, key, _, mods| {
        let cs = mods.contains(gdk::ModifierType::CONTROL_MASK) && mods.contains(gdk::ModifierType::SHIFT_MASK);
        if cs {
            match key {
                gdk::Key::T | gdk::Key::t => {
                    let n = nb_k.n_pages() + 1;
                    add_terminal_tab(&nb_k, &format!("Terminal {n}"));
                    return gtk::glib::Propagation::Stop;
                }
                gdk::Key::W | gdk::Key::w => {
                    if let Some(p) = nb_k.current_page() {
                        if nb_k.n_pages() > 1 { nb_k.remove_page(Some(p)); }
                    }
                    return gtk::glib::Propagation::Stop;
                }
                _ => {}
            }
        }
        if mods.contains(gdk::ModifierType::CONTROL_MASK) && key == gdk::Key::Tab {
            let c = nb_k.current_page().unwrap_or(0);
            nb_k.set_current_page(Some((c + 1) % nb_k.n_pages()));
            return gtk::glib::Propagation::Stop;
        }
        if key == gdk::Key::F11 {
            if win_k.is_fullscreen() { win_k.unfullscreen(); } else { win_k.fullscreen(); }
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });
    window.add_controller(kc);
    
    window.present();
}

fn add_terminal_tab(notebook: &Notebook, title: &str) {
    let terminal = Terminal::new();
    
    let fg = gdk::RGBA::new(0.902, 0.910, 0.957, 1.0);   // #E6EDF3
    let bg = gdk::RGBA::new(0.086, 0.106, 0.133, 1.0);   // #161B22
    let palette: Vec<gdk::RGBA> = vec![
        gdk::RGBA::new(0.282, 0.322, 0.376, 1.0),  // #48515E
        gdk::RGBA::new(1.0, 0.475, 0.435, 1.0),     // #FF796F - red
        gdk::RGBA::new(0.247, 0.725, 0.314, 1.0),   // #3FB950 - green
        gdk::RGBA::new(0.831, 0.690, 0.220, 1.0),   // #D4B036 - yellow
        gdk::RGBA::new(0.345, 0.608, 0.976, 1.0),   // #589BF9 - blue
        gdk::RGBA::new(0.741, 0.502, 0.976, 1.0),   // #BD80F9 - purple
        gdk::RGBA::new(0.318, 0.827, 0.886, 1.0),   // #51D3E2 - cyan
        gdk::RGBA::new(0.788, 0.820, 0.886, 1.0),   // #C9D1E2 - white
        gdk::RGBA::new(0.408, 0.455, 0.518, 1.0),   // #687484 - bright black
        gdk::RGBA::new(1.0, 0.584, 0.553, 1.0),     // #FF958D
        gdk::RGBA::new(0.345, 0.827, 0.424, 1.0),   // #58D36C
        gdk::RGBA::new(0.929, 0.827, 0.318, 1.0),   // #EDD351
        gdk::RGBA::new(0.498, 0.737, 1.0, 1.0),     // #7FBCFF
        gdk::RGBA::new(0.839, 0.639, 1.0, 1.0),     // #D6A3FF
        gdk::RGBA::new(0.435, 0.914, 0.965, 1.0),   // #6FE9F6
        gdk::RGBA::new(0.910, 0.929, 0.976, 1.0),   // #E8EDF9
    ];
    let prefs: Vec<&gdk::RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg), Some(&bg), &prefs);
    
    let font = gtk4::pango::FontDescription::from_string("JetBrainsMono Nerd Font 13");
    terminal.set_font(Some(&font));
    terminal.set_scrollback_lines(10000);
    terminal.set_scroll_on_output(false);
    terminal.set_scroll_on_keystroke(true);
    terminal.set_mouse_autohide(true);
    terminal.set_vexpand(true);
    terminal.set_hexpand(true);
    
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let pty = vte4::Pty::new_sync(vte4::PtyFlags::DEFAULT, gtk::gio::Cancellable::NONE).unwrap();
    pty.spawn_async(
        None, &[&shell], &[], gtk::glib::SpawnFlags::DEFAULT,
        || {}, -1, gtk::gio::Cancellable::NONE, |_| {},
    );
    terminal.set_pty(Some(&pty));
    
    let nb = notebook.clone();
    terminal.connect_child_exited(move |term, _| {
        if let Some(p) = nb.page_num(term) {
            if nb.n_pages() > 1 { nb.remove_page(Some(p)); }
        }
    });
    
    // Tab label
    let tab_box = GtkBox::new(Orientation::Horizontal, 4);
    tab_box.append(&Label::new(Some(title)));
    let close = Button::with_label("×");
    close.set_has_frame(false);
    let nb2 = notebook.clone();
    let t = terminal.clone();
    close.connect_clicked(move |_| {
        if let Some(p) = nb2.page_num(&t) {
            if nb2.n_pages() > 1 { nb2.remove_page(Some(p)); }
        }
    });
    tab_box.append(&close);
    
    notebook.append_page(&terminal, Some(&tab_box));
    notebook.set_tab_reorderable(&terminal, true);
    notebook.set_current_page(Some(notebook.n_pages() - 1));
}
