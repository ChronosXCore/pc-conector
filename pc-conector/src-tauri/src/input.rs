use crate::network::{KeyboardData, KeyboardEventType, MouseData, MouseEventType};
use enigo::{
    Coordinate, Direction, Enigo, Key, Keyboard as EnigoKeyboard, Mouse as EnigoMouse, Settings,
};
use log::{info, error};
use rdev::{grab, Event, EventType};
use std::sync::{Arc, Mutex};
use std::thread;

/// Callback para cuando se recibe un evento de entrada
pub type InputCallback = Arc<dyn Fn(InputEvent) + Send + Sync>;

#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseMove { x: f64, y: f64 },
    MousePress { button: u8 },
    MouseRelease { button: u8 },
    MouseScroll { delta_x: f64, delta_y: f64 },
    KeyPress { key: u32, char: Option<String> },
    KeyRelease { key: u32, char: Option<String> },
}

/// Servicio de captura y simulación de entrada (teclado/mouse)
pub struct InputService {
    enigo: Arc<Mutex<Enigo>>,
    is_capturing: Arc<Mutex<bool>>,
    on_input: Option<InputCallback>,
    #[allow(dead_code)]
    mouse_locked: Arc<Mutex<bool>>,
    pub forwarding_active: Arc<std::sync::atomic::AtomicBool>,
}

impl InputService {
    pub fn new() -> Self {
        let enigo = Enigo::new(&Settings::default()).unwrap_or_else(|_| panic!("Error al inicializar Enigo"));
        
        Self {
            enigo: Arc::new(Mutex::new(enigo)),
            is_capturing: Arc::new(Mutex::new(false)),
            on_input: None,
            mouse_locked: Arc::new(Mutex::new(false)),
            forwarding_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn set_forwarding_active(&self, active: bool) {
        self.forwarding_active.store(active, std::sync::atomic::Ordering::Relaxed);
        unsafe {
            set_system_cursors_hidden(active);
        }
    }

    /// Establecer callback para eventos de entrada capturados
    pub fn set_on_input<F>(&mut self, callback: F)
    where
        F: Fn(InputEvent) + Send + Sync + 'static,
    {
        self.on_input = Some(Arc::new(callback));
    }

    /// Iniciar captura global de teclado y mouse
    pub fn start_capture(&self) -> Result<(), String> {
        let is_capturing = self.is_capturing.clone();
        *is_capturing.lock().unwrap() = true;

        let callback = self.on_input.clone();
        let capturing = self.is_capturing.clone();
        let forwarding = self.forwarding_active.clone();

        thread::spawn(move || {
            let callback = callback;
            let capturing = capturing;
            let forwarding = forwarding;

            if let Err(e) = grab(move |event: Event| {
                if !*capturing.lock().unwrap() {
                    return Some(event);
                }

                let active = forwarding.load(std::sync::atomic::Ordering::Relaxed);

                if let Some(ref cb) = callback {
                    let input_event = match event.event_type {
                        EventType::MouseMove { x, y } => {
                            Some(InputEvent::MouseMove { x, y })
                        }
                        EventType::ButtonPress(button) => {
                            Some(InputEvent::MousePress { button: rdev_button_to_u8(button) })
                        }
                        EventType::ButtonRelease(button) => {
                            Some(InputEvent::MouseRelease { button: rdev_button_to_u8(button) })
                        }
                        EventType::Wheel { delta_x, delta_y } => {
                            Some(InputEvent::MouseScroll { delta_x: delta_x as f64, delta_y: delta_y as f64 })
                        }
                        EventType::KeyPress(key) => {
                            Some(InputEvent::KeyPress {
                                key: 0,
                                char: Some(map_rdev_key_to_string(key)),
                            })
                        }
                        EventType::KeyRelease(key) => {
                            Some(InputEvent::KeyRelease {
                                key: 0,
                                char: Some(map_rdev_key_to_string(key)),
                            })
                        }
                    };

                    if let Some(ev) = input_event {
                        cb(ev);
                    }
                }

                if active {
                    match event.event_type {
                        EventType::MouseMove { .. } => Some(event),
                        _ => None,
                    }
                } else {
                    Some(event)
                }
            }) {
                error!("Error en grab de entrada: {:?}", e);
            }
        });

        info!("Captura de entrada iniciada");
        Ok(())
    }

    /// Simular un evento de mouse en el sistema local
    pub fn simulate_mouse(&self, data: &MouseData) -> Result<(), String> {
        let mut enigo = self.enigo.lock().unwrap();

        match data.event_type {
            MouseEventType::Move => {
                enigo.move_mouse(data.x as i32, data.y as i32, Coordinate::Abs)
                    .map_err(|e| format!("Error al mover mouse: {:?}", e))?;
            }
            MouseEventType::Press => {
                if let Some(button) = data.button {
                    let btn = enigo_mouse_button(button);
                    enigo.button(btn, Direction::Press)
                        .map_err(|e| format!("Error al hacer press: {:?}", e))?;
                }
            }
            MouseEventType::Release => {
                if let Some(button) = data.button {
                    let btn = enigo_mouse_button(button);
                    enigo.button(btn, Direction::Release)
                        .map_err(|e| format!("Error al soltar botón: {:?}", e))?;
                }
            }
            MouseEventType::Scroll => {
                if let Some((dx, dy)) = data.scroll_delta {
                    if dx != 0.0 {
                        enigo.scroll(dx as i32, enigo::Axis::Horizontal)
                            .map_err(|e| format!("Error al hacer scroll horizontal: {:?}", e))?;
                    }
                    if dy != 0.0 {
                        enigo.scroll(dy as i32, enigo::Axis::Vertical)
                            .map_err(|e| format!("Error al hacer scroll vertical: {:?}", e))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Simular un evento de teclado en el sistema local
    pub fn simulate_keyboard(&self, data: &KeyboardData) -> Result<(), String> {
        let mut enigo = self.enigo.lock().unwrap();
        let direction = match data.event_type {
            KeyboardEventType::Press => Direction::Press,
            KeyboardEventType::Release => Direction::Release,
        };

        if let Some(ref ch) = data.key_char {
            if ch.starts_with('[') && ch.ends_with(']') {
                // Special key
                if let Some(key) = parse_special_key(ch) {
                    enigo.key(key, direction)
                        .map_err(|e| format!("Error al simular tecla especial: {:?}", e))?;
                }
            } else if ch.len() == 1 {
                // Unicode character
                let key = Key::Unicode(ch.chars().next().unwrap());
                enigo.key(key, direction)
                    .map_err(|e| format!("Error al simular tecla: {:?}", e))?;
            }
        }

        Ok(())
    }

    /// Warp mouse to absolute position (used to snap cursor back when entering remote zone)
    pub fn warp_mouse(&self, x: i32, y: i32) -> Result<(), String> {
        let mut enigo = self.enigo.lock().unwrap();
        use enigo::{Coordinate, Mouse as EnigoMouse};
        enigo.move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| format!("Error al teletransportar mouse: {:?}", e))
    }

    /// Detener captura
    pub fn stop_capture(&self) {
        *self.is_capturing.lock().unwrap() = false;
        info!("Captura de entrada detenida");
    }
}


fn rdev_button_to_u8(button: rdev::Button) -> u8 {
    match button {
        rdev::Button::Left => 1,
        rdev::Button::Middle => 2,
        rdev::Button::Right => 3,
        rdev::Button::Unknown(code) => code,
    }
}

fn enigo_mouse_button(button: u8) -> enigo::Button {
    match button {
        1 => enigo::Button::Left,
        2 => enigo::Button::Middle,
        3 => enigo::Button::Right,
        _ => enigo::Button::Left,
    }
}

fn map_rdev_key_to_string(key: rdev::Key) -> String {
    match key {
        rdev::Key::ControlLeft | rdev::Key::ControlRight => "[Control]".to_string(),
        rdev::Key::ShiftLeft | rdev::Key::ShiftRight => "[Shift]".to_string(),
        rdev::Key::Alt | rdev::Key::AltGr => "[Alt]".to_string(),
        rdev::Key::MetaLeft | rdev::Key::MetaRight => "[Meta]".to_string(),
        rdev::Key::Return => "[Return]".to_string(),
        rdev::Key::Escape => "[Escape]".to_string(),
        rdev::Key::Backspace => "[Backspace]".to_string(),
        rdev::Key::Tab => "[Tab]".to_string(),
        rdev::Key::Space => " ".to_string(),
        rdev::Key::UpArrow => "[UpArrow]".to_string(),
        rdev::Key::DownArrow => "[DownArrow]".to_string(),
        rdev::Key::LeftArrow => "[LeftArrow]".to_string(),
        rdev::Key::RightArrow => "[RightArrow]".to_string(),
        rdev::Key::CapsLock => "[CapsLock]".to_string(),
        rdev::Key::Delete => "[Delete]".to_string(),
        rdev::Key::End => "[End]".to_string(),
        rdev::Key::Home => "[Home]".to_string(),
        rdev::Key::PageDown => "[PageDown]".to_string(),
        rdev::Key::PageUp => "[PageUp]".to_string(),
        rdev::Key::Insert => "[Insert]".to_string(),
        rdev::Key::F1 => "[F1]".to_string(),
        rdev::Key::F2 => "[F2]".to_string(),
        rdev::Key::F3 => "[F3]".to_string(),
        rdev::Key::F4 => "[F4]".to_string(),
        rdev::Key::F5 => "[F5]".to_string(),
        rdev::Key::F6 => "[F6]".to_string(),
        rdev::Key::F7 => "[F7]".to_string(),
        rdev::Key::F8 => "[F8]".to_string(),
        rdev::Key::F9 => "[F9]".to_string(),
        rdev::Key::F10 => "[F10]".to_string(),
        rdev::Key::F11 => "[F11]".to_string(),
        rdev::Key::F12 => "[F12]".to_string(),
        rdev::Key::KeyA => "a".to_string(),
        rdev::Key::KeyB => "b".to_string(),
        rdev::Key::KeyC => "c".to_string(),
        rdev::Key::KeyD => "d".to_string(),
        rdev::Key::KeyE => "e".to_string(),
        rdev::Key::KeyF => "f".to_string(),
        rdev::Key::KeyG => "g".to_string(),
        rdev::Key::KeyH => "h".to_string(),
        rdev::Key::KeyI => "i".to_string(),
        rdev::Key::KeyJ => "j".to_string(),
        rdev::Key::KeyK => "k".to_string(),
        rdev::Key::KeyL => "l".to_string(),
        rdev::Key::KeyM => "m".to_string(),
        rdev::Key::KeyN => "n".to_string(),
        rdev::Key::KeyO => "o".to_string(),
        rdev::Key::KeyP => "p".to_string(),
        rdev::Key::KeyQ => "q".to_string(),
        rdev::Key::KeyR => "r".to_string(),
        rdev::Key::KeyS => "s".to_string(),
        rdev::Key::KeyT => "t".to_string(),
        rdev::Key::KeyU => "u".to_string(),
        rdev::Key::KeyV => "v".to_string(),
        rdev::Key::KeyW => "w".to_string(),
        rdev::Key::KeyX => "x".to_string(),
        rdev::Key::KeyY => "y".to_string(),
        rdev::Key::KeyZ => "z".to_string(),
        rdev::Key::Num0 => "0".to_string(),
        rdev::Key::Num1 => "1".to_string(),
        rdev::Key::Num2 => "2".to_string(),
        rdev::Key::Num3 => "3".to_string(),
        rdev::Key::Num4 => "4".to_string(),
        rdev::Key::Num5 => "5".to_string(),
        rdev::Key::Num6 => "6".to_string(),
        rdev::Key::Num7 => "7".to_string(),
        rdev::Key::Num8 => "8".to_string(),
        rdev::Key::Num9 => "9".to_string(),
        // Symbols mapping
        rdev::Key::Minus => "-".to_string(),
        rdev::Key::Equal => "=".to_string(),
        rdev::Key::LeftBracket => "[".to_string(),
        rdev::Key::RightBracket => "]".to_string(),
        rdev::Key::BackSlash => "\\".to_string(),
        rdev::Key::SemiColon => ";".to_string(),
        rdev::Key::Quote => "'".to_string(),
        rdev::Key::Comma => ",".to_string(),
        rdev::Key::Dot => ".".to_string(),
        rdev::Key::Slash => "/".to_string(),
        rdev::Key::BackQuote => "`".to_string(),
        // Numpad mapping
        rdev::Key::Kp0 => "0".to_string(),
        rdev::Key::Kp1 => "1".to_string(),
        rdev::Key::Kp2 => "2".to_string(),
        rdev::Key::Kp3 => "3".to_string(),
        rdev::Key::Kp4 => "4".to_string(),
        rdev::Key::Kp5 => "5".to_string(),
        rdev::Key::Kp6 => "6".to_string(),
        rdev::Key::Kp7 => "7".to_string(),
        rdev::Key::Kp8 => "8".to_string(),
        rdev::Key::Kp9 => "9".to_string(),
        _ => "".to_string(),
    }
}

fn parse_special_key(s: &str) -> Option<Key> {
    match s {
        "[Control]" => Some(Key::Control),
        "[Shift]" => Some(Key::Shift),
        "[Alt]" => Some(Key::Alt),
        "[Meta]" => Some(Key::Meta),
        "[Return]" => Some(Key::Return),
        "[Escape]" => Some(Key::Escape),
        "[Backspace]" => Some(Key::Backspace),
        "[Tab]" => Some(Key::Tab),
        "[UpArrow]" => Some(Key::UpArrow),
        "[DownArrow]" => Some(Key::DownArrow),
        "[LeftArrow]" => Some(Key::LeftArrow),
        "[RightArrow]" => Some(Key::RightArrow),
        "[CapsLock]" => Some(Key::CapsLock),
        "[Delete]" => Some(Key::Delete),
        "[End]" => Some(Key::End),
        "[Home]" => Some(Key::Home),
        "[PageDown]" => Some(Key::PageDown),
        "[PageUp]" => Some(Key::PageUp),
        "[Insert]" => Some(Key::Insert),
        "[F1]" => Some(Key::F1),
        "[F2]" => Some(Key::F2),
        "[F3]" => Some(Key::F3),
        "[F4]" => Some(Key::F4),
        "[F5]" => Some(Key::F5),
        "[F6]" => Some(Key::F6),
        "[F7]" => Some(Key::F7),
        "[F8]" => Some(Key::F8),
        "[F9]" => Some(Key::F9),
        "[F10]" => Some(Key::F10),
        "[F11]" => Some(Key::F11),
        "[F12]" => Some(Key::F12),
        _ => None,
    }
}

impl Drop for InputService {
    fn drop(&mut self) {
        unsafe {
            set_system_cursors_hidden(false);
        }
    }
}

#[cfg(target_os = "windows")]
pub unsafe fn set_system_cursors_hidden(hide: bool) {
    #[link(name = "user32")]
    extern "system" {
        fn CreateCursor(
            hInst: *mut std::ffi::c_void,
            xHotSpot: i32,
            yHotSpot: i32,
            nWidth: i32,
            nHeight: i32,
            pvANDPlane: *const u8,
            pvXORPlane: *const u8,
        ) -> *mut std::ffi::c_void;
        fn SetSystemCursor(
            hcur: *mut std::ffi::c_void,
            id: u32,
        ) -> i32;
        fn SystemParametersInfoW(
            uiAction: u32,
            uiParam: u32,
            pvParam: *mut std::ffi::c_void,
            fWinIni: u32,
        ) -> i32;
    }

    const SPI_SETCURSORS: u32 = 0x0057;
    const SPIF_SENDCHANGE: u32 = 0x0002;

    const OCR_NORMAL: u32 = 32512;
    const OCR_IBEAM: u32 = 32513;
    const OCR_WAIT: u32 = 32514;
    const OCR_CROSS: u32 = 32515;
    const OCR_UP: u32 = 32516;
    const OCR_SIZENWSE: u32 = 32642;
    const OCR_SIZENESW: u32 = 32643;
    const OCR_SIZEWE: u32 = 32644;
    const OCR_SIZENS: u32 = 32645;
    const OCR_SIZEALL: u32 = 32646;
    const OCR_NO: u32 = 32648;
    const OCR_HAND: u32 = 32649;
    const OCR_APPSTARTING: u32 = 32650;

    let cursor_ids = [
        OCR_NORMAL, OCR_IBEAM, OCR_WAIT, OCR_CROSS, OCR_UP,
        OCR_SIZENWSE, OCR_SIZENESW, OCR_SIZEWE, OCR_SIZENS,
        OCR_SIZEALL, OCR_NO, OCR_HAND, OCR_APPSTARTING,
    ];

    if hide {
        let and_mask = [0xFFu8; 128];
        let xor_mask = [0x00u8; 128];
        for &id in &cursor_ids {
            let h_cursor = CreateCursor(
                std::ptr::null_mut(),
                0,
                0,
                32,
                32,
                and_mask.as_ptr(),
                xor_mask.as_ptr(),
            );
            if !h_cursor.is_null() {
                SetSystemCursor(h_cursor, id);
            }
        }
    } else {
        SystemParametersInfoW(SPI_SETCURSORS, 0, std::ptr::null_mut(), SPIF_SENDCHANGE);
    }
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn set_system_cursors_hidden(_hide: bool) {
    // No-op on non-Windows
}

