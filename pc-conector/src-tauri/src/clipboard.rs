use arboard::Clipboard;
use log::info;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Servicio de sincronización de portapapeles entre PCs
pub struct ClipboardSync {
    clipboard: Arc<Mutex<Option<Clipboard>>>,
    last_content: Arc<Mutex<String>>,
    is_running: Arc<Mutex<bool>>,
    on_clipboard_change: Arc<Mutex<Option<Arc<dyn Fn(String) + Send + Sync + 'static>>>>,
}

impl ClipboardSync {
    pub fn new() -> Self {
        Self {
            clipboard: Arc::new(Mutex::new(None)),
            last_content: Arc::new(Mutex::new(String::new())),
            is_running: Arc::new(Mutex::new(false)),
            on_clipboard_change: Arc::new(Mutex::new(None)),
        }
    }

    /// Inicializar el portapapeles
    pub fn init(&mut self) -> Result<(), String> {
        let clip = Clipboard::new()
            .map_err(|e| format!("Error al inicializar portapapeles: {}", e))?;
        *self.clipboard.lock().unwrap() = Some(clip);
        Ok(())
    }

    /// Establecer callback para cuando cambie el portapapeles
    pub fn set_on_change<F>(&mut self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_clipboard_change.lock().unwrap() = Some(Arc::new(callback));
    }

    /// Leer el contenido actual del portapapeles
    pub fn read(&self) -> Result<String, String> {
        let mut guard = self.clipboard.lock().unwrap();
        if let Some(ref mut clip) = *guard {
            clip.get_text()
                .map_err(|e| format!("Error al leer portapapeles: {}", e))
        } else {
            Err("Portapapeles no inicializado".to_string())
        }
    }

    /// Escribir contenido en el portapapeles local
    pub fn write(&self, text: &str) -> Result<(), String> {
        let mut guard = self.clipboard.lock().unwrap();
        if let Some(ref mut clip) = *guard {
            clip.set_text(text.to_string())
                .map_err(|e| format!("Error al escribir portapapeles: {}", e))?;
            
            // Actualizar el último contenido para evitar loops
            *self.last_content.lock().unwrap() = text.to_string();
            info!("Portapapeles actualizado desde remoto");
            Ok(())
        } else {
            Err("Portapapeles no inicializado".to_string())
        }
    }

    /// Iniciar monitoreo del portapapeles en un hilo separado
    pub fn start_monitoring(&self) {
        let is_running = self.is_running.clone();
        *is_running.lock().unwrap() = true;

        let clipboard = self.clipboard.clone();
        let last_content = self.last_content.clone();
        let running = self.is_running.clone();
        let on_change = self.on_clipboard_change.clone();

        thread::spawn(move || {
            // Inicializar último contenido
            if let Ok(content) = clipboard.lock().unwrap().as_mut().unwrap().get_text() {
                *last_content.lock().unwrap() = content;
            }

            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(500));

                if let Ok(content) = clipboard.lock().unwrap().as_mut().unwrap().get_text() {
                    let mut last = last_content.lock().unwrap();
                    if content != *last {
                        info!("Portapapeles local cambió");
                        *last = content.clone();
                        
                        // Invocar el callback
                        let cb_guard = on_change.lock().unwrap();
                        if let Some(ref cb) = *cb_guard {
                            cb(content);
                        }
                    }
                }
            }
        });
    }

    /// Detener monitoreo
    pub fn stop_monitoring(&self) {
        *self.is_running.lock().unwrap() = false;
    }
}
