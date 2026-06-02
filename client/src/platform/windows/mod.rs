use anyhow::Result;
use protocol::{KeyEvent, MouseBtn, MouseMove, MouseScroll, Payload};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

#[derive(Default, Clone, Copy)]
struct CursorSample {
    x: i32,
    y: i32,
}

#[derive(Default)]
pub struct InputEngine {
    suppress_until_us: u64,
    last_pos: Option<CursorSample>,
}

impl InputEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture_tick(&mut self) -> Vec<Payload> {
        let mut output = Vec::new();
        if self.is_suppressed() {
            self.last_pos = read_cursor().ok();
            return output;
        }

        if let Ok(current) = read_cursor() {
            if let Some(prev) = self.last_pos {
                let dx = current.x - prev.x;
                let dy = current.y - prev.y;
                if dx != 0 || dy != 0 {
                    output.push(Payload::MouseMove(MouseMove { dx, dy }));
                }
            }
            self.last_pos = Some(current);
        }
        output
    }

    pub fn inject(&mut self, payload: &Payload) -> Result<()> {
        // TODO: replace with SendInput integration.
        match payload {
            Payload::MouseMove(MouseMove { .. })
            | Payload::MouseBtn(MouseBtn { .. })
            | Payload::MouseScroll(MouseScroll { .. })
            | Payload::Key(KeyEvent { .. }) => {
                self.suppress_until_us = now_us() + 8_000;
                match payload {
                    Payload::MouseMove(MouseMove { dx, dy }) => send_relative_move(*dx, *dy)?,
                    Payload::MouseBtn(MouseBtn { button, is_down }) => send_mouse_button(*button, *is_down)?,
                    Payload::MouseScroll(MouseScroll { delta_y, .. }) => send_mouse_wheel(*delta_y)?,
                    Payload::Key(KeyEvent { key_code, is_down }) => send_key(*key_code, *is_down)?,
                    _ => {}
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn is_suppressed(&self) -> bool {
        now_us() < self.suppress_until_us
    }
}

fn now_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

fn read_cursor() -> Result<CursorSample> {
    let mut point = POINT::default();
    // SAFETY: Win32 API writes into provided POINT pointer.
    let ok = unsafe { GetCursorPos(&mut point) };
    if !ok.as_bool() {
        anyhow::bail!("GetCursorPos failed");
    }
    Ok(CursorSample {
        x: point.x,
        y: point.y,
    })
}

fn send_relative_move(dx: i32, dy: i32) -> Result<()> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    // SAFETY: SendInput reads a valid INPUT array for immediate call.
    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        anyhow::bail!("SendInput failed");
    }
    Ok(())
}

fn send_mouse_button(button: u8, is_down: bool) -> Result<()> {
    let flags = match (button, is_down) {
        (1, true) => MOUSEEVENTF_LEFTDOWN,
        (1, false) => MOUSEEVENTF_LEFTUP,
        (2, true) => MOUSEEVENTF_RIGHTDOWN,
        (2, false) => MOUSEEVENTF_RIGHTUP,
        (3, true) => MOUSEEVENTF_MIDDLEDOWN,
        (3, false) => MOUSEEVENTF_MIDDLEUP,
        _ => return Ok(()),
    };

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    // SAFETY: SendInput reads a valid INPUT array for immediate call.
    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        anyhow::bail!("SendInput mouse button failed");
    }
    Ok(())
}

fn send_mouse_wheel(delta_y: i32) -> Result<()> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: delta_y as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    // SAFETY: SendInput reads a valid INPUT array for immediate call.
    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        anyhow::bail!("SendInput wheel failed");
    }
    Ok(())
}

fn send_key(key_code: u32, is_down: bool) -> Result<()> {
    let mut flags = Default::default();
    if !is_down {
        flags = KEYEVENTF_KEYUP;
    }
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(key_code as u16),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    // SAFETY: SendInput reads a valid INPUT array for immediate call.
    let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        anyhow::bail!("SendInput key failed");
    }
    Ok(())
}
