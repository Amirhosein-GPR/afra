use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Alignment,
    style::{Style, Stylize},
    widgets::{Block, BorderType, LineGauge, Paragraph},
};

use crate::module::ui::{AppModel, Message, Phase, component::Component};

pub struct TotalProgressPanelComponent {
    enabled: bool,
}

impl TotalProgressPanelComponent {
    pub fn new() -> Self {
        Self { enabled: false }
    }
}

impl Component for TotalProgressPanelComponent {
    fn update(&mut self, _app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.disable();
                        return vec![Message::FocusOnPhaseProgressPanel];
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.disable();
                        return vec![Message::FocusOnMainPanel];
                    }
                    _ => {}
                }
            }
            Message::FocusOnTotalProgressPanel => {
                self.enable();
            }
            _ => {}
        }

        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        let mut block = Block::bordered()
            .title(" Total Progress ")
            .title_alignment(Alignment::Center);
        if self.enabled {
            block = block.border_type(BorderType::Double);
        }

        let title = Paragraph::new("").block(block).red();
        let line_gauge = LineGauge::default()
            .ratio(app_model.phase.get_phase_number() as f64 / Phase::COUNT as f64)
            .unfilled_style(Style::new().on_black().black())
            .filled_style(Style::new().on_black().red());

        frame.render_widget(title, app_model.layout_manager.total_progress_panel_area);
        frame.render_widget(
            line_gauge,
            app_model
                .layout_manager
                .total_progress_panel_line_gauge_area,
        );
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
