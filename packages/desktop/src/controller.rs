use crate::desktop_context::{UserEvent, UserWindowEvent};
use dioxus_core::*;
use std::{
    collections::HashMap,
    sync::Arc,
    sync::{atomic::AtomicBool, Mutex},
};
use wry::{
    self,
    application::{event_loop::ControlFlow, event_loop::EventLoopProxy, window::WindowId},
    webview::WebView,
};

pub struct DesktopController {
    pub webviews: HashMap<WindowId, WebView>,
    pub sender: futures_channel::mpsc::UnboundedSender<SchedulerMsg>,
    pub(super) pending_edits: Arc<Mutex<Vec<String>>>,
    pub(super) quit_app_on_close: bool,
    pub is_ready: Arc<AtomicBool>,
}

impl DesktopController {
    // Launch the virtualdom on its own thread managed by tokio
    // returns the desktop state
    pub fn new_on_tokio<P, T, CoreCommand>(
        root: Component<P>,
        props: P,
        proxy: EventLoopProxy<UserEvent<CoreCommand>>,
        window_context: T,
    ) -> Self
    where
        P: 'static + Send,
        T: 'static + Send + Clone,
        CoreCommand: Send + Clone,
    {
        let edit_queue = Arc::new(Mutex::new(Vec::new()));
        let (sender, receiver) = futures_channel::mpsc::unbounded::<SchedulerMsg>();

        let pending_edits = edit_queue.clone();
        let return_sender = sender.clone();

        std::thread::spawn(move || {
            // We create the runtime as multithreaded, so you can still "spawn" onto multiple threads
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async move {
                let mut dom =
                    VirtualDom::new_with_props_and_scheduler(root, props, (sender, receiver));

                dom.base_scope().provide_context(window_context);

                let edits = dom.rebuild();

                edit_queue
                    .lock()
                    .unwrap()
                    .push(serde_json::to_string(&edits.edits).unwrap());

                // Make sure the window is ready for any new updates
                let _ = proxy.send_event(UserEvent::WindowEvent(UserWindowEvent::Update));

                loop {
                    dom.wait_for_work().await;
                    let mut muts = dom.work_with_deadline(|| false);

                    while let Some(edit) = muts.pop() {
                        edit_queue
                            .lock()
                            .unwrap()
                            .push(serde_json::to_string(&edit.edits).unwrap());
                    }

                    let _ = proxy.send_event(UserEvent::WindowEvent(UserWindowEvent::Update));
                }
            })
        });

        Self {
            pending_edits,
            sender: return_sender,
            webviews: HashMap::new(),
            is_ready: Arc::new(AtomicBool::new(false)),
            quit_app_on_close: true,
        }
    }

    pub fn close_window(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
        self.webviews.remove(&window_id);

        if self.webviews.is_empty() && self.quit_app_on_close {
            *control_flow = ControlFlow::Exit;
        }
    }

    pub(super) fn try_load_ready_webviews(&mut self) {
        if self.is_ready.load(std::sync::atomic::Ordering::Relaxed) {
            let mut queue = self.pending_edits.lock().unwrap();
            let (_id, view) = self.webviews.iter_mut().next().unwrap();

            while let Some(edit) = queue.pop() {
                view.evaluate_script(&format!("window.interpreter.handleEdits({})", edit))
                    .unwrap();
            }
        }
    }
}
