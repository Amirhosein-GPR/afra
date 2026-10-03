use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Alignment,
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::module::ui::{AppModel, Message, Phase, component::Component};

pub struct PhasePanelComponent {
    enabled: bool,
    highlight_enabled: bool,
    highlight_index: u8,
    v_scrollbar_state: ScrollbarState,
    h_scrollbar_state: ScrollbarState,
    line_strings: Vec<String>,
}

impl PhasePanelComponent {
    pub fn new() -> Self {
        Self {
            enabled: false,
            highlight_enabled: false,
            highlight_index: 0,
            v_scrollbar_state: ScrollbarState::new(0),
            h_scrollbar_state: ScrollbarState::new(0),
            line_strings: vec![
                "1. Compiling".to_string(),
                "2. Disassembling".to_string(),
                "3. Simulating".to_string(),
                "4. Processing Original Assemblies".to_string(),
                "5. Extracting Subroutines".to_string(),
                "6. Extracting Main Subroutine Indices".to_string(),
                "7. Extracting Basic Blocks".to_string(),
                // "8. Computing Runtimes".to_string(),
                "8. Computing CFGs".to_string(),
                "9. Trimming Unconnected Graphs".to_string(),
                "10. Exporting CFGs".to_string(),
                "11. Exporting Benchmarks".to_string(),
            ],
        }
    }
}

impl Component for PhasePanelComponent {
    fn update(&mut self, app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                match key_event.code {
                    KeyCode::Right | KeyCode::Char('l') => {
                        self.disable();
                        return vec![Message::FocusOnMainPanel];
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.disable();
                        return vec![Message::FocusOnIoPanel];
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.disable();
                        return vec![Message::FocusOnStatisticsPanel];
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
            Message::FocusOnPhasePanel => {
                self.enable();
            }
            Message::Resize => {
                self.v_scrollbar_state = ScrollbarState::new(
                    (self.line_strings.len() as u16 + 1)
                        .saturating_sub(app_model.layout_manager.phase_panel_paragraph_area.height)
                        as usize,
                );
                self.h_scrollbar_state = ScrollbarState::new(
                    (self.line_strings.iter().map(|l| l.len()).max().unwrap() as u16 + 1)
                        .saturating_sub(app_model.layout_manager.phase_panel_paragraph_area.width)
                        as usize,
                );
            }
            _ => {}
        }

        match app_model.phase {
            Phase::Welcome => {}
            Phase::Compilation => self.highlight_enabled = true,
            Phase::Disassembling => self.highlight_index = 1,
            Phase::Simulating => self.highlight_index = 2,
            Phase::ProcessingOriginalAssemblies => self.highlight_index = 3,
            Phase::ExtractingSubroutines => self.highlight_index = 4,
            Phase::ExtractingMainSubroutineIndices => self.highlight_index = 5,
            Phase::ExtractingBasicBlocks => self.highlight_index = 6,
            // Phase::ComputingRuntimes => self.highlight_index = 7,
            Phase::ComputingCFG => self.highlight_index = 7,
            Phase::TrimmingUnconnectedGraphs => self.highlight_index = 8,
            Phase::ExportingCFG => self.highlight_index = 9,
            Phase::ExportingBenchmarks => self.highlight_index = 10,
        }

        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        let mut block = Block::bordered()
            .title(" Phases ")
            .title_alignment(Alignment::Center)
            .green();
        if self.enabled {
            block = block.border_type(BorderType::Double);
        }
        let title = Paragraph::new("").block(block);
        let mut lines = self
            .line_strings
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect::<Vec<_>>();
        if self.highlight_enabled {
            for i in 0..self.highlight_index {
                lines[i as usize] = lines[i as usize].clone().italic().green();
            }
            lines[self.highlight_index as usize] = lines[self.highlight_index as usize]
                .clone()
                .bold()
                .black()
                .on_green();
        }
        let paragraph = Paragraph::new(lines).white().scroll((
            self.v_scrollbar_state.get_position() as u16,
            self.h_scrollbar_state.get_position() as u16,
        ));
        let v_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let h_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);

        frame.render_widget(title, app_model.layout_manager.phase_panel_area);
        frame.render_widget(
            paragraph,
            app_model.layout_manager.phase_panel_paragraph_area,
        );
        frame.render_stateful_widget(
            v_scrollbar,
            app_model.layout_manager.phase_panel_area,
            &mut self.v_scrollbar_state,
        );
        frame.render_stateful_widget(
            h_scrollbar,
            app_model.layout_manager.phase_panel_area,
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
