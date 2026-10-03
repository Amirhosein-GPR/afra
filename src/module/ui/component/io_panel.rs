use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Alignment,
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::module::ui::{AppModel, Message, component::Component};

pub struct IoPanelComponent {
    enabled: bool,
    v_scrollbar_state: ScrollbarState,
    h_scrollbar_state: ScrollbarState,
    line_strings: Vec<String>,
}

impl IoPanelComponent {
    pub fn new() -> Self {
        Self {
            enabled: false,
            v_scrollbar_state: ScrollbarState::new(0),
            h_scrollbar_state: ScrollbarState::new(0),
            line_strings: vec![
                "Sources: benchmarks/.../sources/*".to_string(),
                "Binaries: benchmarks/.../binaries/*".to_string(),
                "Assemblies (Origianl): benchmarks/.../assemblies/original/*".to_string(),
                "Assemblies (Cleaned Full): benchmarks/.../assemblies/processed/*".to_string(),
                "Assemblies (Cleaned Necessary): benchmarks/.../assemblies/cleaned_necessary/*"
                    .to_string(),
                "Gem5 Trace: workspace/gem5_traces/*".to_string(),
                "CFG Text: workspace/cfg/text/*".to_string(),
                "CFG Graphics: workspace/cfg/graphics/*".to_string(),
            ],
        }
    }
}

impl Component for IoPanelComponent {
    fn update(&mut self, app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                match key_event.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.disable();
                        return vec![Message::FocusOnNotificationPanel];
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.disable();
                        return vec![Message::FocusOnPhasePanel];
                    }
                    _ => {}
                }
            }
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('H') => {
                        self.h_scrollbar_state.first();
                    }
                    KeyCode::Right | KeyCode::Char('L') => {
                        self.h_scrollbar_state.last();
                    }
                    KeyCode::Up | KeyCode::Char('K') => {
                        self.v_scrollbar_state.first();
                    }
                    KeyCode::Down | KeyCode::Char('J') => {
                        self.v_scrollbar_state.last();
                    }
                    _ => {}
                }
            }
            Message::KeyEvent(key_event) if self.enabled => match key_event.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    self.h_scrollbar_state.prev();
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.h_scrollbar_state.next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.v_scrollbar_state.prev();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.v_scrollbar_state.next();
                }
                _ => {}
            },
            Message::FocusOnIoPanel => {
                self.enable();
            }
            Message::Resize => {
                self.v_scrollbar_state = ScrollbarState::new(
                    (self.line_strings.len() as u16 + 1)
                        .saturating_sub(app_model.layout_manager.io_panel_paragraph_area.height)
                        as usize,
                );
                self.h_scrollbar_state = ScrollbarState::new(
                    (self.line_strings.iter().map(|l| l.len()).max().unwrap() as u16 + 1)
                        .saturating_sub(app_model.layout_manager.io_panel_paragraph_area.width)
                        as usize,
                );
            }
            _ => {}
        }
        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        let mut block = Block::bordered()
            .title(" Inputs / Outputs ")
            .title_alignment(Alignment::Center);
        if self.enabled {
            block = block.border_type(BorderType::Double);
        }

        let title = Paragraph::new("").block(block).red();
        let lines = self
            .line_strings
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines).white().scroll((
            self.v_scrollbar_state.get_position() as u16,
            self.h_scrollbar_state.get_position() as u16,
        ));
        let v_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let h_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);

        frame.render_widget(title, app_model.layout_manager.io_panel_area);
        frame.render_widget(paragraph, app_model.layout_manager.io_panel_paragraph_area);

        frame.render_stateful_widget(
            v_scrollbar,
            app_model.layout_manager.io_panel_area,
            &mut self.v_scrollbar_state,
        );
        frame.render_stateful_widget(
            h_scrollbar,
            app_model.layout_manager.io_panel_area,
            &mut self.h_scrollbar_state,
        );
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
