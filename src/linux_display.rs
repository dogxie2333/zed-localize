//! One-shot Wayland readiness check; no watcher or resident display service.
use anyhow::{Result, bail};
use clap::ValueEnum;
use std::{
    process::Command,
    sync::mpsc,
    time::{Duration, Instant},
};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::wl_registry,
};

#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum DisplayServer {
    #[default]
    Auto,
    Wayland,
    X11,
}

struct Registry;
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Registry {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

fn seat_ready() -> Result<bool> {
    let connection = Connection::connect_to_env()?;
    let (globals, _queue) = registry_queue_init::<Registry>(&connection)?;
    Ok(globals
        .contents()
        // Verified official Zed 1.23.2 requires wl_pointer.frame (seat v5+).
        .with_list(|items| {
            items
                .iter()
                .any(|item| item.interface == "wl_seat" && item.version >= 5)
        }))
}

fn wait_for_seat(timeout: Duration) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    // The deadline also bounds a compositor that accepts the socket but stops
    // answering roundtrips. Any blocked worker exits with this short-lived CLI.
    std::thread::spawn(move || {
        let start = Instant::now();
        loop {
            match seat_ready() {
                Ok(true) => {
                    let _ = sender.send(Ok(()));
                    return;
                }
                result if start.elapsed() >= timeout => {
                    let reason = match result {
                        Err(error) => error.to_string(),
                        _ => "compositor did not advertise wl_seat version 5 or newer".to_owned(),
                    };
                    let _ = sender.send(Err(reason));
                    return;
                }
                _ => std::thread::sleep(Duration::from_millis(100)),
            }
        }
    });
    match receiver.recv_timeout(timeout) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(reason)) => bail!("Wayland input is not ready: {reason}"),
        Err(_) => bail!(
            "Wayland input readiness timed out after {} seconds",
            timeout.as_secs()
        ),
    }
}

fn select(
    requested: DisplayServer,
    has_wayland: bool,
    has_x11: bool,
    ready: bool,
) -> Result<DisplayServer> {
    match requested {
        DisplayServer::X11 if has_x11 => Ok(DisplayServer::X11),
        DisplayServer::X11 => bail!("X11 requires DISPLAY"),
        DisplayServer::Wayland if has_wayland && ready => Ok(DisplayServer::Wayland),
        DisplayServer::Wayland => bail!("Wayland requires WAYLAND_DISPLAY and a ready wl_seat"),
        DisplayServer::Auto if has_wayland && ready => Ok(DisplayServer::Wayland),
        DisplayServer::Auto if has_x11 => Ok(DisplayServer::X11),
        DisplayServer::Auto => bail!("No ready display server; check WAYLAND_DISPLAY / DISPLAY"),
    }
}

pub fn prepare(command: &mut Command, requested: DisplayServer) -> Result<&'static str> {
    let has_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some_and(|value| !value.is_empty());
    let has_x11 = std::env::var_os("DISPLAY").is_some_and(|value| !value.is_empty());
    let readiness = if requested != DisplayServer::X11 && has_wayland {
        wait_for_seat(Duration::from_secs(5))
    } else {
        Ok(())
    };
    if requested == DisplayServer::Wayland {
        if let Err(error) = &readiness {
            bail!("{error}");
        }
    } else if let Err(error) = &readiness {
        eprintln!("{error}; trying X11");
    }
    match select(requested, has_wayland, has_x11, readiness.is_ok())? {
        DisplayServer::X11 => {
            command.env_remove("WAYLAND_DISPLAY");
            Ok("x11")
        }
        DisplayServer::Wayland => Ok("wayland"),
        DisplayServer::Auto => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_keeps_ready_wayland_and_falls_back_only_when_possible() {
        assert_eq!(
            select(DisplayServer::Auto, true, true, true).unwrap(),
            DisplayServer::Wayland
        );
        assert_eq!(
            select(DisplayServer::Auto, true, true, false).unwrap(),
            DisplayServer::X11
        );
        assert!(select(DisplayServer::Auto, true, false, false).is_err());
        assert_eq!(
            select(DisplayServer::Auto, false, true, true).unwrap(),
            DisplayServer::X11
        );
    }
    #[test]
    fn explicit_selection_does_not_silently_change_protocol() {
        assert!(select(DisplayServer::Wayland, true, true, false).is_err());
        assert!(select(DisplayServer::Wayland, false, true, true).is_err());
        assert_eq!(
            select(DisplayServer::X11, true, true, false).unwrap(),
            DisplayServer::X11
        );
        assert!(select(DisplayServer::X11, true, false, true).is_err());
    }
}
