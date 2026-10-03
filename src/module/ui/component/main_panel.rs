use std::{sync::Arc, time::Instant};

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Alignment,
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::module::{
    analyzer::AssemblyMetaData,
    assembly::Subroutine,
    basic_block::BasicBlock,
    benchmark::{self, BenchmarkStatistics},
    cfg,
    io::{self, FileType, InputManager},
    regex::RegexContainer,
    ui::{
        AppModel, Message, Phase,
        component::{Component, about::AboutComponent},
    },
};

pub enum ChannelMessage {
    Text(String),
    PhaseFinishedSignal,
    Subroutines(Vec<Vec<Subroutine>>),
    MainSubroutineIndices(Vec<usize>),
    BasicBlocks(Vec<Vec<BasicBlock>>),
    AssemblyMetaData(AssemblyMetaData),
    CFGExportTimes(u128, u128),
    NodeEdgeCounts(usize, usize),
    PhaseProgress(f64),
}

pub struct MainPanelComponent {
    regex_container: Arc<RegexContainer>,
    input_manager: InputManager,
    assembly_meta_data: Option<AssemblyMetaData>,
    enabled: bool,
    content: Vec<String>,
    v_scrollbar_state: ScrollbarState,
    h_scrollbar_state: ScrollbarState,
    max_line_length: u16,
    about: AboutComponent,
    tx: Sender<ChannelMessage>,
    rx: Receiver<ChannelMessage>,
    no_phase_running: bool,
    total_start: Instant,
    analysis_start: Instant,
    benchmark_statistics: BenchmarkStatistics,
}

impl MainPanelComponent {
    pub fn new() -> Self {
        let regex_container = Arc::new(RegexContainer::new());
        let input_manager = InputManager::new(&regex_container.src_file_regex);

        let (tx, rx) = mpsc::channel::<ChannelMessage>(100);

        Self {
            regex_container,
            input_manager,
            assembly_meta_data: Some(AssemblyMetaData::new()),
            enabled: true,
            content: Vec::new(),
            v_scrollbar_state: ScrollbarState::new(0),
            h_scrollbar_state: ScrollbarState::new(0),
            max_line_length: 0,
            about: AboutComponent::new(),
            tx,
            rx,
            no_phase_running: true,
            total_start: Instant::now(),
            analysis_start: Instant::now(),
            benchmark_statistics: BenchmarkStatistics::new(),
        }
    }
}

impl Component for MainPanelComponent {
    fn update(&mut self, app_model: &mut AppModel, message: &Message) -> Vec<Message> {
        let mut messages = Vec::new();
        match message {
            Message::KeyEvent(key_event)
                if self.enabled && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                match key_event.code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        self.disable();
                        messages.push(Message::FocusOnIoPanel);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.disable();
                        messages.push(Message::FocusOnNotificationPanel);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.disable();
                        messages.push(Message::FocusOnPhaseProgressPanel);
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
                KeyCode::Char('r') if app_model.phase == Phase::Welcome => {
                    self.total_start = Instant::now();
                    app_model.phase = Phase::Compilation;

                    messages.push(Message::RunPhase);
                    messages.push(Message::StartWizard);
                }
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
            Message::FocusOnMainPanel => {
                self.enable();
            }

            Message::RunPhase if self.no_phase_running => {
                self.no_phase_running = false;

                match app_model.phase {
                    Phase::Welcome => {}
                    Phase::Compilation => {
                        tokio::spawn(io::gcc_compile_source_files(
                            self.input_manager.src_paths.as_ref().unwrap().clone(),
                            self.input_manager.get_binary_paths().clone(),
                            self.tx.clone(),
                        ));
                    }
                    Phase::Disassembling => {
                        let asm_paths = self.input_manager.get_assembly_paths(
                            &self.regex_container.bin_file_regex,
                            FileType::AsmOriginal,
                        );

                        tokio::spawn(io::objdump_binary_files(
                            self.input_manager.get_binary_paths().clone(),
                            asm_paths,
                            self.tx.clone(),
                        ));
                    }
                    Phase::Simulating => {
                        tokio::spawn(io::gem5_simulate(
                            self.input_manager.get_binary_paths().clone(),
                            self.tx.clone(),
                        ));
                    }
                    Phase::ProcessingOriginalAssemblies => {
                        self.analysis_start = Instant::now();

                        let asm_paths = self.input_manager.get_assembly_paths(
                            &self.regex_container.bin_file_regex,
                            FileType::AsmOriginal,
                        );
                        let processed_asm_paths = self.input_manager.get_assembly_paths(
                            &self.regex_container.bin_file_regex,
                            FileType::AsmProcessed,
                        );
                        let regex_container = Arc::clone(&self.regex_container);

                        tokio::spawn(AssemblyMetaData::process_original_assemblies(
                            asm_paths,
                            processed_asm_paths,
                            regex_container,
                            self.tx.clone(),
                        ));
                    }
                    Phase::ExtractingSubroutines => {
                        let processed_asm_paths = self.input_manager.get_assembly_paths(
                            &self.regex_container.bin_file_regex,
                            FileType::AsmProcessed,
                        );
                        let regex_container = Arc::clone(&self.regex_container);

                        self.assembly_meta_data.as_mut().unwrap().processed_asm_path =
                            processed_asm_paths.clone();

                        tokio::spawn(AssemblyMetaData::extract_subroutines(
                            processed_asm_paths,
                            regex_container,
                            self.tx.clone(),
                        ));
                    }
                    Phase::ExtractingMainSubroutineIndices => {
                        let subroutines = self
                            .assembly_meta_data
                            .as_mut()
                            .unwrap()
                            .subroutines
                            .take()
                            .unwrap();

                        tokio::spawn(AssemblyMetaData::extract_main_subroutine_indices(
                            subroutines,
                            self.tx.clone(),
                        ));
                    }
                    Phase::ExtractingBasicBlocks => {
                        let subroutines = self
                            .assembly_meta_data
                            .as_mut()
                            .unwrap()
                            .subroutines
                            .take()
                            .unwrap();
                        let trace_paths = self
                            .input_manager
                            .get_gem5_trace_paths(&self.regex_container.bin_file_regex)
                            .clone();
                        let regex_container = Arc::clone(&self.regex_container);

                        tokio::spawn(AssemblyMetaData::extract_basic_blocks(
                            subroutines,
                            trace_paths,
                            regex_container,
                            self.tx.clone(),
                        ));
                    }
                    // Phase::ComputingRuntimes => {
                    //     let tx = self.tx.clone();

                    //     tokio::spawn(async move {
                    //         tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
                    //     });
                    // }
                    Phase::ComputingCFG => {
                        let asm_meta_data = self.assembly_meta_data.take().unwrap();

                        tokio::spawn(cfg::compute_control_flow(asm_meta_data, self.tx.clone()));
                    }
                    Phase::TrimmingUnconnectedGraphs => {
                        let asm_meta_data = self.assembly_meta_data.take().unwrap();

                        tokio::spawn(cfg::trim_unconnected_basic_blocks(
                            asm_meta_data,
                            self.tx.clone(),
                        ));
                    }
                    Phase::ExportingCFG => {
                        self.benchmark_statistics.analysis_time =
                            self.analysis_start.elapsed().as_millis();

                        let asm_meta_data = self.assembly_meta_data.take().unwrap();
                        let cfg_text_paths = self
                            .input_manager
                            .get_cfg_paths(&self.regex_container.bin_file_regex, FileType::CfgText)
                            .clone();
                        let cfg_graphics_paths = self
                            .input_manager
                            .get_cfg_paths(
                                &self.regex_container.bin_file_regex,
                                FileType::CfgGraphics,
                            )
                            .clone();

                        tokio::spawn(cfg::export_to_all_formats(
                            asm_meta_data,
                            cfg_text_paths,
                            cfg_graphics_paths,
                            self.tx.clone(),
                        ));
                    }
                    Phase::ExportingBenchmarks => {
                        let asm_meta_data = self.assembly_meta_data.as_ref().unwrap();

                        let connected_bb_iter = asm_meta_data
                            .basic_blocks
                            .iter()
                            .flatten()
                            .filter(|bb| bb.connected);

                        self.benchmark_statistics.total_node_count =
                            connected_bb_iter.clone().count();
                        self.benchmark_statistics.total_edge_count =
                            connected_bb_iter.map(|bb| bb.edges.len()).sum();

                        self.benchmark_statistics.total_time =
                            self.total_start.elapsed().as_millis();

                        benchmark::write_benchmarks(&self.benchmark_statistics);

                        messages.push(Message::ShowStatistics(self.benchmark_statistics.clone()));
                        messages.push(Message::NextNotification);
                        messages.push(Message::FocusOnStatisticsPanel);
                        messages.push(Message::PhaseProgress(1.0));
                    }
                }
            }
            _ => {}
        }

        loop {
            if let Ok(channel_message) = self.rx.try_recv() {
                match channel_message {
                    ChannelMessage::Text(line) => {
                        self.content.push(line);

                        let current_position = self.v_scrollbar_state.get_position();
                        let content_length = self.content.len().saturating_sub(
                            app_model.layout_manager.main_panel_area.height as usize - 2,
                        );

                        if current_position == content_length.saturating_sub(2) {
                            self.v_scrollbar_state =
                                ScrollbarState::new(content_length).position(content_length);
                        } else {
                            self.v_scrollbar_state =
                                ScrollbarState::new(content_length).position(current_position);
                        }

                        let line_len = self.content.last().unwrap().len();
                        if line_len as u16 > self.max_line_length {
                            self.max_line_length = line_len as u16;

                            self.h_scrollbar_state =
                                ScrollbarState::new((line_len + 5).saturating_sub(
                                    app_model.layout_manager.main_panel_area.width as usize,
                                ))
                                .position(self.h_scrollbar_state.get_position());
                        }

                        continue;
                    }
                    ChannelMessage::PhaseFinishedSignal => {
                        app_model.phase.next_phase();
                        self.no_phase_running = true;
                        messages.push(Message::RunPhase);
                        messages.push(Message::NextNotification);
                        messages.push(Message::PhaseProgress(0.0));
                    }
                    ChannelMessage::Subroutines(subroutines) => {
                        self.assembly_meta_data.as_mut().unwrap().subroutines = Some(subroutines);
                    }
                    ChannelMessage::MainSubroutineIndices(main_subroutine_indices) => {
                        self.assembly_meta_data
                            .as_mut()
                            .unwrap()
                            .main_subroutine_indices = main_subroutine_indices;
                    }
                    ChannelMessage::BasicBlocks(basic_blocks) => {
                        self.assembly_meta_data.as_mut().unwrap().basic_blocks = basic_blocks;
                    }
                    ChannelMessage::AssemblyMetaData(assembly_meta_data) => {
                        self.assembly_meta_data = Some(assembly_meta_data);
                    }
                    ChannelMessage::CFGExportTimes(text_export_time, graphics_export_time) => {
                        self.benchmark_statistics.text_export_time = text_export_time;
                        self.benchmark_statistics.graphics_export_time = graphics_export_time;
                    }
                    ChannelMessage::NodeEdgeCounts(total_node_count, total_edege_count) => {
                        self.benchmark_statistics.total_node_count = total_node_count;
                        self.benchmark_statistics.total_edge_count = total_edege_count;
                    }
                    ChannelMessage::PhaseProgress(phase_progress) => {
                        messages.push(Message::PhaseProgress(phase_progress));
                    }
                }
            }

            break;
        }

        for m in self.about.update(app_model, message) {
            messages.push(m);
        }

        messages
    }

    fn render(&mut self, app_model: &AppModel, frame: &mut Frame) {
        let mut block = Block::bordered()
            .title(" Main Panel ")
            .title_alignment(Alignment::Center);
        if self.enabled {
            block = block.border_type(BorderType::Double);
        }

        let title = Paragraph::new("").block(block).light_yellow();
        let content = Paragraph::new(
            self.content
                .iter()
                .map(|cl| Line::raw(cl))
                .collect::<Vec<Line>>(),
        )
        .scroll((
            self.v_scrollbar_state.get_position() as u16,
            self.h_scrollbar_state.get_position() as u16,
        ))
        .white();
        let v_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let h_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);

        frame.render_widget(title, app_model.layout_manager.main_panel_area);
        frame.render_widget(content, app_model.layout_manager.main_panel_content_area);
        frame.render_stateful_widget(
            v_scrollbar,
            app_model.layout_manager.main_panel_area,
            &mut self.v_scrollbar_state,
        );
        frame.render_stateful_widget(
            h_scrollbar,
            app_model.layout_manager.main_panel_area,
            &mut self.h_scrollbar_state,
        );
        self.about.render(app_model, frame);
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false
    }
}
