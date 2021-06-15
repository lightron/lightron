#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use std::collections::HashMap;
use serde_derive::{Deserialize,Serialize};
use std::io::prelude::*;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::path::{PathBuf};
use std::{thread,time};
use plotters::prelude::*;
use plotters;
use sysinfo::{ProcessExt, SystemExt};
use std::error::Error;
use plotters::style::Color;
use std::time::SystemTime;
use plotters_bitmap::bitmap_pixel::RGBPixel;
use plotters_bitmap::BitMapBackend;
use std::sync::mpsc;
use std::cell::RefCell;
use systemstat::{System, Platform,saturating_sub_bytes};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use toml::{to_string,from_str};
use vlc::*;
use fltk::prelude::WidgetBase;
use fltk::prelude::MenuExt;
use fltk::prelude::WidgetExt;
use fltk::prelude::InputExt;
use fltk::prelude::GroupExt;
use fltk::prelude::WindowExt;
use fltk::prelude::ButtonExt;
use fltk::prelude::DisplayExt;
use fltk::{app::*,button::*,window::*,input::*,dialog::*,frame::*,image::*,group::*,text::*,tree::*,draw,enums};

#[derive(Debug, Clone)]
pub struct MyDial {
    main_wid: Frame,
    value: Rc<RefCell<i32>>,
    value_frame: Frame,
}

fn get_cpu_load(sys: &System) -> f64
{
    match sys.cpu_load_aggregate() {
        Ok(cpu)=> {
            thread::sleep(time::Duration::from_secs(1));
            let load : f64 = cpu.done().unwrap().user as f64 * 100.0;
            load
        },
        Err(_x) => 0.00
    }
}

fn get_memory_load(sys: &System) -> f64
{
    match sys.memory() {
        Ok(mem) => {
            let used = saturating_sub_bytes(mem.total, mem.free).to_string();
            used[..used.len()-3].parse().unwrap()
        }
        Err(_x) => 0.00
    }
}


impl MyDial {
    pub fn new(x: i32, y: i32, w: i32, h: i32, label: &'static str, from_total: i32) -> Self {
        let value = Rc::from(RefCell::from(0));
        let mut main_wid = Frame::new(x, y, w, h, label).with_align(enums::Align::Top);
        // println!("height = {}",main_wid.h());
        let mut value_frame = Frame::new(main_wid.x(), main_wid.y() + (main_wid.h() as f32/2.4) as i32, main_wid.w(), 40, "0");
        value_frame.set_label_size(26);
        let value_c = value.clone();
        main_wid.draw(move |w| {
            draw::set_draw_rgb_color(230, 230, 230);
            draw::draw_pie(w.x(), w.y(), w.w(), w.h(), 0., 180.);  
            draw::set_draw_hex_color(0xb0bf1a);
            draw::draw_pie(
                w.x(),
                w.y(),
                w.w(),
                w.h(),
                (from_total - *value_c.borrow()) as f64 * 1.8,
                180.,
            );
            draw::set_draw_color(enums::Color::Cyan);
            draw::draw_pie(
                w.x() - 50 + w.w() / 2,
                w.y() - 50 + w.h() / 2,
                100,
                100,
                0.,
                360.,
            );
        });
        Self { main_wid, value, value_frame }
    }
    pub fn value(&self) -> i32 {
        *self.value.borrow()
    }
    pub fn set_value(&mut self, val: i32) {
        *self.value.borrow_mut() = val;
        self.value_frame.set_label(&val.to_string());
        self.main_wid.redraw();
    }
}

impl Deref for MyDial {
    type Target = Frame;

    fn deref(&self) -> &Self::Target {
        &self.main_wid
    }
}

impl DerefMut for MyDial {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.main_wid
    }
}


#[derive(Deserialize, Serialize, Debug, Clone)]
struct Website {
    name: String,
    class: String,
    access: String,
    resource: PathBuf,
    certificate: PathBuf,
    private_key: PathBuf,
    port_no: u16,
    push_protocol_files: Vec<PathBuf>,
    log_level: String
}

#[derive(Copy, Clone)]
pub enum Message {
    BrowseDocs,
    BrowseCerts,
    BrowseKeys,
    Write,
    ClassToggle,
    AccessToggle,
    AddNewWebsite,
    RemoveWebsite,
    Remove,
    Search,
    LogLevelChange,
    BrowsePushFiles,
    TabChange
}

fn print_headers(group : &mut Group)
{
    let (width,height) = fltk::app::screen_size();
    println!("width = {}, height = {}",width,height);
    let mut frame_header_class = Group::new((width*0.35)  as i32,(height*0.18) as i32,(width/15.36) as i32,(height/20.6) as i32,"Sr. No.");
    dummy_button(&mut frame_header_class, enums::FrameType::EngravedBox, enums::Color::from_u32(0x00bfff));
    group.add(&frame_header_class);

    let mut frame_header_resource = Group::default().with_size((width/15.36) as i32,(height/20.6) as i32).right_of(&frame_header_class,20).with_label("Name");
    dummy_button(&mut frame_header_resource, enums::FrameType::EngravedBox, enums::Color::from_u32(0x00bfff));
    group.add(&frame_header_resource);

    let mut frame_header_port_no = Group::default().with_size((width/15.36) as i32,(height/20.6) as i32).right_of(&frame_header_resource,20).with_label("Class");
    dummy_button(&mut frame_header_port_no, enums::FrameType::EngravedBox, enums::Color::from_u32(0x00bfff));
    group.add(&frame_header_port_no);

    let mut frame_header_status = Group::default().with_size((width/15.36) as i32,(height/20.6) as i32).right_of(&frame_header_port_no,20).with_label("Port No.");
    dummy_button(&mut frame_header_status, enums::FrameType::EngravedBox, enums::Color::from_u32(0x00bfff));
    group.add(&frame_header_status);
}

fn dummy_button(grp:&mut Group, frametype:enums::FrameType, framecolor:enums::Color) {
    grp.set_frame(frametype);
    grp.set_color(framecolor);
    grp.set_align(enums::Align::Center);
    grp.end();
}

fn print_get_entries(con : String, group : &mut Group) {  

    let (width,height) = fltk::app::screen_size();
    let config: HashMap<String, Vec<Website>> = from_str(&con).unwrap();
    let websites = config["websites"].clone();
    let mut sr_no = 1;
    let mut counter = 0;
    for website in websites {
        let mut name_entry=Group::new((width*0.35) as i32,(height*0.26) as i32 + counter,(width/15.36) as i32,(height/20.6) as i32,None).with_label(&sr_no.to_string());
        dummy_button(&mut name_entry,enums::FrameType::GtkDownBox, enums::Color::from_u32(0x05f7ff));
        group.add(&name_entry);

        let mut class_entry=Group::default().with_size((width/15.36) as i32,(height/20.6) as i32).right_of(&name_entry,20).with_label(&website.name.to_string());
        dummy_button(&mut class_entry,enums::FrameType::GtkDownBox, enums::Color::from_u32(0x05f7ff));
        group.add(&class_entry);
        
        let mut port_no_entry=Group::default().with_size((width/15.36) as i32,(height/20.6) as i32).right_of(&class_entry,20).with_label(&website.class.to_string());
        dummy_button(&mut port_no_entry, enums::FrameType::GtkDownBox, enums::Color::from_u32(0x05f7ff));
        group.add(&port_no_entry);
        
        let mut status_entry=Group::default().with_size((width/15.36) as i32,(height/20.6) as i32).right_of(&port_no_entry,20).with_label(&website.port_no.to_string());
        dummy_button(&mut status_entry,enums::FrameType::GtkDownBox, enums::Color::from_u32(0x05f7ff));
        group.add(&status_entry);
        counter += (height/16.48) as i32;
        sr_no+=1;
    }
}


fn read_conf() -> String {
    let mut file = if cfg!(target_os = "windows") {
        OpenOptions::new().read(true).write(true).create(true).open("..\\lightron.conf").unwrap()
    }
    else {
        OpenOptions::new().read(true).write(true).create(true).open("../lightron.conf").unwrap()
    };
    
    let mut con=String::new();
    file.read_to_string(&mut con).expect("Unable to read to string.");
    con
}


fn main()  -> Result<(), Box<dyn Error>>  {
    let sys = System::new();
    let system = sysinfo::System::new_all();
    if !cfg!(target_os = "windows") { 
        extern "C" {
            pub fn XInitThreads() -> i32;
        }
        unsafe { XInitThreads(); }
    }
    
    // INTRO //
    let intro_app = App::default();
    // Create inner window to act as embedded media player
    let mut win = Window::default().with_size(500,500).center_screen();
    let mut vlc_win = Window::new(0, 0, 500, 500, "");
    vlc_win.end();
    vlc_win.set_color(enums::Color::Black);
    win.end();
    win.set_border(false);
    //vlc_win.make_modal(true);
    win.show();
    win.make_resizable(true);
    let instance = Instance::new().unwrap();
    let md = if cfg!(target_os = "windows") {
        Media::new_path(&instance, "assets\\Lightron.mp4")
    }
    else {
        Media::new_path(&instance, "assets/Lightron.mp4")
    };
    match md {
        Some(md) => {
            let mdp = MediaPlayer::new(&instance).unwrap();
            mdp.set_media(&md);
            let handle = vlc_win.raw_handle();
            // For Linux
            #[cfg(target_os = "linux")]
            mdp.set_xwindow(handle as u32);
            // For Windows
            #[cfg(target_os = "windows")]
            mdp.set_hwnd(handle);
            // For MacOS
            #[cfg(target_os = "macos")]
            mdp.set_nsobject(handle);
            mdp.set_key_input(false);
            mdp.set_mouse_input(false);
            mdp.play().unwrap();
            while intro_app.wait() {
                if mdp.state() == State::Ended {
                    intro_app.quit();
                }
                awake();
            }
        }
        None => {
            intro_app.quit();
        }
    }
    
    // INTRO ENDS HERE //

    let main_app = App::default().with_scheme(Scheme::Gleam);
    let font = if cfg!(target_os = "windows") {
        main_app.load_font("assets\\Overpass Regular 400.ttf")
    }
    else {
        main_app.load_font("assets/Overpass Regular 400.ttf")
    };
    match font {
        Ok(font) => set_font(enums::Font::by_name(&font)),
        Err(_) => ()
    }
    let (width,height) = fltk::app::screen_size();
    let mut main_wind=Window::default().with_size(width as i32,height as i32).with_label("Lightron Web Server").center_screen();
    let icon = if cfg!(target_os = "windows") {
        PngImage::load(&std::path::Path::new("assets\\lightron_icon.png"))
    }
    else {
        PngImage::load(&std::path::Path::new("assets/lightron_icon.png"))
    };
    match icon {
        Ok(icon) => main_wind.set_icon(Some(icon)),
        Err(_) => ()
    }
    let mut tabs=Tabs::default().with_size(width as i32,height as i32);
    let icons = if cfg!(target_os = "windows") {
        main_app.load_font("assets\\icons.ttf")
    }
    else {
        main_app.load_font("assets/icons.ttf")
    };
    let icons = match icons {
        Ok(icons) => icons,
        Err(_) => "Helvetica".to_string()
    };
    let mut main_tab = Group::new(10, 45, width as i32, height as i32, "\u{E800} \n\tDashboard\t");
    main_tab.set_label_font(enums::Font::by_name(&icons));
    tabs.set_selection_color(enums::Color::Yellow);
    main_tab.set_label_size(19);
    main_tab.clear_visible_focus();
    let mut main_frame = Frame::default().size_of(&tabs);
    let dash_img = if cfg!(target_os = "windows") {
        JpegImage::load(&std::path::Path::new("assets\\dashboard-1.jpg"))
    }
    else {
        JpegImage::load(&std::path::Path::new("assets/dashboard-1.jpg"))
    };
    match dash_img {
        Ok(dash_img) => main_frame.set_image(Some(dash_img)),
        Err(_) => (),
    }

    let (s, r) = fltk::app::channel::<Message>();

    let mut frames_group = Group::default().with_pos((width as i32)/2,(height as i32)/5);
    frames_group.end();
    let mut flag:bool = false;
    let mut lightron_pid = 0;
    for (pid, proc_) in system.get_processes() {
        if cfg!(target_os = "windows") && proc_.name() == "lightron-core.exe" {
            flag=true;
            lightron_pid=*pid;
            break;
        }
        else if proc_.name() == "lightron" {
            flag=true;
            lightron_pid=*pid;
            break;
        }
    }
    if flag {
        let mut pid_print = Group::new((width*0.06)  as i32,(height*0.20) as i32,(width/7.68) as i32,(height/4.12) as i32,None).with_label(&lightron_pid.to_string());
        let _pid_str = Group::new((width*0.063)  as i32,(height*0.28) as i32,(width/7.68) as i32,(height/4.12) as i32,"Lightron is Running on\nPID");
        dummy_button(&mut pid_print,enums::FrameType::GleamDownBox, enums::Color::from_u32(0xff8c00));
        pid_print.set_label_size(30); 
        if let Some(process) = system.get_process(lightron_pid.try_into().unwrap()) {
            let lightron_mem = process.memory();
            let lightron_cpu = process.cpu_usage() as i32;
            let mut dummy_grp = Group::new((width*0.81)  as i32,(height*0.60) as i32,(width/7.68) as i32,(height/4.12) as i32,"");
            let mut memory_use = Group::new((width*0.81)  as i32,(height*0.72) as i32,(width/7.68) as i32,(height/4.12) as i32,None).with_label(&(lightron_mem.to_string() + " kB"));
            let _memory_use_server_print = Group::new((width*0.81)  as i32,(height*0.67) as i32,(width/7.68) as i32,(height/4.12) as i32,"Memory Used by Lightron : ");
            let mut cpu_use = Group::new((width*0.81)  as i32,(height*0.80) as i32,(width/7.68) as i32,(height/4.12) as i32,None).with_label(&(lightron_cpu.to_string() + "%"));
            let _cpu_use_str = Group::new((width*0.81)  as i32,(height*0.75) as i32,(width/7.68) as i32,(height/4.12) as i32,"CPU used by Lightron : ");
            dummy_button(&mut dummy_grp,enums::FrameType::GleamDownBox,enums::Color::from_u32(0x4cbb17));
            memory_use.set_label_size(25);
            cpu_use.set_label_size(25);
            // println!("{} kB", process.memory());
            // println!("{}%", process.cpu_usage());
        }
    }
    else{
        let mut pid_print = Group::new((width*0.06)  as i32,(height*0.20) as i32,(width/7.68) as i32,(height/4.12) as i32,"Lightron Webserver \n\nis Not started yet");
        dummy_button(&mut pid_print,enums::FrameType::GleamDownBox, enums::Color::from_u32(0xff8c00));
        match sys.uptime() {  
            Ok(uptime) => {
                let mut frame_header_uptime = Group::new((width*0.81)  as i32,(height*0.60) as i32,(width/7.68) as i32,(height/4.12) as i32,None).with_label(&(uptime.as_secs().to_string()+" s"));
                let _frame_header_uptime_str = Group::new((width*0.81)  as i32,(height*0.68) as i32,(width/7.68) as i32,(height/4.12) as i32,"Uptime : ");
                frame_header_uptime.set_label_size(30);
                dummy_button(&mut frame_header_uptime,enums::FrameType::GleamDownBox,enums::Color::from_u32(0x4cbb17));
                // println!("\nUptime: {:?}", uptime)
            },
            Err(x) => println!("\nUptime: error: {}", x)
        }
    }
    match sys.memory() {   
        Ok(mem) =>{
            let memo_used=&saturating_sub_bytes(mem.total, mem.free).to_string();
            let mut frame_header_class = Group::new((width*0.81)  as i32,(height*0.20) as i32,(width/7.68) as i32,(height/4.12) as i32,None).with_label(&memo_used);
            let _memory_used_str = Group::new((width*0.81)  as i32,(height*0.28) as i32,(width/7.68) as i32,(height/4.12) as i32,"Memory used :");
            dummy_button(&mut frame_header_class,enums::FrameType::GleamDownBox,enums::Color::from_u32(0xff0000));
            frame_header_class.set_label_size(30);
            // println!("\nMemory: {} used ", saturating_sub_bytes(mem.total, mem.free))
        },
        Err(x) => println!("\nMemory: error: {}", x)
    }
    let processes = system.get_processes().len();
    let mut process_print = Group::new((width*0.06)  as i32,(height*0.60) as i32,(width/7.68) as i32,(height/4.12) as i32,None).with_label(&processes.to_string());
    let _process_str = Group::new((width*0.06)  as i32,(height*0.69) as i32,(width/7.68) as i32,(height/4.12) as i32,"Processes Running : ");
    dummy_button(&mut process_print,enums::FrameType::GleamDownBox,enums::Color::from_u32(0xFCD12A));
    process_print.set_label_size(25);

    

    let mut add_new_website_button = Button::new((width*0.30) as i32,(height*0.65) as i32,(width/5.12) as i32,(height/20.6) as i32,"\u{E801} \tAdd New Website");
    add_new_website_button.set_label_font(enums::Font::by_name(&icons));
    add_new_website_button.set_frame(enums::FrameType::GleamUpBox);
    add_new_website_button.set_color(enums::Color::from_u32(0xffdc73));
    add_new_website_button.clear_visible_focus();
    let mut remove_website_button = Button::default().with_size((width/5.12) as i32,(height/20.6) as i32).right_of(&add_new_website_button,20).with_label("\u{E803} \tRemove Existing Website");
    remove_website_button.set_label_font(enums::Font::by_name(&icons));
    remove_website_button.set_frame(enums::FrameType::GleamUpBox);
    remove_website_button.set_color(enums::Color::from_u32(0xffdc73));
    remove_website_button.clear_visible_focus();
    add_new_website_button.emit(s,Message::AddNewWebsite);
    remove_website_button.emit(s,Message::RemoveWebsite);
    let mut cpu_dial = MyDial::new((width*0.437) as i32,(height*0.75) as i32,(width/7.68) as i32,(height/4.12) as i32, "CPU Load %",100);
    cpu_dial.set_label_size(22);
    cpu_dial.set_label_color(enums::Color::from_u32(0x797979));
    let (tx, rx) = mpsc::channel(); // 
    let total_mem : f64;
    match sys.memory() {
        Ok(mem) => {
            total_mem = ((mem.total.as_u64() as f64 / 1024.00) / 1024.00 ) / 1024.00;
        }
        Err(_) => {
            total_mem = 0.00;
        }
    };
    thread::spawn(move || {
        loop {
            let load = get_cpu_load(&sys);
            let mem_used = get_memory_load(&sys);
            tx.send((load,mem_used)).unwrap();
            awake();
            // println!("Sent {},{}",load,mem_used);    
        }
    });
    main_tab.end();
    // Main tab ends here // 

    // Statistics tab starts here //
    let mut statistics_tab = Group::new(10, 45, width as i32, height as i32, "\u{E802} \n   Statistics\t");
    statistics_tab.set_label_font(enums::Font::by_name(&icons));
    statistics_tab.set_label_size(19);
    statistics_tab.clear_visible_focus();
    let mut stat_frame = Frame::default().size_of(&tabs);
    let stat_img= if cfg!(target_os = "windows") {
        JpegImage::load(&std::path::Path::new("assets\\stat.jpg"))
    }
    else {
        JpegImage::load(&std::path::Path::new("assets/stat.jpg"))
    };
    match stat_img {
        Ok(stat_img) => stat_frame.set_image(Some(stat_img)),
        Err(_) => (),
    }
    
    
    // CPU GRAPH //
    let mut cpu_buf = vec![0u8; ((width/2.3) as i32 as usize) * ((height/2.2) as i32 as usize) * 3]; // creates vector with all elements as zero
    let mut cpu_win = Window::default().with_size((width/2.3) as i32, (height/2.2) as i32).with_pos((width*0.03) as i32,(height*0.30) as i32);
    let mut cpu_frame = Frame::default().size_of(&cpu_win);
    cpu_win.end();
    statistics_tab.add(&cpu_win);
    cpu_win.show();
    let cpu_root = BitMapBackend::<RGBPixel>::with_buffer_and_format(&mut cpu_buf, ((width/2.3) as i32 as u32, (height/2.2) as i32 as u32)).unwrap().into_drawing_area();
    cpu_root.fill(&WHITE).unwrap();
    let mut cpu_chart = ChartBuilder::on(&cpu_root)
        .margin(0)
        .set_label_area_size(LabelAreaPosition::Left, 35)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .set_label_area_size(LabelAreaPosition::Right, 40)
        .set_label_area_size(LabelAreaPosition::Top, 30)
        .build_cartesian_2d(0.00..60.00, 0.00..100.00)
        .unwrap();

    cpu_chart.configure_mesh().label_style(("sans-serif", 10).into_font().color(&BLACK)).axis_style(&BLACK).draw().unwrap();

    let cpu_cs = cpu_chart.into_chart_state();
    drop(cpu_root);
    let mut cpu_data = Vec::new();
    // CPU GRAPH END //

    // MEMORY GRAPH //
    let mut mem_buf = vec![0u8; ((width/2.3) as i32 as usize) * ((height/2.2) as i32 as usize) * 3]; // creates vector with all elements as zero
    let mut mem_win = Window::default().with_size((width/2.3) as i32, (height/2.2) as i32).with_pos((width*0.53) as i32,(height*0.30) as i32);

    let mut mem_frame = Frame::default().size_of(&mem_win);
    mem_win.end();
    statistics_tab.add(&mem_win);
    //mem_win.make_modal(true);
    mem_win.show();
    let mem_root = BitMapBackend::<RGBPixel>::with_buffer_and_format(&mut mem_buf, ((width/2.3) as i32 as u32,(height/2.2) as i32 as u32)).unwrap().into_drawing_area();
    mem_root.fill(&WHITE).unwrap();
    let mut mem_chart = ChartBuilder::on(&mem_root)
        .margin(0)
        .set_label_area_size(LabelAreaPosition::Left, 35)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .set_label_area_size(LabelAreaPosition::Right, 40)
        .set_label_area_size(LabelAreaPosition::Top, 30)
        .build_cartesian_2d(0.00..60.00, 0.00..total_mem)
        .unwrap();
    mem_chart.configure_mesh().label_style(("sans-serif", 10).into_font().color(&BLACK)).axis_style(&BLACK).draw().unwrap();

    let mem_cs = mem_chart.into_chart_state();
    drop(mem_root);
    let mut mem_data = Vec::new();
    // MEMORY GRAPH END //
    let mut cpu_label = Frame::default().with_size((width/51.2) as i32,(height/41.2) as i32).with_pos((width*0.24) as i32,(height*0.25) as i32).with_label("CPU Utilization Graph");
    // cpu_label.set_label_color(enums::Color::White);
    cpu_label.set_label_size(20);
    statistics_tab.add(&cpu_label);
    let mut mem_label = Frame::default().with_size((width/51.2) as i32,(height/41.2) as i32).with_pos((width*0.74) as i32,(height*0.25) as i32).with_label("Memory Utilization Graph");
    mem_label.set_label_size(20);
    // mem_label.set_label_color(enums::Color::White);
    statistics_tab.add(&mem_label);
    statistics_tab.end();
    // statistics tab ends here //

    // Logs tab starts here //
    let mut log_tab = Group::new(10, 45, width as i32, height as i32, "\u{F0F6} \n      Logs  \t");
    log_tab.set_label_font(enums::Font::by_name(&icons));
    let mut log_frame = Frame::default().size_of(&tabs);
    log_tab.clear_visible_focus();
    log_tab.set_label_size(19);
    let mut paths = vec![];
    let current_dir = if cfg!(target_os = "windows") {
        std::path::PathBuf::from(r"..\")
    }
    else {
        std::path::PathBuf::from(r"../")
    };
    let path: String = current_dir
        .to_str()
        .unwrap()
        .chars()
        .enumerate()
        .map(|(_, c)| match c {
            '\\' => '/', // change window paths to posix paths
            _ => c,
        })
        .collect();
    let directory_read = if cfg!(target_os = "windows") {
        "..\\"
    }
    else {
        "../"
    };
    let files = std::fs::read_dir(directory_read).unwrap();
    for file in files {
        let entry = file.unwrap();
        let typ = entry.file_type().unwrap();
        if !typ.is_dir() && !typ.is_symlink() {
            let mut p = path.clone();
            p.push('/');
            p.push_str(&entry.file_name().into_string().unwrap());
            paths.push(p);
        }
    }

    let text_buffer = TextBuffer::default();
    let mut tree = Tree::new(15, 95, (width/5.12) as i32, (height - 120.0) as i32, "");
    let mut disp = TextDisplay::default().with_size((width - 350.0) as i32,(height - 120.0) as i32).right_of(&tree, 20);

   

    disp.set_buffer(text_buffer);

    for path in paths {
        tree.add(&path);
    }

    let mut items = tree.get_items().unwrap();
    let root = &mut items.as_mut_slice()[0];
    if let Some(label) = root.label() {
        if label == "ROOT" {
            root.set_label("/");
        }
    }

    tree.set_callback(move |t| {
        if let Some(selected) = t.get_selected_items() {
            // println!("{}",directory_read.to_string() + selected[0].label().unwrap().as_str());
            disp.buffer()
                .unwrap()
                .load_file(directory_read.to_string() + selected[0].label().unwrap().as_str())
                .ok();
        }
    });
    let mut chce = fltk::menu::Choice::new((width*0.90) as i32, (height * 0.05) as i32, (width/15.36) as i32, (height/27.46) as i32, "");
    chce.add_choice("Off|Error|Warn|Info|Debug|Trace");
    chce.set_value(3);
    chce.emit(s,Message::LogLevelChange);
    log_tab.end();
    let log_img = if cfg!(target_os = "windows") {
        JpegImage::load(&std::path::Path::new("assets\\log_bg.jpg"))
    }
    else {
        JpegImage::load(&std::path::Path::new("assets/log_bg.jpg"))
    };
    match log_img {
        Ok(log_img) => log_frame.set_image(Some(log_img)),
        Err(_) => ()
    }
    // Logs tab ends here //
    tabs.emit(s,Message::TabChange);
    tabs.end();

    frames_group.show();

    main_wind.make_resizable(true);
    main_wind.show();
    main_wind.end();

    /* Add new website window starts here */
    let new_web_width = width/1.50;
    let new_web_height = height/1.20;
    let mut window_add_new = Window::default().with_size(new_web_width as i32,new_web_height as i32).with_label("Add New Website Dialog").center_screen();
    window_add_new.begin();
    let mut new_web_frame = Frame::default().size_of(&window_add_new);
    let new_web_img = if cfg!(target_os = "windows") {
        PngImage::load(&std::path::Path::new("assets\\Wave.png"))
    }
    else {
        PngImage::load(&std::path::Path::new("assets/Wave.png"))
    };
    match new_web_img {
        Ok(new_web_img) => new_web_frame.set_image(Some(new_web_img)),
        Err(_) => ()
    }

    let mut class_toggle = ToggleButton::new((new_web_width/2.16) as i32,(new_web_height/5.0) as i32,(new_web_width/15.8) as i32,(new_web_height/22.6) as i32,None).with_label(format!("@+{}circle",(new_web_height/90.44) as i32).as_str());
    let _http = Frame::default().with_size((new_web_width/12.8) as i32,(new_web_height/18.6) as i32).left_of(&class_toggle,5).with_label("HTTP");
    let _https = Frame::default().with_size((new_web_width/12.8) as i32,(new_web_height/18.6) as i32).right_of(&class_toggle,5).with_label("HTTPS");
    let mut access_toggle = ToggleButton::default().with_size((new_web_width/15.8) as i32,(new_web_height/22.6) as i32).below_of(&class_toggle,30).with_label(format!("@+{}circle",(new_web_height/90.44) as i32).as_str());
    let _local = Frame::default().with_size((new_web_width/12.8) as i32,(new_web_height/18.6) as i32).left_of(&access_toggle,5).with_label("Local");
    let _public = Frame::default().with_size((new_web_width/12.8) as i32,(new_web_height/18.6) as i32).right_of(&access_toggle,5).with_label("Public");

    access_toggle.set_align(enums::Align::Inside|enums::Align::Left);
    access_toggle.set_frame(enums::FrameType::RFlatBox);
    access_toggle.set_label_color(enums::Color::White);
    access_toggle.set_color(enums::Color::from_u32(0x878787));
    access_toggle.set_selection_color(enums::Color::from_u32(0x147efb));
    access_toggle.clear_visible_focus();
    

    class_toggle.set_align(enums::Align::Inside|enums::Align::Left);
    class_toggle.set_frame(enums::FrameType::RFlatBox);
    class_toggle.set_label_color(enums::Color::White);
    class_toggle.set_color(enums::Color::from_u32(0x878787));
    class_toggle.set_selection_color(enums::Color::from_u32(0x147efb));
    class_toggle.clear_visible_focus();


    let _frm=Frame::new((new_web_width*0.35) as i32,(new_web_height*0.01) as i32,(new_web_width/3.41) as i32,(new_web_height/3.96) as i32,"Provide Necessary Details");
    let mut browse_webdocs_button = Button::new((new_web_width * 0.40) as i32,(new_web_height * 0.37) as i32,(new_web_width/5.12) as i32,(new_web_height/16.03) as i32,"\u{E807}  Browse Webdocs Folder");
    browse_webdocs_button.set_label_font(enums::Font::by_name(&icons));
    browse_webdocs_button.clear_visible_focus();

    let mut privkey_dialog = FileDialog::new(FileDialogType::BrowseFile);
    let mut browse_key_button= Button::default().with_size((new_web_width/5.12) as i32,(new_web_height/16.03) as i32).right_of(&browse_webdocs_button,30).with_label("\u{E809}  Browse Private Key (.pem)");
    browse_key_button.set_label_font(enums::Font::by_name(&icons));
    browse_key_button.emit(s,Message::BrowseKeys);
    browse_key_button.hide();

    let mut cert_dialog = FileDialog::new(FileDialogType::BrowseFile);
    let mut browse_cert_button = Button::default().with_size((new_web_width/5.12) as i32,(new_web_height/16.03) as i32).left_of(&browse_webdocs_button,30).with_label("\u{E80D}  Browse Certificate (.crt)");
    browse_cert_button.set_label_font(enums::Font::by_name(&icons));
    browse_cert_button.emit(s,Message::BrowseCerts);
    browse_cert_button.hide();

    browse_webdocs_button.set_color(enums::Color::from_u32(0xffdc73));
    browse_key_button.set_color(enums::Color::from_u32(0xffdc73));
    browse_cert_button.set_color(enums::Color::from_u32(0xffdc73));
    

    let mut webdocs_dialog = FileDialog::new(FileDialogType::BrowseDir);
    let add_name_input = Input::new((new_web_width*0.50) as i32,(new_web_height*0.51) as i32,(new_web_width/12.8) as i32,(new_web_height/16.03) as i32,"Enter Name :");
    let port_input = IntInput::default().with_size((new_web_width/12.8) as i32,(new_web_height/16.03) as i32).with_label("Enter Port Number :").below_of(&add_name_input,20);

    let mut push_button = Button::default().with_size((new_web_width/5.12) as i32,(new_web_height/16.03) as i32).with_label("\u{E80B}  Browse Files to be Pushed").below_of(&browse_webdocs_button,(new_web_height*0.29) as i32);
    push_button.set_label_font(enums::Font::by_name(&icons));
    let mut push_dialog = FileDialog::new(FileDialogType::BrowseMultiFile);
    push_button.emit(s,Message::BrowsePushFiles);
    push_button.hide();
    window_add_new.make_resizable(true);
    let mut submit_button = Button::default().with_pos((new_web_width*0.46) as i32,(new_web_height*0.83) as i32).with_size((new_web_width/12.8) as i32,(new_web_height/16.03) as i32).with_label("Submit");
    submit_button.set_color(enums::Color::from_u32(0xffdc73));
    submit_button.emit(s, Message::Write);
    browse_webdocs_button.emit(s,Message::BrowseDocs);

    class_toggle.emit(s,Message::ClassToggle);
    access_toggle.emit(s,Message::AccessToggle);
    
    window_add_new.end();
    /* Add new website window ends here */

    /* Remove Window Starts here */
    let mut window_remove = Window::default().with_size(new_web_width as i32,new_web_height as i32).with_label("Remove Existing Website Dialog").center_screen();
    window_remove.begin();
    let mut rm_web_frame = Frame::default().size_of(&window_remove);
    
    let rm_web_img = if cfg!(target_os = "windows") {
        JpegImage::load(&std::path::Path::new("assets\\remove_bg.jpg"))
    }
    else {
        JpegImage::load(&std::path::Path::new("assets/remove_bg.jpg"))
    };
    match rm_web_img {
        Ok(rm_web_img) => rm_web_frame.set_image(Some(rm_web_img)),
        Err(_) => ()
    }
    let remove_name_input= Input::new((new_web_width*0.47) as i32,(new_web_height*0.20) as i32,(new_web_width/14.72) as i32,(new_web_height/16.03) as i32,"Name : ");
    let mut search_remove = Button::new((new_web_width*0.47) as i32,(new_web_height*0.30) as i32,(new_web_width/14.72) as i32,(new_web_height/16.03) as i32,"Search");
    let mut remove = Button::new((new_web_width*0.47) as i32,(new_web_height*0.80) as i32,(new_web_width/14.72) as i32,(new_web_height/16.03) as i32,"Remove");
    remove.set_frame(enums::FrameType::GleamUpBox);
    remove.set_color(enums::Color::from_u32(0xffdc73));
    search_remove.set_frame(enums::FrameType::GleamUpBox);
    search_remove.set_color(enums::Color::from_u32(0xffdc73));

    let mut rem_name_str = Frame::new((new_web_width*0.37) as i32,(new_web_height*0.45) as i32,(new_web_width/8.53) as i32,(new_web_height/27.46) as i32,"Name : ").with_align(enums::Align::Wrap);
    rem_name_str.set_frame(enums::FrameType::GtkUpFrame);
    rem_name_str.set_color(enums::Color::Magenta);
    let mut rem_class_str = Frame::default().with_size((new_web_width/8.53) as i32,(new_web_height/27.46) as i32).below_of(&rem_name_str,10).with_label("Class : ").with_align(enums::Align::Wrap);
    rem_class_str.set_frame(enums::FrameType::GtkUpFrame);
    rem_class_str.set_color(enums::Color::Magenta);
    let mut rem_web_resource_str = Frame::default().with_size((new_web_width/8.53) as i32,(new_web_height/27.46) as i32).below_of(&rem_class_str,10).with_label("Resource Path : ").with_align(enums::Align::Wrap);
    rem_web_resource_str.set_frame(enums::FrameType::GtkUpFrame);
    rem_web_resource_str.set_color(enums::Color::Magenta);
    let mut rem_port_str = Frame::default().with_size((new_web_width/8.53) as i32,(new_web_height/27.46) as i32).below_of(&rem_web_resource_str,10).with_label("Port No : ").with_align(enums::Align::Wrap);
    rem_port_str.set_frame(enums::FrameType::GtkUpFrame);
    rem_port_str.set_color(enums::Color::Magenta);
    let mut rem_cert_str = Frame::default().with_size((new_web_width/8.53) as i32,(new_web_height/27.46) as i32).below_of(&rem_port_str,10).with_label("Certificate : ").with_align(enums::Align::Wrap);
    rem_cert_str.set_frame(enums::FrameType::GtkUpFrame);
    rem_cert_str.set_color(enums::Color::Magenta);
    let mut rem_key_str = Frame::default().with_size((new_web_width/8.53) as i32,(new_web_height/27.46) as i32).below_of(&rem_cert_str,10).with_label("Private Key : ").with_align(enums::Align::Wrap);
    rem_key_str.set_frame(enums::FrameType::GtkUpFrame);
    rem_key_str.set_color(enums::Color::Magenta);

    let mut rem_name = Frame::default().with_size((new_web_width/5.12) as i32,(new_web_height/27.46) as i32).with_pos((new_web_width*0.48) as i32,(new_web_height*0.45) as i32).with_align(enums::Align::Wrap);
    rem_name.set_frame(enums::FrameType::GtkUpFrame);
    rem_name.set_color(enums::Color::Magenta);
    let mut rem_class = Frame::default().with_size((new_web_width/5.12) as i32,(new_web_height/27.46) as i32).below_of(&rem_name,10).with_align(enums::Align::Wrap);
    rem_class.set_frame(enums::FrameType::GtkUpFrame);
    rem_class.set_color(enums::Color::Magenta);
    let mut rem_web_resource = Frame::default().with_size((new_web_width/5.12) as i32,(new_web_height/27.46) as i32).below_of(&rem_class,10).with_align(enums::Align::Wrap);
    rem_web_resource.set_frame(enums::FrameType::GtkUpFrame);
    rem_web_resource.set_color(enums::Color::Magenta);
    let mut rem_port = Frame::default().with_size((new_web_width/5.12) as i32,(new_web_height/27.46) as i32).below_of(&rem_web_resource,10).with_align(enums::Align::Wrap);
    rem_port.set_frame(enums::FrameType::GtkUpFrame);
    rem_port.set_color(enums::Color::Magenta);
    let mut rem_cert = Frame::default().with_size((new_web_width/5.12) as i32,(new_web_height/27.46) as i32).below_of(&rem_port,10).with_align(enums::Align::Wrap);
    rem_cert.set_frame(enums::FrameType::GtkUpFrame);
    rem_cert.set_color(enums::Color::Magenta);
    let mut rem_key = Frame::default().with_size((new_web_width/5.12) as i32,(new_web_height/27.46) as i32).below_of(&rem_cert,10).with_align(enums::Align::Wrap);
    rem_key.set_frame(enums::FrameType::GtkUpFrame);
    rem_key.set_color(enums::Color::Magenta);
    rem_name_str.hide();
    rem_class_str.hide();
    rem_web_resource_str.hide();
    rem_port_str.hide();
    rem_cert_str.hide();
    rem_key_str.hide();
    rem_name.hide();
    rem_class.hide();
    rem_web_resource.hide();
    rem_port.hide();
    rem_cert.hide();
    rem_key.hide();
    remove.hide();
    search_remove.emit(s,Message::Search);
    remove.emit(s,Message::Remove);
    window_remove.make_resizable(true);
    window_remove.end();
    /* Remove Window Ends here */

    let con=read_conf();
    if !con.is_empty() {
        print_headers(&mut frames_group);
        print_get_entries(con,&mut frames_group);
    }
    let start_ts = SystemTime::now(); // Start time, from here

    while main_app.wait() {
        match r.recv() {
            Some(val) => match val {
                Message::TabChange => {
                    if tabs.value().unwrap().label() == statistics_tab.label() {
                        frames_group.set_changed();
                    }
                }
                Message::AddNewWebsite => {
                    window_add_new.make_modal(true);
                    window_add_new.show();
                }
                Message::RemoveWebsite => {
                    window_remove.make_modal(true);
                    window_remove.show();
                }
                Message::Remove => {
                    let con=read_conf();
                    let config: HashMap<String, Vec<Website>> = from_str(&con).unwrap();
                    let websites: &[Website] = &config["websites"];
                    let mut v=websites.to_vec();
                    let mut index = 0;
                    for website in v.iter() {
                        if website.name.to_string()==remove_name_input.value() {
                            break;
                        }
                        index += 1;
                    }
                    v.remove(index);
                    let mut conf_file = if cfg!(target_os = "windows") {
                        std::fs::remove_file("C:\\Program Files\\Common Files\\Lightron\\lightron.conf").unwrap();
                        OpenOptions::new().write(true).create(true).append(true).open("C:\\Program Files\\Common Files\\Lightron\\lightron.conf").unwrap()
                    }
                    else {
                        std::fs::remove_file("/etc/lightron.conf").unwrap();
                        OpenOptions::new().write(true).create(true).append(true).open("/etc/lightron.conf").unwrap()
                    };
                    for website in v.iter() {
                        let toml = "\n\n[[websites]]\n".to_string()+ &to_string(&website).unwrap();
                        conf_file.write(toml.as_bytes()).expect("Error writing file.");
                    }
                    frames_group.clear();
                    print_headers(&mut frames_group);
                    let conf_string = read_conf();
                    if !conf_string.is_empty() {
                        print_get_entries(conf_string,&mut frames_group);
                    }
                    remove_name_input.set_value("");
                    rem_name.set_label("");
                    rem_class.set_label("");
                    rem_web_resource.set_label("");
                    rem_cert.set_label("");
                    rem_key.set_label("");
                    rem_port.set_label("");
                    window_remove.hide();
                    rem_name_str.hide();
                    rem_class_str.hide();
                    rem_web_resource_str.hide();
                    rem_port_str.hide();
                    rem_cert_str.hide();
                    rem_key_str.hide();
                    rem_name.hide();
                    rem_class.hide();
                    rem_web_resource.hide();
                    rem_port.hide();
                    rem_cert.hide();
                    rem_key.hide();
                    remove.hide();
                    main_app.redraw();
                }
                Message::Search => {
                    rem_cert.set_label("");
                    rem_key.set_label("");
                    rem_cert_str.hide();
                    rem_key_str.hide();
                    rem_cert.hide();
                    rem_key.hide();
                    rem_name.set_label("");
                    rem_class.set_label("");
                    rem_web_resource.set_label("");
                    rem_port.set_label("");
                    rem_name_str.hide();
                    rem_class_str.hide();
                    rem_web_resource_str.hide();
                    rem_port_str.hide();
                    rem_name.hide();
                    rem_class.hide();
                    rem_web_resource.hide();
                    rem_port.hide();
                    remove.hide();
                    let con=read_conf();
                    let config: HashMap<String, Vec<Website>> = from_str(&con).unwrap();
                    let websites: &[Website] = &config["websites"];
                    let v=websites.to_vec();
                    let mut index = 0;
                    for website in v.iter() {
                        if website.name.to_string()==remove_name_input.value() {
                            rem_name.set_label(&website.name);
                            rem_port.set_label(&website.port_no.to_string());
                            rem_class.set_label(&website.class);
                            rem_web_resource.set_label(&website.resource.to_str().unwrap_or(""));
                            if &website.class == "HTTPS" {
                                rem_cert.set_label(&website.certificate.to_str().unwrap_or(""));
                                rem_key.set_label(&website.private_key.to_str().unwrap_or(""));
                                rem_cert_str.show();
                                rem_key_str.show();
                                rem_cert.show();
                                rem_key.show();
                            }
                            rem_name_str.show();
                            rem_class_str.show();
                            rem_web_resource_str.show();
                            rem_port_str.show();
                            rem_name.show();
                            rem_class.show();
                            rem_web_resource.show();
                            rem_port.show();
                            break;
                        }
                        index += 1;
                    }
                    if index==v.len() {
                        let _error_msg = message(300, 300, "No Record Found !");
                        remove_name_input.set_value("");
                        rem_cert.set_label("");
                        rem_key.set_label("");
                        rem_cert_str.hide();
                        rem_key_str.hide();
                        rem_cert.hide();
                        rem_key.hide();
                        rem_name.set_label("");
                        rem_class.set_label("");
                        rem_web_resource.set_label("");
                        rem_port.set_label("");
                        rem_name_str.hide();
                        rem_class_str.hide();
                        rem_web_resource_str.hide();
                        rem_port_str.hide();
                        rem_name.hide();
                        rem_class.hide();
                        rem_web_resource.hide();
                        rem_port.hide();
                        remove.hide();
                    }
                    else {
                        remove.show();
                    }
                }
                Message::BrowseDocs => webdocs_dialog.show(),
                Message::BrowseCerts => cert_dialog.show(),
                Message::BrowseKeys => privkey_dialog.show(),
                Message::BrowsePushFiles => push_dialog.show(),
                Message::ClassToggle => {
                    if class_toggle.is_set() {
                        class_toggle.set_align(enums::Align::Inside | enums::Align::Right);
                        push_button.show();
                        browse_cert_button.show();
                        browse_key_button.show();
                    }
                    else {
                        class_toggle.set_align(enums::Align::Inside | enums::Align::Left);
                        push_button.hide();
                        browse_cert_button.hide();
                        browse_key_button.hide();
                    }
                    window_add_new.redraw();
                }
                Message::AccessToggle => {
                    if access_toggle.is_set() {
                        access_toggle.set_align(enums::Align::Inside | enums::Align::Right);
                    }
                    else {
                        access_toggle.set_align(enums::Align::Inside | enums::Align::Left);
                    }
                    window_add_new.redraw();
                }
                Message::Write => {
                    let config = Website {
                        name : add_name_input.value(),
                        class : if class_toggle.is_toggled() {
                            "HTTPS".to_string()
                        }
                        else {
                            "HTTP".to_string()
                        },
                        access: if access_toggle.is_toggled() {
                            "Public".to_string()
                        }
                        else {
                            "Local".to_string()
                        },
                        resource: webdocs_dialog.filename(),
                        certificate: cert_dialog.filename(), 
                        private_key: privkey_dialog.filename(),
                        port_no : port_input.value().parse::<u16>().unwrap(),
                        push_protocol_files : if class_toggle.is_toggled() {
                            push_dialog.filenames().iter().map(|x| x.strip_prefix(webdocs_dialog.filename()).unwrap().to_path_buf()).collect()
                        }
                        else {
                            // "".to_string()
                            Vec::new()
                        },
                        log_level : "Info".to_string(),

                    };
                    let toml = "[[websites]]\n".to_string()+ &to_string(&config).unwrap();
                    let mut conf_file = if cfg!(target_os = "windows") {
                        OpenOptions::new().write(true).create(true).append(true).open("C:\\Program Files\\Common Files\\lightron\\Lightron.conf").unwrap()
                    }
                    else {
                        OpenOptions::new().write(true).create(true).append(true).open("/etc/lightron.conf").unwrap()
                    };
                    conf_file.write(toml.as_bytes()).expect("Error writing file.");
                    frames_group.clear();
                    print_headers(&mut frames_group);
                    let conf_string = read_conf();
                    if !conf_string.is_empty() {
                        print_get_entries(conf_string,&mut frames_group);
                    }
                    port_input.set_value("");
                    add_name_input.set_value("");
                    //push_input.set_value("");
                    class_toggle.set_align(enums::Align::Inside | enums::Align::Left);
                    class_toggle.toggle(false);
                    access_toggle.set_align(enums::Align::Inside | enums::Align::Left);
                    access_toggle.toggle(false);
                    browse_cert_button.hide();
                    browse_key_button.hide();
                    window_add_new.hide();
                    main_app.redraw();
                }
                Message::LogLevelChange => {
                    let con=read_conf();
                    let config: HashMap<String, Vec<Website>> = from_str(&con).unwrap();
                    let websites: &[Website] = &config["websites"];
                    let mut v=websites.to_vec();
                    for website in v.iter_mut() {
                        website.log_level = chce.text(chce.value()).unwrap();
                    }
                    let mut conf_file = if cfg!(target_os = "windows") {
                        std::fs::remove_file("C:\\Program Files\\Common Files\\Lightron\\lightron.conf").unwrap();
                        OpenOptions::new().write(true).create(true).append(true).open("C:\\Program Files\\Common Files\\Lightron\\lightron.conf").unwrap()
                    }
                    else {
                        std::fs::remove_file("/etc/lightron.conf").unwrap();
                        OpenOptions::new().write(true).create(true).append(true).open("/etc/lightron.conf").unwrap()
                    };
                    for website in v.iter() {
                        let toml = "[[websites]]\n".to_string()+ &to_string(&website).unwrap();
                        conf_file.write(toml.as_bytes()).expect("Error writing file.");
                    }
                }
            }
            None=>()
        }
        match rx.try_recv() {
            Ok((load,mem_used)) => {
                if (tabs.value().unwrap().label() == main_tab.label()) && (frames_group.changed()) {
                    frames_group.clear();
                    let conf_string = read_conf();
                    if !conf_string.is_empty() {
                        print_headers(&mut frames_group);
                        print_get_entries(conf_string,&mut frames_group);
                    }
                    frames_group.clear_changed();
                }
                
                //println!("received {},{}",load,mem_used);
                cpu_dial.set_value(load as i32); // Dial set
                // CPU GRAPH //
                cpu_data.push((SystemTime::now().duration_since(start_ts).unwrap().as_secs_f64(),load)); // duration since time start and respective load
                let cpu_root = BitMapBackend::<RGBPixel>::with_buffer_and_format(
                    &mut cpu_buf,
                    ((width/2.3) as i32 as u32, (height/2.2) as i32 as u32),
                ).unwrap()
                .into_drawing_area();
                let mut chart = cpu_cs.clone().restore(&cpu_root);
                chart.plotting_area().fill(&WHITE).unwrap();
                chart.configure_mesh().bold_line_style(&BLACK.mix(0.2)).light_line_style(&TRANSPARENT).draw().unwrap();    
                chart.draw_series(cpu_data.iter().zip(cpu_data.iter().skip(1)).map(
                    |(&(x0, y0), &(x1, y1))| {
                        PathElement::new(
                            vec![(x0, y0), (x1, y1)],
                            &BLUE,
                        )
                    },
                )).unwrap();
                drop(cpu_root);
                drop(chart);
                draw::draw_rgb(&mut cpu_frame, &cpu_buf).unwrap();
                cpu_win.redraw();
                if SystemTime::now().duration_since(start_ts).unwrap().as_secs_f64() >= 60.00 {
                    let (x,_) = cpu_data.remove(0);
                    for (j,_) in cpu_data.iter_mut() {
                        *j -= x;
                    }
                }
                // CPU GRAPH END //

                // MEMORY GRAPH //
                mem_data.push((SystemTime::now().duration_since(start_ts).unwrap().as_secs_f64(),mem_used)); // duration since time start and respective load
                let mem_root = BitMapBackend::<RGBPixel>::with_buffer_and_format(
                    &mut mem_buf,
                    ((width/2.3) as i32 as u32, (height/2.2) as i32 as u32),
                ).unwrap()
                .into_drawing_area();
                let mut mem_chart = mem_cs.clone().restore(&mem_root);
                mem_chart.plotting_area().fill(&WHITE).unwrap();
                mem_chart.configure_mesh().bold_line_style(&BLACK.mix(0.2)).light_line_style(&TRANSPARENT).draw().unwrap();    
                mem_chart.draw_series(mem_data.iter().zip(mem_data.iter().skip(1)).map(
                    |(&(x0, y0), &(x1, y1))| {
                        PathElement::new(
                            vec![(x0, y0), (x1, y1)],
                            &BLUE,
                        )
                    },
                )).unwrap();
                drop(mem_root);
                drop(mem_chart);
                draw::draw_rgb(&mut mem_frame, &mem_buf).unwrap();
                mem_win.redraw();
                if SystemTime::now().duration_since(start_ts).unwrap().as_secs_f64() >= 60.00 {
                    let (x,_) = mem_data.remove(0);
                    for (j,_) in mem_data.iter_mut() {
                        *j -= x;
                    }
                }
                // MEMORY GRAPH END //
            }
            _ => {},
        }
    }
    Ok(())
}