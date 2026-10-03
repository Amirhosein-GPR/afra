pub mod component;

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Margin, Rect},
    DefaultTerminal, Frame,
};
use std::{thread, time::Duration};

use crate::module::{benchmark::BenchmarkStatistics, ui::component::Component};

const TICK_VALUE: u8 = 16;

pub enum Message {
    RenderLoopExit,
    RenderLoopTick,
    Resize,
    NextNotification,
    NotificationComplete,
    AnimationIsFinished,
    StartWizard,
    KeyEvent(KeyEvent),
    IoIsAlreadySet,
    FocusOnIoPanel,
    FocusOnPhasePanel,
    FocusOnStatisticsPanel,
    FocusOnNotificationPanel,
    FocusOnMainPanel,
    FocusOnPhaseProgressPanel,
    FocusOnTotalProgressPanel,
    RunPhase,
    ShowStatistics(BenchmarkStatistics),
    PhaseProgress(f64),
}

#[derive(PartialEq)]
pub enum Phase {
    Welcome,
    Compilation,
    Disassembling,
    Simulating,
    ProcessingOriginalAssemblies,
    ExtractingSubroutines,
    ExtractingMainSubroutineIndices,
    ExtractingBasicBlocks,
    // ComputingRuntimes,
    ComputingCFG,
    TrimmingUnconnectedGraphs,
    ExportingCFG,
    ExportingBenchmarks,
}

impl Phase {
    pub const COUNT: u8 = 11;

    pub fn next_phase(&mut self) {
        match self {
            Phase::Welcome => *self = Phase::Compilation,
            Phase::Compilation => *self = Phase::Disassembling,
            Phase::Disassembling => *self = Phase::Simulating,
            Phase::Simulating => *self = Phase::ProcessingOriginalAssemblies,
            Phase::ProcessingOriginalAssemblies => *self = Phase::ExtractingSubroutines,
            Phase::ExtractingSubroutines => *self = Phase::ExtractingMainSubroutineIndices,
            Phase::ExtractingMainSubroutineIndices => *self = Phase::ExtractingBasicBlocks,
            Phase::ExtractingBasicBlocks => *self = Phase::ComputingCFG,
            // Phase::ExtractingBasicBlocks => *self = Phase::ComputingRuntimes,
            // Phase::ComputingRuntimes => *self = Phase::ComputingCFG,
            Phase::ComputingCFG => *self = Phase::TrimmingUnconnectedGraphs,
            Phase::TrimmingUnconnectedGraphs => *self = Phase::ExportingCFG,
            Phase::ExportingCFG => *self = Phase::ExportingBenchmarks,
            Phase::ExportingBenchmarks => *self = Phase::ExportingBenchmarks,
        }
    }

    pub fn get_phase_number(&self) -> u8 {
        match self {
            Phase::Welcome => 0,
            Phase::Compilation => 1,
            Phase::Disassembling => 2,
            Phase::Simulating => 3,
            Phase::ProcessingOriginalAssemblies => 4,
            Phase::ExtractingSubroutines => 5,
            Phase::ExtractingMainSubroutineIndices => 6,
            Phase::ExtractingBasicBlocks => 7,
            Phase::ComputingCFG => 8,
            Phase::TrimmingUnconnectedGraphs => 9,
            Phase::ExportingCFG => 10,
            Phase::ExportingBenchmarks => 11,
        }
    }
}

pub struct AppModel {
    layout_manager: LayoutManager,
    phase: Phase,
}

impl AppModel {
    fn new() -> Self {
        let layout_manager = LayoutManager::new(Rect::new(0, 0, 0, 0));

        Self {
            layout_manager,
            phase: Phase::Welcome,
        }
    }
}

pub struct App {
    ui_components: Vec<Box<dyn Component>>,
    app_model: AppModel,
}

impl App {
    pub fn new() -> Self {
        let ui_components = component::initialize_components();
        let app_model = AppModel::new();

        Self {
            ui_components,
            app_model,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut message_stack = Vec::new();
        'outer: loop {
            let message = self.handle_events();
            message_stack.push(message);

            while let Some(message) = message_stack.pop() {
                terminal.draw(|frame| {
                    let frame_area = frame.area();
                    if self.app_model.layout_manager.area.width != frame_area.width
                        || self.app_model.layout_manager.area.height != frame_area.height
                    {
                        self.app_model.layout_manager = LayoutManager::new(frame_area);
                        message_stack.push(Message::Resize);
                    } else {
                        self.update(&mut message_stack, &message);
                        self.render(frame)
                    }
                })?;

                match message {
                    Message::RenderLoopExit => break 'outer,
                    Message::RenderLoopTick => {
                        thread::sleep(Duration::from_millis(TICK_VALUE as u64));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, message_stack: &mut Vec<Message>, message: &Message) {
        for component in &mut self.ui_components {
            let messages = component.update(&mut self.app_model, message);
            if messages.len() > 0 {
                for message in messages {
                    message_stack.push(message);
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        for component in &mut self.ui_components {
            component.render(&self.app_model, frame);
        }
    }

    fn handle_events(&mut self) -> Message {
        if event::poll(Duration::from_secs(0)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key_event) => match &key_event.code {
                    KeyCode::Char('q') => return Message::RenderLoopExit,
                    _ => return Message::KeyEvent(key_event),
                },
                _ => {}
            }
        }

        Message::RenderLoopTick
    }
}

struct LayoutManager {
    area: Rect,
    io_panel_area: Rect,
    io_panel_paragraph_area: Rect,
    phase_panel_area: Rect,
    phase_panel_paragraph_area: Rect,
    statistics_panel_area: Rect,
    statistics_panel_paragraph_area: Rect,
    notification_panel_area: Rect,
    notification_panel_text_area: Rect,
    main_panel_area: Rect,
    main_panel_content_area: Rect,
    about_logo_area1: Rect,
    about_logo_area2: Rect,
    about_logo_area3: Rect,
    about_logo_area4: Rect,
    about_version_area: Rect,
    about_message_area: Rect,
    phase_progress_panel_area: Rect,
    phase_progress_panel_line_gauge_area: Rect,
    total_progress_panel_area: Rect,
    total_progress_panel_line_gauge_area: Rect,
}

impl LayoutManager {
    fn new(area: Rect) -> Self {
        let [left_section, right_section] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)])
                .spacing(1)
                .areas::<2>(area);
        let [io_panel_area, phase_panel_area, statistics_panel_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas::<3>(left_section);

        let [notification_panel_area, main_panel_area, progress_section] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas::<3>(right_section);

        let [phase_progress_panel_area, total_progress_panel_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                .spacing(1)
                .areas::<2>(progress_section);

        let [io_panel_paragraph_area] = Layout::vertical([Constraint::Fill(1)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .areas::<1>(io_panel_area);

        let [phase_panel_paragraph_area] = Layout::vertical([Constraint::Fill(1)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .areas::<1>(phase_panel_area);

        let [statistics_panel_paragraph_area] = Layout::vertical([Constraint::Fill(1)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .areas::<1>(statistics_panel_area);

        let notification_panel_text_area = notification_panel_area.inner(Margin::new(2, 1));

        let main_panel_content_area = main_panel_area.inner(Margin::new(2, 1));

        let [_, about_logo_area, about_version_area, _, about_message_area, _] =
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas::<6>(main_panel_area)
            .map(|r| r.inner(Margin::new(1, 0)));
        let [_, about_logo_area1, _, about_logo_area2, _, about_logo_area3, _, about_logo_area4, _] =
            Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(21),
                Constraint::Length(3),
                Constraint::Length(14),
                Constraint::Length(3),
                Constraint::Length(15),
                Constraint::Length(3),
                Constraint::Length(21),
                Constraint::Fill(1),
            ])
            .areas::<9>(about_logo_area);

        let phase_progress_panel_line_gauge_area =
            phase_progress_panel_area.inner(Margin::new(2, 1));

        let total_progress_panel_line_gauge_area =
            total_progress_panel_area.inner(Margin::new(2, 1));

        Self {
            area,
            io_panel_area,
            io_panel_paragraph_area,
            phase_panel_area,
            phase_panel_paragraph_area,
            statistics_panel_area,
            statistics_panel_paragraph_area,
            notification_panel_area,
            notification_panel_text_area,
            main_panel_area,
            main_panel_content_area,
            about_logo_area1,
            about_logo_area2,
            about_logo_area3,
            about_logo_area4,
            about_version_area,
            about_message_area,
            phase_progress_panel_area,
            phase_progress_panel_line_gauge_area,
            total_progress_panel_area,
            total_progress_panel_line_gauge_area,
        }
    }
}
