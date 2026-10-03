use ratatui::{Frame, layout::Position, style::Stylize, widgets::Paragraph};

use crate::module::ui::{AppModel, Message, component::Component};

pub struct AnimatedTextComponent {
    enabled: bool,
    text: String,
    current_index: usize,
}

impl AnimatedTextComponent {
    pub fn new(text: &str) -> Self {
        Self {
            enabled: true,
            text: text.to_string(),
            current_index: 0,
        }
    }

    pub fn change_text(&mut self, new_text: &str) {
        self.text = new_text.to_string();
        self.current_index = 0;
    }
}

impl Component for AnimatedTextComponent {
    fn update(&mut self, _app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::RenderLoopTick => {
                if self.enabled {
                    if self.current_index < self.text.len() {
                        self.current_index += 1;
                    } else {
                        self.enabled = false;
                        return vec![Message::AnimationIsFinished];
                    }
                }
            }
            _ => {}
        }

        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        frame.render_widget(
            Paragraph::new(&self.text[..self.current_index])
                .centered()
                .white(),
            app_model.layout_manager.notification_panel_text_area,
        );
        if self.enabled {
            frame.set_cursor_position(Position::new(
                app_model.layout_manager.notification_panel_text_area.x
                    + app_model.layout_manager.notification_panel_text_area.width / 2
                    + self.current_index.div_ceil(2) as u16,
                app_model.layout_manager.notification_panel_text_area.y,
            ));
        }
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
