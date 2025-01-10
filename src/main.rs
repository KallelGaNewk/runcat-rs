#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::Path,
    time::{Duration, Instant},
};

use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
};

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    let event_loop = EventLoopBuilder::new().build();
    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append_items(&[&quit_i]);

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu.clone()))
        .with_menu_on_left_click(false)
        .with_tooltip("RunCat in Rust")
        .build()
        .unwrap();

    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
    );

    let dark_cats = [
        load_icon(&path.join("dark_cat_0.ico")),
        load_icon(&path.join("dark_cat_1.ico")),
        load_icon(&path.join("dark_cat_2.ico")),
        load_icon(&path.join("dark_cat_3.ico")),
        load_icon(&path.join("dark_cat_4.ico")),
    ];

    let light_cats = [
        load_icon(&path.join("light_cat_0.ico")),
        load_icon(&path.join("light_cat_1.ico")),
        load_icon(&path.join("light_cat_2.ico")),
        load_icon(&path.join("light_cat_3.ico")),
        load_icon(&path.join("light_cat_4.ico")),
    ];

    let mut current = 0;
    let mut update_time = Instant::now();
    let mut interval = 200.0;
    let mut cpu_time = Instant::now();
    let mut is_dark = true;
    event_loop.run(move |_, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16));

        if update_time.elapsed().as_millis() > interval as u128 {
            if dark_cats.len() <= current {
                current = 0;
            }

            if is_dark {
                let _ = tray_icon.set_icon(Some(dark_cats[current].clone()));
            } else {
                let _ = tray_icon.set_icon(Some(light_cats[current].clone()));
            }

            current += 1;

            let cpu_usage = sys.global_cpu_usage();
            let _ = tray_icon.set_tooltip(Some(format!("CPU: {cpu_usage:.2}%")));

            interval = 200.0 / 1.0_f32.max(20.0_f32.min(cpu_usage / 5.0));
            update_time = Instant::now();
        }

        if cpu_time.elapsed() > Duration::from_millis(3000) {
            sys.refresh_cpu_usage();
            cpu_time = Instant::now();
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Ok(event) = tray_channel.try_recv() {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if let MouseButton::Left = button {
                    if let MouseButtonState::Down = button_state {
                        is_dark = !is_dark;
                    }
                }
            }
        }
    })
}

fn load_icon(path: &Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
