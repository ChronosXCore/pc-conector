use crate::network::{KeyboardData, KeyboardEventType, MouseData, MouseEventType};
use enigo::{
    Coordinate, Direction, Enigo, Key, Keyboard as EnigoKeyboard, Mouse as EnigoMouse, Settings,
};
use log::{info, warn, error};
use rdev::{listen, Event, EventType};
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
    mouse_locked: Arc<Mutex<bool>>,
}

impl InputService {
    pub fn new() -> Self {
        let enigo = Enigo::new(&Settings::default()).unwrap_or_else(|_| panic!("Error al inicializar Enigo"));
        
        Self {
            enigo: Arc::new(Mutex::new(enigo)),
            is_capturing: Arc::new(Mutex::new(false)),
            on_input: None,
            mouse_locked: Arc::new(Mutex::new(false)),
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

        thread::spawn(move || {
            let callback = callback;
            let capturing = capturing;

            if let Err(e) = listen(move |event: Event| {
                if !*capturing.lock().unwrap() {
                    return;
                }

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
                        _ => None,
                    };

                    if let Some(event) = input_event {
                        cb(event);
                    }
                }
            }) {
                error!("Error en captura de entrada: {:?}", e);
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
