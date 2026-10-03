pub mod about;
pub mod animated_text;
pub mod io_panel;
pub mod main_panel;
pub mod notificaiton_panel;
pub mod phase_panel;
pub mod phase_progress_panel;
pub mod statistics_panel;
pub mod total_progress_panel;

use ratatui::Frame;

use crate::module::ui::{
    AppModel, Message,
    component::{
        io_panel::IoPanelComponent, main_panel::MainPanelComponent,
        notificaiton_panel::NotificationPanelComponent, phase_panel::PhasePanelComponent,
        phase_progress_panel::PhaseProgressPanelComponent,
        statistics_panel::StatisticsPanelComponent,
        total_progress_panel::TotalProgressPanelComponent,
    },
};

pub trait Component {
    fn update(&mut self, app_model: &mut AppModel, message: &Message) -> Vec<Message>;
    fn render(&mut self, app_model: &AppModel, frame: &mut Frame);
    fn enable(&mut self);
    fn disable(&mut self);
}

pub fn initialize_components() -> Vec<Box<dyn Component>> {
    let io_panel_component = IoPanelComponent::new();
    let phase_panel_component = PhasePanelComponent::new();
    let statistics_panel_component = StatisticsPanelComponent::new();
    let notification_panel_component = NotificationPanelComponent::new();
    let main_panel_component = MainPanelComponent::new();
    let phase_panel_progress_component = PhaseProgressPanelComponent::new();
    let total_panel_progress_component = TotalProgressPanelComponent::new();

    vec![
        Box::new(io_panel_component),
        Box::new(phase_panel_component),
        Box::new(statistics_panel_component),
        Box::new(notification_panel_component),
        Box::new(main_panel_component),
        Box::new(phase_panel_progress_component),
        Box::new(total_panel_progress_component),
    ]
}
