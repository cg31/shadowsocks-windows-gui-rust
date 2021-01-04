
use native_windows_gui as nwg;
use native_windows_derive as nwd;

use nwg::NativeUi;
use nwd::NwgUi;

use winapi::um::winuser::{SendMessageW, SetForegroundWindow};
use winapi::um::commctrl::{LVM_SETCOLUMNWIDTH};

use std::cell::RefCell;

use crate::client;
use crate::config;
use crate::utils;


const OK_PNG: &[u8] = include_bytes!("../res/StatusOK_16x.png");
const CLOUD_PNG: &[u8] = include_bytes!("../res/Cloud_16x.png");
const SS_PNG: &[u8] = include_bytes!("../res/ssw128.png");


#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(size: (530, 350), position: (300, 300), title: "russ")]
    #[nwg_events(OnInit: [App::init], OnResize: [App::size], OnWindowClose: [App::close], OnWindowMinimize: [App::close])]
    window: nwg::Window,

    #[nwg_resource(source_bin: Some(SS_PNG))]
    tray_icon: nwg::Icon,

    #[nwg_control(icon: Some(&data.tray_icon), tip: Some("russ"))]
    #[nwg_events(MousePressLeftDown: [App::open], MousePressRightDown: [App::show_menu], OnContextMenu: [App::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Open")]
    #[nwg_events(OnMenuItemSelected: [App::open])]
    tray_menu_open: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [App::exit])]
    tray_menu_exit: nwg::MenuItem,

    #[nwg_control(parent: window, popup: true)]
    options_menu: nwg::Menu,

    #[nwg_control(parent: options_menu, text: "Start on boot")]
    #[nwg_events(OnMenuItemSelected: [App::autostart])]
    options_menu_start: nwg::MenuItem,

    // TODO
    //#[nwg_control(parent: options_menu, text: "Exit to tray")]
    //#[nwg_events(OnMenuItemSelected: [App::exit])]
    //options_menu_hide: nwg::MenuItem,

    //#[nwg_control(parent: options_menu, text: "Ping servers")]
    //#[nwg_events(OnMenuItemSelected: [App::exit])]
    //options_menu_ping: nwg::MenuItem,

    #[nwg_resource(initial: 5, size: (16, 16))]
    view_icons: nwg::ImageList,

    #[nwg_layout(parent: window)]
    layout: nwg::DynLayout,

    #[nwg_control(item_count: 10, position: (10, 10), size: (510, 300), focus: true,
                  flags: "VISIBLE|SINGLE_SELECTION|ALWAYS_SHOW_SELECTION",
                  ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT)]
    listview: nwg::ListView,

    #[nwg_control(text: "Options", position: (10, 320), size: (80, 25))]
    #[nwg_events(OnButtonClick: [App::options])]
    options_btn: nwg::Button,

    #[nwg_control(text: "Connect", position: (440, 320), size: (80, 25))]
    #[nwg_events(OnButtonClick: [App::connect])]
    connect_btn: nwg::Button,

    data: RefCell<client::Client>,
}

impl App {
    fn init(&self) {
        let icons = &self.view_icons;

        let icon = utils::load_bitmap(SS_PNG).copy_as_icon();
        self.tray.set_icon(&icon);

        self.listview.set_list_style(nwg::ListViewStyle::Detailed);

        // Load the listview images
        icons.add_bitmap(&utils::load_bitmap(CLOUD_PNG));
        icons.add_bitmap(&utils::load_bitmap(OK_PNG));
        self.listview.set_image_list(Some(icons), nwg::ListViewImageListType::Small);

        // Setup columns
        self.listview.insert_column("Name");
        self.listview.insert_column("Server");
        self.listview.insert_column("Method");
        //self.listview.insert_column("Status"); //TODO
        self.listview.set_headers_enabled(true);

        let hwnd = self.listview.handle.hwnd().unwrap();
        unsafe {
            SendMessageW(hwnd, LVM_SETCOLUMNWIDTH, 0, 120 as isize);
            SendMessageW(hwnd, LVM_SETCOLUMNWIDTH, 1, 120 as isize);
            SendMessageW(hwnd, LVM_SETCOLUMNWIDTH, 2, 150 as isize);
            //SendMessageW(hwnd, LVM_SETCOLUMNWIDTH, 3, 80 as isize);
        }

        /*
        self.listview.set_column_width(0, 120);
        self.listview.set_column_width(1, 120);
        self.listview.set_column_width(2, 150);
        self.listview.set_column_width(3, 80);
        */
        let mut data = self.data.borrow_mut();

        let mut row = 0;
        for svr in &data.config.servers {
            let name = svr.name.clone();
            let server = svr.server.clone();
            let method = svr.method.clone();

            self.listview.insert_item(nwg::InsertListViewItem{ index: Some(row), column_index: 0, text: Some(name), image: Some(0) });
            self.listview.insert_item(nwg::InsertListViewItem{ index: Some(row), column_index: 1, text: Some(server), image: None });
            self.listview.insert_item(nwg::InsertListViewItem{ index: Some(row), column_index: 2, text: Some(method), image: None });

            row += 1;
        }

        if data.config.select >= data.config.servers.len() {
            data.config.select = 0;
        }

        let index = data.config.select;

        data.connect(index);
        self.listview.select_item(index, true);
        self.listview.update_item(index, nwg::InsertListViewItem { image: Some(1), ..Default::default() });

        self.options_menu_start.set_enabled(true);
        if data.config.autostart != 0 {
            self.options_menu_start.set_checked(true);
            let _ = utils::autostart(true);
        } else {
            self.options_menu_start.set_checked(false);
            let _ = utils::autostart(false);
        }

        self.layout.add_child((0, 0), (100, 100), &self.listview);
        self.layout.add_child((0, 100), (0, 0), &self.options_btn);
        self.layout.add_child((100, 100), (0, 0), &self.connect_btn);
    }

    fn size(&self) {
        self.layout.fit();
    }

    fn options(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.options_menu.popup(x, y);
    }

    fn connect(&self) {
        if let Some(index) = self.listview.selected_item() {
            let mut data = self.data.borrow_mut();

            if data.config.select != index {
                self.listview.update_item(data.config.select, nwg::InsertListViewItem { image: Some(0), ..Default::default() });
                data.connect(index);
                self.listview.update_item(index, nwg::InsertListViewItem { image: Some(1), ..Default::default() });
            }
        }

        self.listview.set_focus();
    }

    fn autostart(&self) {
        let mut data = self.data.borrow_mut();

        // flip state
        if self.options_menu_start.checked() {
            self.options_menu_start.set_checked(false);
            let _ = utils::autostart(false);
            data.config.autostart = 0;
        } else {
            self.options_menu_start.set_checked(true);
            let _ = utils::autostart(true);
            data.config.autostart = 1;
        }
    }

    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn open(&self) {
        let hwnd = self.listview.handle.hwnd().unwrap();
        self.window.set_visible(true);
        self.window.set_focus();
        unsafe { SetForegroundWindow(hwnd); }
    }

    fn close(&self) {
        self.window.set_visible(false);
    }

    fn exit(&self) {
        let data = self.data.borrow_mut();
        let _ = config::Config::save(&data.config);

        nwg::stop_thread_dispatch();
    }
}


pub fn open() {
    let data = client::Client::new();
    let app = App{ data: RefCell::new(data), ..Default::default() };
    let _appui = App::build_ui(app).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}

