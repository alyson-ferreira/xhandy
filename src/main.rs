/*
  MIT License

  Copyright (c) 2023 Alyson Tiago S. Ferreira

  Permission is hereby granted, free of charge, to any person obtaining a copy
  of this software and associated documentation files (the "Software"), to deal
  in the Software without restriction, including without limitation the rights
  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  copies of the Software, and to permit persons to whom the Software is
  furnished to do so, subject to the following conditions:

  The above copyright notice and this permission notice shall be included in all
  copies or substantial portions of the Software.

  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  SOFTWARE.
*/

use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::CStr;
use std::ptr;
use std::slice::from_raw_parts;
use x11_dl::xlib;
use x11_dl::xrandr;

#[derive(Debug)]
struct Controller {
    mode: u64,
    pos: (i32, i32),
}

#[derive(Debug)]
struct Mode {
    name: String,
}

#[derive(Debug)]
struct Output {
    name: String,
    connected: bool,
    modes: Vec<u64>,
    controller: Option<u64>,
}

fn get_display_screen_window(xlib: &xlib::Xlib) -> (*mut xlib::_XDisplay, i32, u64) {
    unsafe {
        let display = (xlib.XOpenDisplay)(ptr::null());

        if display.is_null() {
            panic!("XOpenDisplay failed");
        }

        let screen = (xlib.XDefaultScreen)(display);
        let window = (xlib.XRootWindow)(display, screen);

        (display, screen, window)
    }
}

fn get_outputs(
    xrandr: &xrandr::Xrandr,
    display: *mut xlib::_XDisplay,
    screen_resources: *mut xrandr::XRRScreenResources,
) -> HashMap<u64, Output> {
    let mut outputs: HashMap<u64, Output> = HashMap::new();

    unsafe {
        let outputs_id = from_raw_parts(
            (*screen_resources).outputs,
            (*screen_resources).noutput as usize,
        );

        for output_id in outputs_id.iter() {
            let output_info = (xrandr.XRRGetOutputInfo)(display, screen_resources, *output_id);

            if output_info.is_null() {
                panic!("XRRGetOutputInfo failed")
            }

            let name = CStr::from_ptr((*output_info).name)
                .to_str()
                .to_owned()
                .unwrap()
                .to_string();
            let connected = (*output_info).connection == 0;
            let modes_id = from_raw_parts((*output_info).modes, (*output_info).nmode as usize);
            let modes = modes_id.to_vec();
            let controller = if (*output_info).crtc > 0 {
                Some((*output_info).crtc)
            } else {
                None
            };
            let output = Output {
                name: name.to_owned(),
                connected,
                modes,
                controller,
            };

            outputs.insert(*output_id, output);
        }
    }

    outputs
}

fn get_controllers(
    xrandr: &xrandr::Xrandr,
    display: *mut xlib::_XDisplay,
    screen_resources: *mut xrandr::XRRScreenResources,
) -> HashMap<u64, Controller> {
    let mut controllers: HashMap<u64, Controller> = HashMap::new();

    unsafe {
        let controllers_id = from_raw_parts(
            (*screen_resources).crtcs,
            (*screen_resources).ncrtc as usize,
        );

        for controller_id in controllers_id.iter() {
            let controller_info =
                (xrandr.XRRGetCrtcInfo)(display, screen_resources, *controller_id);
            let mode = (*controller_info).mode;
            let pos = ((*controller_info).x, (*controller_info).y);
            let controller = Controller { mode, pos };

            controllers.insert(*controller_id, controller);
        }
    }

    controllers
}

fn get_modes(screen_resources: *mut xrandr::XRRScreenResources) -> HashMap<u64, Mode> {
    let mut modes: HashMap<u64, Mode> = HashMap::new();

    unsafe {
        let modes_info = from_raw_parts(
            (*screen_resources).modes,
            (*screen_resources).nmode as usize,
        );

        for mode_info in modes_info.iter() {
            let name = CStr::from_ptr(mode_info.name)
                .to_str()
                .to_owned()
                .unwrap()
                .to_string();
            let id = mode_info.id;
            let mode = Mode { name };
            modes.insert(id, mode);
        }
    }

    modes
}

fn present(
    outputs: &HashMap<u64, Output>,
    modes: &HashMap<u64, Mode>,
    controllers: &HashMap<u64, Controller>,
) {
    for (_, output) in outputs {
        if !output.connected {
            continue;
        }

        let mut controller: Option<&Controller> = None;

        if let Some(controller_id) = output.controller {
            controller = Some(&controllers[&controller_id]);
        }

        let displaying = if let Some(ctrl) = controller {
            let mode_name = &modes[&ctrl.mode].name;
            let (x, y) = ctrl.pos;
            format!("{} ({}, {})", mode_name, x, y)
        } else {
            "-".to_string()
        };

        let mut all_modes = String::new();
        let mut modes_set = HashSet::<String>::new();

        for mode_id in &output.modes {
            let name = &modes[&mode_id].name;

            if modes_set.contains(name.as_str()) {
                continue;
            }

            modes_set.insert(name.to_owned());

            let display_part = if all_modes.is_empty() {
                name.to_owned()
            } else {
                (" ".to_string() + name.as_str()).to_owned()
            };
            all_modes.push_str(display_part.as_str());
        }

        let result = format!(
            "{}:\n  displaying: {}\n  modes: {}\n",
            output.name, displaying, all_modes
        );
        print!("{}", result);
    }
}

fn main() {
    let xlib = xlib::Xlib::open().unwrap();
    let xrandr = xrandr::Xrandr::open().unwrap();
    let (display, _, window) = get_display_screen_window(&xlib);
    let screen_resources = unsafe { (xrandr.XRRGetScreenResources)(display, window) };

    let outputs = get_outputs(&xrandr, display, screen_resources);
    let modes = get_modes(screen_resources);
    let controllers = get_controllers(&xrandr, display, screen_resources);

    present(&outputs, &modes, &controllers);
}
