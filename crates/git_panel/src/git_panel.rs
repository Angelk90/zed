use crate::{GitPanelSettings};
use anyhow::Result;
use gpui::{
    div, Label, ViewContext, ListAlignment, Pixels, IconName,
    FocusHandle, EventEmitter, Panel, DockPosition, WindowContext, Render,
    StatefulInteractiveElement, Task,
};
use settings::SettingsStore;
use std::{sync::Arc};
use workspace::{
    notifications::NotificationId,
    dock::PanelEvent,
    Workspace,
};

#[derive(Serialize, Deserialize)]
struct SerializedGitPanel {
    width: Option<Pixels>,
}

#[derive(Debug)]
pub enum Event {
    DockPositionChanged,
    Focus,
    Dismissed,
}

pub fn init(cx: &mut AppContext) {
    cx.observe_new_views(|workspace: &mut Workspace, _| {
        workspace.register_action(|workspace, _: &ToggleFocus, cx| {
            // Implement your action logic here  
        });
    })
    .detach();
}

impl GitPanel {
    pub fn new(workspace: &mut Workspace, cx: &mut ViewContext<Workspace>) -> View<Self> {
        // Implementation of new function  
    }

    pub fn load(
        workspace: WeakView<Workspace>,
        cx: AsyncWindowContext,
    ) -> Task<Result<View<Self>>> {
        // Implementation of load function  
    }

    fn serialize(&mut self, cx: &mut ViewContext<Self>) {
        // Implementation of serialize function  
    }
}

impl Render for GitPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        this.child(
            v_flex().p_4().child(
                div().flex().w_full().items_center().child(
                    Label::new("You have no notifications.")
                        .color(Color::Muted)
                        .size(LabelSize::Small),
                ),
            ),
        )
    }
}

impl FocusableView for GitPanel {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<Event> for GitPanel {}
impl EventEmitter<PanelEvent> for GitPanel {}

impl Panel for GitPanel {
    fn persistent_name() -> &'static str {
        "GitPanel"
    }

    fn position(&self, cx: &gpui::WindowContext) -> DockPosition {
        GitPanelSettings::get_global(cx).dock  
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(position, DockPosition::Left | DockPosition::Right)
    }

    fn set_position(&mut self, position: DockPosition, cx: &mut ViewContext<Self>) {
        settings::update_settings_file::<GitPanelSettings>(
            self.fs.clone(),
            cx,
            move |settings, _| settings.dock = Some(position),
        );
    }

    fn size(&self, cx: &gpui::WindowContext) -> Pixels {
        self.width  
            .unwrap_or_else(|| GitPanelSettings::get_global(cx).default_width)
    }

    fn set_size(&mut self, size: Option<Pixels>, cx: &mut ViewContext<Self>) {
        self.width = size;
        self.serialize(cx);
        cx.notify();
    }

    fn set_active(&mut self, active: bool, cx: &mut ViewContext<Self>) {
        self.active = active;
    }

    fn icon(&self, cx: &gpui::WindowContext) -> Option<IconName> {
        let show_button = GitPanelSettings::get_global(cx).button;
        if !show_button {
            return None;
        }

        Some(IconName::BellDot)
    }

    fn icon_tooltip(&self, _cx: &WindowContext) -> Option<&'static str> {
        Some("Notification Panel")
    }

    fn icon_label(&self, cx: &WindowContext) -> Option<String> {
        let count = self.notification_store.read(cx).unread_notification_count();
        if count == 0 {
            None  
        } else {
            Some(count.to_string())
        }
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        Box::new(ToggleFocus)
    }
}