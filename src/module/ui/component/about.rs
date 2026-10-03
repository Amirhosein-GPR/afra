use ratatui::{Frame, style::Stylize, widgets::Paragraph};

use crate::module::{
    config::APP_VERSION,
    ui::{AppModel, Message, component::Component},
};

pub struct AboutComponent {
    enabled: bool,
}

impl AboutComponent {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl Component for AboutComponent {
    fn update(&mut self, _app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        match message {
            Message::StartWizard => {
                self.disable();
                return vec![Message::NextNotification];
            }
            _ => {}
        }
        Vec::new()
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        if self.enabled {
            let logo_pars = [
                Paragraph::new(
                    "
         .8.         
        .888.        
       :88888.       
      . `88888.      
     .8. `88888.     
    .8`8. `88888.    
   .8' `8. `88888.   
  .8'   `8. `88888.  
 .888888888. `88888. 
.8'       `8. `88888.",
                )
                .blue(),
                Paragraph::new(
                    "
8 8888888888  
8 8888        
8 8888        
8 8888        
8 888888888888
8 8888        
8 8888        
8 8888        
8 8888        
8 8888        
",
                )
                .green(),
                Paragraph::new(
                    "
8 888888888o.  
8 8888    `88. 
8 8888     `88 
8 8888     ,88 
8 8888.   ,88' 
8 888888888P'  
8 8888`8b      
8 8888 `8b.    
8 8888   `8b.  
8 8888     `88.
",
                )
                .red(),
                Paragraph::new(
                    "
         .8.         
        .888.        
       :88888.       
      . `88888.      
     .8. `88888.     
    .8`8. `88888.    
   .8' `8. `88888.   
  .8'   `8. `88888.  
 .888888888. `88888. 
.8'       `8. `88888.",
                )
                .light_yellow(),
            ];
            let version = Paragraph::new(APP_VERSION).centered().white();
            let message = Paragraph::new("Press <R> to Start!")
                .centered()
                .slow_blink()
                .white();

            frame.render_widget(&logo_pars[0], app_model.layout_manager.about_logo_area1);
            frame.render_widget(&logo_pars[1], app_model.layout_manager.about_logo_area2);
            frame.render_widget(&logo_pars[2], app_model.layout_manager.about_logo_area3);
            frame.render_widget(&logo_pars[3], app_model.layout_manager.about_logo_area4);

            frame.render_widget(version, app_model.layout_manager.about_version_area);

            frame.render_widget(message, app_model.layout_manager.about_message_area);
        }
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
