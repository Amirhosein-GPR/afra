use std::{env, fs, process::Stdio};

use clap::Parser;
use regex::{Captures, Regex};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStderr, ChildStdout, Command},
    sync::mpsc::Sender,
};

use crate::module::{
    config::{
        ASSEMBLY_EXT_CLEANED_NECESSARY, ASSEMBLY_EXT_ORGINAL, ASSEMBLY_EXT_PROCESSED, BINARY_EXT,
        TRACE_EXT,
    },
    ui::component::main_panel::ChannelMessage,
};

/// Represents cli arguments of the program.
#[derive(Parser)]
#[command(
    name = "CFGC",
    version,
    about = "A Control Flow Graph (CFG) generator for AArch64 (Arm-64bit) programs"
)]
pub struct Cli {
    #[arg(short, long)]
    /// Path to the source files. If set, '--binary_path' option will be ignored.
    pub source_path: Option<String>,
    #[arg(short, long)]
    /// Path to the binary files. If no '--source_path' option is set this option will be used.
    pub binary_path: Option<String>,
}

#[derive(Debug)]
pub enum FileType {
    AsmOriginal,
    AsmProcessed,
    AsmCleanedNecessary,
    CfgText,
    CfgGraphics,
}

pub struct InputManager {
    pub src_paths: Option<Vec<String>>,
    pub bin_paths: Vec<String>,
    pub asm_paths: Option<Vec<String>>,
    pub trace_paths: Option<Vec<String>>,
    pub cfg_text_paths: Option<Vec<String>>,
    pub cfg_graphics_paths: Option<Vec<String>>,
}

impl InputManager {
    pub fn new(src_file_regex: &Regex) -> Self {
        env::set_current_dir("../..").unwrap();

        if !fs::exists("workspace/gem5_traces").unwrap() {
            fs::create_dir("workspace/gem5_traces").unwrap();
        }

        let cli = Cli::parse();

        match cli.source_path.as_ref() {
            Some(source_path) => {
                let src_paths = Self::get_files_in_path(&source_path);
                let bin_paths = Self::create_bin_paths(&src_paths, src_file_regex);

                Self {
                    src_paths: Some(src_paths),
                    bin_paths,
                    asm_paths: None,
                    trace_paths: None,
                    cfg_text_paths: None,
                    cfg_graphics_paths: None,
                }
            }
            None => match cli.binary_path.as_ref() {
                Some(binary_path) => Self {
                    src_paths: None,
                    bin_paths: Self::get_files_in_path(&binary_path),
                    asm_paths: None,
                    trace_paths: None,
                    cfg_text_paths: None,
                    cfg_graphics_paths: None,
                },
                None => {
                    panic!(
                        "At least one of the options should be set! Run with --help for more information.",
                    )
                }
            },
        }
    }

    pub fn get_binary_paths(&self) -> &Vec<String> {
        &self.bin_paths
    }

    pub fn get_assembly_paths(
        &mut self,
        bin_file_regex: &Regex,
        file_type: FileType,
    ) -> Vec<String> {
        if self.asm_paths.is_none() {
            self.asm_paths = Some(Vec::new());

            for bin_path in &self.bin_paths {
                let asm_path = bin_file_regex.replace(&bin_path, |caps: &Captures<'_>| {
                    format!(
                        "{}{}{}{}",
                        &caps[1], "assemblies/original", &caps[2], ASSEMBLY_EXT_ORGINAL
                    )
                });
                self.asm_paths.as_mut().unwrap().push(asm_path.to_string());
            }
        }

        match file_type {
            FileType::AsmOriginal => {
                Self::create_parent_dirs(self.asm_paths.as_ref().unwrap().first().unwrap());
                self.asm_paths.clone().unwrap()
            }
            FileType::AsmProcessed => {
                let mut processed_file_paths = Vec::new();
                for bin_path in &self.bin_paths {
                    let processed_file_path =
                        bin_file_regex.replace(&bin_path, |caps: &Captures<'_>| {
                            format!(
                                "{}assemblies/processed{}{}",
                                &caps[1], &caps[2], ASSEMBLY_EXT_PROCESSED
                            )
                        });
                    processed_file_paths.push(processed_file_path.to_string());
                }

                Self::create_parent_dirs(processed_file_paths.first().unwrap());
                processed_file_paths
            }
            FileType::AsmCleanedNecessary => {
                let mut cleaned_necessary_file_paths = Vec::new();
                for bin_path in &self.bin_paths {
                    let asm_path = bin_file_regex.replace(&bin_path, |caps: &Captures<'_>| {
                        format!(
                            "{}assemblies/cleaned_necessary{}{}",
                            &caps[1], &caps[2], ASSEMBLY_EXT_CLEANED_NECESSARY
                        )
                    });
                    cleaned_necessary_file_paths.push(asm_path.to_string());
                }

                Self::create_parent_dirs(cleaned_necessary_file_paths.first().unwrap());
                cleaned_necessary_file_paths
            }
            _ => {
                panic!("Wrong file type passed to get_assembly_paths function: {file_type:?}")
            }
        }
    }

    pub fn get_gem5_trace_paths(&mut self, bin_file_regex: &Regex) -> &Vec<String> {
        if self.trace_paths.is_none() {
            self.trace_paths = Some(Vec::new());

            for bin_path in &self.bin_paths {
                let trace_path = bin_file_regex.replace(&bin_path, |caps: &Captures<'_>| {
                    format!("workspace/gem5_traces{}{}", &caps[2], TRACE_EXT)
                });
                self.trace_paths
                    .as_mut()
                    .unwrap()
                    .push(trace_path.to_string());
            }
        }

        self.trace_paths.as_ref().unwrap()
    }

    pub fn get_cfg_paths(&mut self, bin_file_regex: &Regex, file_type: FileType) -> &Vec<String> {
        match file_type {
            FileType::CfgText => {
                if self.cfg_text_paths.is_none() {
                    self.cfg_text_paths = Some(Vec::new());

                    for bin_path in &self.bin_paths {
                        let cfg_text_path = bin_file_regex
                            .replace(&bin_path, |caps: &Captures<'_>| {
                                format!("workspace/cfg/text{}", &caps[2])
                            });
                        self.cfg_text_paths
                            .as_mut()
                            .unwrap()
                            .push(cfg_text_path.to_string());
                    }
                }
                self.cfg_text_paths.as_ref().unwrap()
            }
            FileType::CfgGraphics => {
                if self.cfg_graphics_paths.is_none() {
                    self.cfg_graphics_paths = Some(Vec::new());

                    for bin_path in &self.bin_paths {
                        let cfg_graphics_path = bin_file_regex
                            .replace(&bin_path, |caps: &Captures<'_>| {
                                format!("workspace/cfg/graphics{}", &caps[2])
                            });
                        self.cfg_graphics_paths
                            .as_mut()
                            .unwrap()
                            .push(cfg_graphics_path.to_string());
                    }
                }
                self.cfg_graphics_paths.as_ref().unwrap()
            }
            _ => {
                panic!("Wrong file type passed to get_cfg_paths function: {file_type:?}");
            }
        }
    }

    fn get_files_in_path(input_path: &str) -> Vec<String> {
        // Creating the absolute path from the program's relative path.
        let input_path = format!(
            "{}/{}",
            env::current_dir().unwrap().to_str().unwrap(),
            input_path
        );

        let meta_data = fs::metadata(&input_path).unwrap();

        let input_paths = if meta_data.is_file() {
            vec![input_path]
        } else if meta_data.is_dir() {
            fs::read_dir(input_path)
                .unwrap()
                .map(|e| e.unwrap().path().to_str().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            panic!("Error: Invalid input directory or file path!");
        };

        input_paths
    }

    fn create_bin_paths(src_paths: &[String], src_file_regex: &Regex) -> Vec<String> {
        let mut bin_paths = Vec::new();
        for src_path in src_paths {
            let bin_path = src_file_regex.replace(&src_path, |caps: &Captures<'_>| {
                format!("{}{}{}{}", &caps[1], "binaries", &caps[2], BINARY_EXT)
            });

            bin_paths.push(bin_path.to_string());
        }

        Self::create_parent_dirs(bin_paths.first().unwrap());

        bin_paths
    }

    pub fn create_parent_dirs(file_path: &str) {
        let parent_dir_path = file_path.rsplit_once('/').unwrap().0;
        if !fs::exists(parent_dir_path).unwrap() {
            fs::create_dir_all(parent_dir_path).unwrap();
        }
    }
}

pub fn run_command(cmd: &str, args: &[&str]) -> (BufReader<ChildStdout>, BufReader<ChildStderr>) {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    (BufReader::new(child_stdout), BufReader::new(child_stderr))
}

fn run_command_and_set_dir(cmd: &str, args: &[&str], dir: &str) -> BufReader<ChildStdout> {
    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();

    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(format!("{}/{dir}", current_dir))
        .spawn()
        .unwrap();

    let child_stdout = child.stdout.take().unwrap();

    BufReader::new(child_stdout)
}

/// Same as [`run_command`] but also returns the output of the command execution.
fn run_command_and_get_output(cmd: &str, args: &[&str]) -> String {
    String::from_utf8(
        std::process::Command::new(cmd)
            .args(args)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
}

/// Compiles source file(s) in C language with the help of 'aarch64-linux-gnu-gcc' cross compiler.
pub async fn gcc_compile_source_files(
    src_paths: Vec<String>,
    bin_paths: Vec<String>,
    tx: Sender<ChannelMessage>,
) {
    tx.send(ChannelMessage::Text(
        "=================== Compiling With 'aarch64-linux-gnu-gcc' Cross Compiler ===================="
            .to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(
        "<--> CMD: \"aarch64-linux-gnu-gcc [SOURCE_PATH] -static -lm -g -o [BINARY_PATH]\" "
            .to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();

    for i in 0..src_paths.len() {
        tx.send(ChannelMessage::Text(format!(
            "Compiling: '{}'",
            src_paths[i]
        )))
        .await
        .unwrap();

        let (buf_reader_stdout, buf_reader_stderr) = run_command(
            "aarch64-linux-gnu-gcc",
            &[&src_paths[i], "-static", "-lm", "-g", "-o", &bin_paths[i]],
        );
        let mut lines = buf_reader_stdout.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            tx.send(ChannelMessage::Text(line)).await.unwrap();
        }
        let mut lines = buf_reader_stderr.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            tx.send(ChannelMessage::Text(line)).await.unwrap();
        }

        tx.send(ChannelMessage::Text(format!(
            "Saved At: '{}'",
            bin_paths[i]
        )))
        .await
        .unwrap();
        tx.send(ChannelMessage::Text(String::new())).await.unwrap();

        tx.send(ChannelMessage::PhaseProgress(
            (i + 1) as f64 / src_paths.len() as f64,
        ))
        .await
        .unwrap();
    }

    tx.send(ChannelMessage::Text(
        "=================================== Compilation Finished! ===================================="
            .to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();

    tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
}

/// Disassembles aarch64 binay file(s) with the help of 'aarch64-linux-gnu-objdump' disassembler.
pub async fn objdump_binary_files(
    bin_paths: Vec<String>,
    asm_paths: Vec<String>,
    tx: Sender<ChannelMessage>,
) {
    tx.send(ChannelMessage::Text(
        "================ Disassembling With 'aarch64-linux-gnu-objdump' Disassembler =================".to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(
        "<--> CMD: \"aarch64-linux-gnu-objdump -d -l [BINARY_PATH] > [DISASSEMBLED_PATH]\""
            .to_string(),
    ))
    .await
    .unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();

    for i in 0..bin_paths.len() {
        tx.send(ChannelMessage::Text(format!(
            "ObjDump disassembling: '{}'",
            bin_paths[i]
        )))
        .await
        .unwrap();

        let output =
            run_command_and_get_output("aarch64-linux-gnu-objdump", &["-d", "-l", &bin_paths[i]]);
        fs::write(&asm_paths[i], output).unwrap();

        tx.send(ChannelMessage::Text(format!(
            "Saved disassembled file at: '{}'\n",
            asm_paths[i]
        )))
        .await
        .unwrap();
        tx.send(ChannelMessage::Text(String::new())).await.unwrap();

        tx.send(ChannelMessage::PhaseProgress(
            (i + 1) as f64 / bin_paths.len() as f64,
        ))
        .await
        .unwrap();
    }

    tx.send(ChannelMessage::Text(
        "================================== Disassembling Finished! ==================================="
            .to_string(),
    ))
    .await
    .unwrap();

    tx.send(ChannelMessage::Text(String::new())).await.unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();

    tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
}

/// Simulates the execution of the binary programs(s) with the help of `gem5` simulator.
pub async fn gem5_simulate(bin_paths: Vec<String>, tx: Sender<ChannelMessage>) {
    tx.send(ChannelMessage::Text(
        "============================== Simulating With 'gem5' Simulator ==============================".to_string(),
    ))
    .await
    .unwrap();
    tx.send(
        ChannelMessage::Text("<--> CMD: \"../programs/gem5/build/ALL/gem5.opt --debug-flags=Exec,-ExecSymbol --debug-file=../gem5_traces/[TRACE_FILE_NAME].trace ../programs/gem5_arm_script.py --binary [BINARY_FILE_PATH]\"".to_string())
    ).await.unwrap();
    tx.send(ChannelMessage::Text(String::new())).await.unwrap();

    let file_regex = Regex::new(r"([\w\-]*)\.run$").unwrap();

    for (i, bp) in bin_paths.iter().enumerate() {
        let file_name = file_regex.captures(bp).unwrap().get(1).unwrap().as_str();

        tx.send(ChannelMessage::Text(format!("gem5 simulating: '{bp}'")))
            .await
            .unwrap();

        let buf_reader = run_command_and_set_dir(
            "../programs/gem5/build/ALL/gem5.opt",
            &[
                "--debug-flags=Exec,-ExecSymbol",
                format!("--debug-file=../gem5_traces/{file_name}.trace").as_str(),
                "../programs/gem5_arm_script.py",
                "--binary",
                bp,
            ],
            "workspace",
        );
        let mut lines = buf_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            tx.send(ChannelMessage::Text(line)).await.unwrap();
        }

        tx.send(ChannelMessage::Text(format!(
            "gem5 saved trace result at: 'workspace/gem5_traces/{file_name}.trace'"
        )))
        .await
        .unwrap();
        tx.send(ChannelMessage::Text(String::new())).await.unwrap();

        tx.send(ChannelMessage::PhaseProgress(
            (i + 1) as f64 / bin_paths.len() as f64,
        ))
        .await
        .unwrap();
    }

    tx.send(ChannelMessage::Text(
        "===================================== Simulation Finished! ===================================".to_string(),
    ))
    .await
    .unwrap();

    tx.send(ChannelMessage::PhaseFinishedSignal).await.unwrap();
}
