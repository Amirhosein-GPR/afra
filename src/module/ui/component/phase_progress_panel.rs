use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Alignment,
    style::{Style, Stylize},
    widgets::{Block, BorderType, LineGauge, Paragraph},
};

use crate::module::ui::{AppModel, Message, component::Component};

pub struct PhaseProgressPanelComponent {
    enabled: bool,
    progress_ratio: f64,
}

impl PhaseProgressPanelComponent {
    pub fn new() -> Self {
        Self {
            enabled: false,
            progress_ratio: 0.0,
        }
    }
}

impl Component for PhaseProgressPanelComponent {
    fn update(&mut self, _app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.disable();
                        return vec![Message::FocusOnStatisticsPanel];
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.disable();
                        return vec![Message::FocusOnTotalProgressPanel];
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.disable();
                        return vec![Message::FocusOnMainPanel];
                    }
                    _ => {}
                }
            }
            Message::FocusOnPhaseProgressPanel => {
                self.enable();
            }
            Message::PhaseProgress(phase_progress) => self.progress_ratio = *phase_progress,
            _ => {}
        }

        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        let mut block = Block::bordered()
            .title(" Phase Progress ")
            .title_alignment(Alignment::Center);
        if self.enabled {
            block = block.border_type(BorderType::Double);
        }

        let title = Paragraph::new("").block(block).green();
        let line_gauge = LineGauge::default()
            .ratio(self.progress_ratio)
            .unfilled_style(Style::new().on_black().black())
            .filled_style(Style::new().on_black().green());

        frame.render_widget(title, app_model.layout_manager.phase_progress_panel_area);
        frame.render_widget(
            line_gauge,
            app_model
                .layout_manager
                .phase_progress_panel_line_gauge_area,
        );
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
