use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Alignment,
    style::Stylize,
    widgets::{Block, BorderType, Paragraph},
};

use crate::module::ui::{
    AppModel, Message,
    component::{Component, animated_text::AnimatedTextComponent},
};

const MESSAGES: [&str; 13] = [
    "Welcome to AFRA!",
    "Compiling the source codes...",
    "Disassembling the binaries...",
    "Simulating the execution of binaries...",
    "Processing original assembly files (Removing redundant info and beutifying)...",
    "Extracting subroutines from processed assembly files...",
    "Extracting main subroutine indices (From Vec<Vec<Subroutine>>)...",
    "Extracting basic blocks (From Vec<Vec<Subroutine>>)...",
    // "Computing runtimes...",
    "Computing control flow graphs...",
    "Trimming unconnected graphs...",
    "Exporting CFGs...",
    "Exporting benchmarks to workspace/benchmark.bench.txt...",
    "AFRA finished all the works! Press 'Q' to quit!...",
];

pub struct NotificationPanelComponent {
    enabled: bool,
    current_message_index: u8,
    animated_text: AnimatedTextComponent,
}

impl NotificationPanelComponent {
    pub fn new() -> Self {
        let animated_text = AnimatedTextComponent::new(MESSAGES[0]);

        Self {
            enabled: false,
            current_message_index: 0,
            animated_text,
        }
    }
}

impl Component for NotificationPanelComponent {
    fn update(&mut self, app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::NextNotification => {
                if self.current_message_index < MESSAGES.len() as u8 - 1 {
                    self.current_message_index += 1;
                }

                self.animated_text
                    .change_text(MESSAGES[self.current_message_index as usize]);
                self.animated_text.enable();
            }
            Message::RenderLoopTick => {
                return self
                    .animated_text
                    .update(app_model, message)
                    .into_iter()
                    .map(|m| match m {
                        Message::AnimationIsFinished => {
                            self.animated_text.disable();
                            Message::NotificationComplete
                        }
                        _ => m,
                    })
                    .collect();
            }
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.disable();
                        return vec![Message::FocusOnIoPanel];
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.disable();
                        return vec![Message::FocusOnMainPanel];
                    }
                    _ => {}
                }
            }
            Message::FocusOnNotificationPanel => {
                self.enable();
            }
            _ => {}
        }

        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        let mut block = Block::bordered()
            .title(" Notifications ")
            .title_alignment(Alignment::Center);
        if self.enabled {
            block = block.border_type(BorderType::Double);
        }

        let title = Paragraph::new("").block(block).blue();

        frame.render_widget(title, app_model.layout_manager.notification_panel_area);
        self.animated_text.render(app_model, frame);
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
