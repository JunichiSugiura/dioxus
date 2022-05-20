use crate::event::{UIEvent, WindowEvent::*};
use futures_intrusive::channel::shared::{Receiver, Sender};
use std::fmt::Debug;
use wry::application::event_loop::EventLoopProxy;

pub type ProxyType<CoreCommand> = EventLoopProxy<UIEvent<CoreCommand>>;

#[derive(Clone)]
pub struct DesktopContext<CoreCommand: Debug + 'static + Clone, UICommand: 'static + Clone> {
    proxy: ProxyType<CoreCommand>,
    channel: (Sender<CoreCommand>, Receiver<UICommand>),
}

impl<CoreCommand, UICommand> DesktopContext<CoreCommand, UICommand>
where
    CoreCommand: Debug + Clone,
    UICommand: Debug + Clone,
{
    pub fn new(
        proxy: ProxyType<CoreCommand>,
        channel: (Sender<CoreCommand>, Receiver<UICommand>),
    ) -> Self {
        Self { proxy, channel }
    }

    pub fn receiver(&self) -> Receiver<UICommand> {
        self.channel.1.clone()
    }

    pub fn send(&self, cmd: CoreCommand) {
        self.channel
            .0
            .try_send(cmd)
            .expect("Failed to send CoreCommand");
    }

    pub fn drag(&self) {
        let _ = self.proxy.send_event(UIEvent::WindowEvent(DragWindow));
    }

    pub fn set_minimized(&self, minimized: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(Minimize(minimized)));
    }

    pub fn set_maximized(&self, maximized: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(Maximize(maximized)));
    }

    pub fn toggle_maximized(&self) {
        let _ = self.proxy.send_event(UIEvent::WindowEvent(MaximizeToggle));
    }

    pub fn set_visible(&self, visible: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(Visible(visible)));
    }

    pub fn close(&self) {
        let _ = self.proxy.send_event(UIEvent::WindowEvent(CloseWindow));
    }

    pub fn focus(&self) {
        let _ = self.proxy.send_event(UIEvent::WindowEvent(FocusWindow));
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(Fullscreen(fullscreen)));
    }

    pub fn set_resizable(&self, resizable: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(Resizable(resizable)));
    }

    pub fn set_always_on_top(&self, top: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(AlwaysOnTop(top)));
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(CursorVisible(visible)));
    }

    pub fn set_cursor_grab(&self, grab: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(CursorGrab(grab)));
    }

    pub fn set_title(&self, title: &str) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(SetTitle(String::from(title))));
    }

    pub fn set_decorations(&self, decoration: bool) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(SetDecorations(decoration)));
    }

    pub fn devtool(&self) {
        let _ = self.proxy.send_event(UIEvent::WindowEvent(DevTool));
    }

    pub fn eval(&self, script: impl std::string::ToString) {
        let _ = self
            .proxy
            .send_event(UIEvent::WindowEvent(Eval(script.to_string())));
    }
}
