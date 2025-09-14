mod alloc;
mod tic80;
mod land;
mod hud;

use tic80::*;
use std::cell::{Cell, RefCell};
use land::{Land, LandTexture};

struct Camera {
    x: i32,
    y: i32
}

const LAND_TEXTURE: LandTexture = LandTexture{spr_id: 1, width: 2, height: 2};
thread_local! {
    static LAND_SEED: Cell<u32> = Cell::new(0);
    static LAND: RefCell<Land> = RefCell::new(Land::from_map_or_new(45, 24, LAND_TEXTURE));
    static CAMERA: RefCell<Camera> = RefCell::new(Camera {x: 0, y: 0});
    static NOTIFIER: RefCell<hud::Notifier> = RefCell::new(hud::Notifier::default());
}

const NOTIFY_TIME: i32 = 5*60;
fn notify(msg: &str) {
    NOTIFIER.with_borrow_mut(|note| note.notify(msg, NOTIFY_TIME));
}

#[export_name = "BOOT"]
pub fn boot() {
    LAND.with_borrow_mut(|land| {
        if land.seed() == 0 {
            land.set_seed(tstamp());
            land.generate();
            land.save_in_map();
            notify("Land generated");
        } else {
            notify("Land loaded");
        }
        LAND_SEED.set(land.seed());
    });
}

#[export_name = "TIC"]
pub fn tic() {
    cls(13);
    // Button press actions
    if btn(4) {
        // Button A: save land to MAP
        LAND.with_borrow(|land| land.save_in_map());
        unsafe {
            sync(4, 0, true);
        }
        notify("Land saved");
    } else if btn(5) {
        // Button B: clear data in MAP
        mset(0, 0, 0);
        mset(1, 0, 0);
        unsafe {
            sync(4, 0, true);
        }
        notify("Land cleared from the save");
    } else if btn(6) {
        // Button X: generate new land with a different seed
        LAND.with_borrow_mut(|land| {
            land.set_seed(tstamp());
            land.generate();
            LAND_SEED.set(land.seed());
        });
        notify("Land generated");
    } else if btn(7) {
        // Button Y: generate new land with the same seed (reset)
        LAND.with_borrow(|land| land.generate());
        notify("Land reset");
    }

    // Mouse manipuations & land rendering
    let mouse_input = mouse();
    let (mx, my) = (mouse_input.x as i32, mouse_input.y as i32);
    let radius = 8;
    CAMERA.with_borrow_mut(|cam| {
        LAND.with_borrow(|land| {
            // Move camera
            const CAMERA_MOVE_BORDER: i32 = 5;
            const CAMERA_ADD_MARGIN: (i32, i32, i32, i32) = (75, 75, 50, 25);
            let (land_w, land_h) = land.size();
            if mx < CAMERA_MOVE_BORDER && cam.x > -CAMERA_ADD_MARGIN.0 {
                cam.x -= 1;
            } else if mx > 240 - CAMERA_MOVE_BORDER && cam.x + 240 < land_w + CAMERA_ADD_MARGIN.1 {
                cam.x += 1;
            }
            if my < CAMERA_MOVE_BORDER && cam.y > -CAMERA_ADD_MARGIN.2 {
                cam.y -= 1;
            } else if my > 137 - CAMERA_MOVE_BORDER && cam.y + 137 < land_h + CAMERA_ADD_MARGIN.3 {
                cam.y += 1;
            }
            // Update & render LAND
            if mouse_input.left || mouse_input.right {
                let (x, y) = (mx + cam.x, my + cam.y);
                let state = mouse_input.right;
                land.set_circle(x, y, radius, state);
            }
            land.draw(-cam.x, -cam.y, 1);
        });
    });

    // HUD drawing
    circb(mx, my, radius, 2); // Mouse manipulation circle
    let stats = format!("Seed: {}\nRadius: {}", LAND_SEED.get(), radius);
    print!(stats, 0, 6, PrintOptions::default());
    NOTIFIER.with_borrow_mut(|note| {
        note.countdown();
        note.draw();
    });
}
