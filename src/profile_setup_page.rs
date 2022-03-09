use crate::profile_tab::{ProfileTab, ProfileTabState};
use adw::traits::{ActionRowExt, ExpanderRowExt, PreferencesGroupExt};
use adw::{ActionRow, Clamp, ExpanderRow, PreferencesGroup, Toast, ToastOverlay};
use gio::prelude::InputStreamExtManual;
use gio::traits::{FileExt, InputStreamExt};
use gio::{InputStream, SubprocessFlags, SubprocessLauncher};
use glib::{clone, BoolError, Cast, DateTime, Error as GError, MainContext, PRIORITY_DEFAULT};
use gtk::traits::{BoxExt, ButtonExt, EditableExt, FileChooserExt, NativeDialogExt, WidgetExt};
use gtk::{
    Align, Box as BoxWidget, Button, Entry, FileChooserAction, FileChooserNative, FileFilter,
    Label, Orientation, ResponseType, Window,
};
use serde_json::{Deserializer, Value};
use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const PERF_COMMAND: &str =
    "perf record --freq 99 --call-graph dwarf --output=${TMP_FILE} ${PROGRAM} ${PROGRAM_ARGUMENTS}";
const CARGO_BUILD_COMMAND: &str = "RUSTFLAGS=-g cargo build --release --message-format=json";

// TODO: Add comments

pub fn new_profile_setup_page(profile_tab: &ProfileTab) -> ToastOverlay {
    let cargo_toml_path = Label::builder()
        .css_classes(vec!["dim-label".to_owned()])
        .build();
    let cargo_toml_chooser_button = Button::builder()
        .label("Select")
        .valign(Align::Center)
        .build();

    let program_arguments_entry = Entry::builder()
        .placeholder_text("No arguments")
        .css_classes(vec!["monospace".to_owned()])
        .valign(Align::Center)
        .build();
    let perf_command_entry = Entry::builder()
        .text(PERF_COMMAND)
        .css_classes(vec!["monospace".to_owned()])
        .valign(Align::Center)
        .build();
    let cargo_build_entry = Entry::builder()
        .text(CARGO_BUILD_COMMAND)
        .css_classes(vec!["monospace".to_owned()])
        .valign(Align::Center)
        .build();

    let start_profiling_button = Button::builder()
        .label("Start Profiling")
        .sensitive(false)
        .css_classes(vec!["pill".to_owned(), "opaque".to_owned()])
        .build();
    let open_profile_button = Button::builder()
        .label("Open Existing Profile")
        .css_classes(vec![
            "pill".to_owned(),
            "opaque".to_owned(),
            "blue_button".to_owned(),
        ])
        .build();

    let cargo_toml_row = ActionRow::builder()
        .title("Cargo Project")
        .subtitle("Select a Cargo.toml")
        .build();
    cargo_toml_row.add_suffix(&{
        let container = BoxWidget::new(Orientation::Horizontal, 6);
        container.append(&cargo_toml_path);
        container.append(&cargo_toml_chooser_button);
        container
    });

    let cargo_toml_chooser = FileChooserNative::new(
        Some("Select a Cargo.toml"),
        Window::NONE,
        FileChooserAction::Open,
        None,
        None,
    );
    let cargo_toml_filter = FileFilter::new();
    cargo_toml_filter.set_name(Some("Cargo.toml"));
    cargo_toml_filter.add_pattern("Cargo.toml");
    cargo_toml_chooser.add_filter(&cargo_toml_filter);
    let profile_chooser = FileChooserNative::new(
        Some("Select a .perf.json"),
        Window::NONE,
        FileChooserAction::Open,
        None,
        None,
    );
    let profile_filter = FileFilter::new();
    profile_filter.set_name(Some(".perf.json"));
    profile_filter.add_pattern("*.perf.json");
    profile_chooser.add_filter(&profile_filter);

    let list = PreferencesGroup::builder()
        .title("Setup Profile")
        .description("Start profiling or open a previously recorded profile.")
        .build();
    list.add(&cargo_toml_row);
    list.add(&{
        let row = ActionRow::builder()
            .title("Program arguments")
            .subtitle("Arguments to pass to the program")
            .build();
        row.add_suffix(&program_arguments_entry);
        row
    });
    list.add(&{
        let row = ExpanderRow::builder()
            .title("Advanced Options")
            .subtitle("Configure commands")
            .build();
        row.add_row(&{
            let row = ActionRow::builder()
                .title("Perf")
                .subtitle("Command for perf")
                .build();
            row.add_suffix(&perf_command_entry);
            row
        });
        row.add_row(&{
            let row = ActionRow::builder()
                .title("Cargo Build")
                .subtitle("Command for cargo build")
                .build();
            row.add_suffix(&cargo_build_entry);
            row
        });
        row
    });

    let footer_box = BoxWidget::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .homogeneous(true)
        .css_classes(vec!["linked".to_owned()])
        .build();
    footer_box.append(&start_profiling_button);
    footer_box.append(&open_profile_button);

    let content = BoxWidget::new(Orientation::Vertical, 18);
    content.append(&list);
    content.append(&footer_box);

    let page = Clamp::builder()
        .child(&content)
        .hexpand(true)
        .vexpand(true)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();

    let error_toast = ToastOverlay::new();
    error_toast.set_child(Some(&page));

    // When a Cargo.toml is chosen, set the path label, update the tab title, and enable the start profiling button
    cargo_toml_chooser.connect_response(clone!(
        @weak profile_tab,
        @weak error_toast,
        @weak cargo_toml_path,
        @weak cargo_toml_row,
        @weak start_profiling_button,
        @weak open_profile_button
        => move |cargo_toml_chooser, response| {
            if response == ResponseType::Accept {
                let mut path = cargo_toml_chooser.file().unwrap().path().unwrap();
                path.pop();
                if let Some(path_str) = path.to_str() {
                    cargo_toml_row.set_css_classes(&["success_row"]);
                    start_profiling_button.add_css_class("green_button");
                    open_profile_button.remove_css_class("blue_button");


                    let profile_name = path.file_name().unwrap().to_str().unwrap();
                    profile_tab.set_data(ProfileTabState::Setup, profile_name);
                    cargo_toml_path.set_label(path_str);
                    start_profiling_button.set_sensitive(true);
                } else {
                    error_toast.add_toast(&Toast::new("Error: Project path is not valid UTF-8"));
                }
            }
        }
    ));
    cargo_toml_chooser_button.connect_clicked(move |cargo_toml_chooser_button| {
        cargo_toml_chooser.set_transient_for(
            cargo_toml_chooser_button
                .root()
                .map(Cast::downcast::<Window>)
                .map(Result::ok)
                .flatten()
                .as_ref(),
        );
        cargo_toml_chooser.show();
    });

    // When a .perf.json is chosen, switch the tab to ProfilePage
    profile_chooser.connect_response(clone!(@weak profile_tab =>
        move |profile_chooser, response| {
        if response == ResponseType::Accept {
            let profile_path = profile_chooser.file().unwrap().path().unwrap();
            profile_tab.switch_to_profile_page(&profile_path);
        }
    }));
    open_profile_button.connect_clicked(move |open_profile_button| {
        profile_chooser.set_transient_for(
            open_profile_button
                .root()
                .map(Cast::downcast::<Window>)
                .map(Result::ok)
                .flatten()
                .as_ref(),
        );
        profile_chooser.show();
    });

    // When start profiling is clicked: Build project, run perf, convert to .perf.json, and then switch the tab to ProfilePage
    start_profiling_button.connect_clicked(clone!(
        @weak page,
        @weak error_toast,
        @weak profile_tab,
        @weak cargo_toml_path,
        @weak cargo_build_entry,
        @weak perf_command_entry,
        @weak program_arguments_entry
        => move |_| {
            let prefix = match get_prefix() {
                Ok(prefix) => prefix,
                Err(error) => {
                    error_toast.add_toast(&Toast::new(&format!("Failed to get current datetime: {error}")));
                    return;
                },
            };

            page.set_sensitive(false);

            let project_directory = PathBuf::from(&cargo_toml_path.label());
            let profile_name = project_directory.file_name().unwrap().to_str().unwrap();
            let perf_file = format!("{profile_name}:{prefix}.perf.data");
            let profile = format!("{profile_name}:{prefix}.perf.json");

            let cargo_build_command = cargo_build_entry.text();
            let perf_command = perf_command_entry.text()
                .replace("${TMP_FILE}", &perf_file)
                .replace("${PROGRAM_ARGUMENTS}", &program_arguments_entry.text());
            let perf_convert_command = format!("perf data convert --input {perf_file} --to-json {profile}");

            MainContext::default().spawn_local(clone!(@weak page, @weak error_toast, @weak profile_tab => async move {
                let profiling_result = start_profiling(&project_directory, &cargo_build_command, perf_command, &perf_convert_command, &profile_tab).await;
                match profiling_result {
                    Ok(_) => {
                        let mut profile_path = project_directory;
                        profile_path.push(profile);
                        profile_tab.switch_to_profile_page(&profile_path);
                    },
                    Err(error) => {
                        let profile_name = project_directory.file_name().unwrap().to_str().unwrap();
                        profile_tab.set_data(ProfileTabState::Setup, profile_name);
                        page.set_sensitive(true);
                        error_toast.add_toast(&Toast::new(&format!("{error}")));
                        return;
                    },
                }
            }));
        }
    ));

    error_toast
}

fn get_prefix() -> Result<String, BoolError> {
    Ok(DateTime::now_utc()?.format("%F:%T")?.to_string())
}

async fn start_profiling(
    project_directory: &Path,
    cargo_build_command: &str,
    mut perf_command: String,
    perf_convert_command: &str,
    profile_tab: &ProfileTab,
) -> Result<(), Box<dyn Error>> {
    let profile_name = project_directory.file_name().unwrap().to_str().unwrap();
    profile_tab.set_data(ProfileTabState::SetupCompilingProgram, profile_name);

    let cargo_build_output = run_command(&cargo_build_command, &project_directory, true)
        .await
        .map_err(|error| format!("Failed to compile project: {error}"))?;

    let mut program_path = None;
    for compiler_message in Deserializer::from_slice(&cargo_build_output).into_iter::<Value>() {
        let compiler_message = compiler_message
            .map_err(|error| format!("Failed to parse compiler output: {error}"))?;
        if let Some(executable) = compiler_message.get("executable") {
            if let Some(prog_path) = executable.as_str() {
                perf_command = perf_command.replace("${PROGRAM}", prog_path);
                program_path = Some(PathBuf::from(prog_path));
                break;
            }
        }
    }
    let program_path = program_path
        .ok_or::<Box<dyn Error>>("Failed to find program path in compiler output".into())?;

    let profile_name = program_path.file_name().unwrap().to_str().unwrap();
    profile_tab.set_data(ProfileTabState::SetupProfilingProgram, profile_name);

    run_command(&perf_command, &project_directory, false)
        .await
        .map_err(|error| format!("Failed to profile project: {error}"))?;
    run_command(&perf_convert_command, &project_directory, false)
        .await
        .map_err(|error| format!("Failed to convert profile: {error}"))?;
    // TODO: Delete .perf.data

    Ok(())
}

// TODO: Handle killing the subprocess when the parent dies
// TODO: Don't return a buffer of stdout, instead take a callback to process a stream?
async fn run_command(
    command_text: &str,
    current_working_directory: &Path,
    get_stdout: bool,
) -> Result<Vec<u8>, GError> {
    let subprocess = SubprocessLauncher::new(if get_stdout {
        SubprocessFlags::STDOUT_PIPE
    } else {
        SubprocessFlags::NONE
    });
    subprocess.set_cwd(current_working_directory);

    let mut command = Vec::new();
    let mut parsing_flags = true;
    for x in command_text.split(" ") {
        match x.split_once("=") {
            Some((var, value)) if parsing_flags => subprocess.setenv(var, value, true),
            _ => {
                command.push(OsStr::new(x));
                parsing_flags = false;
            }
        }
    }

    let subprocess = subprocess.spawn(&command)?;
    let stdout = if get_stdout {
        read_all_from(subprocess.stdout_pipe().unwrap()).await?
    } else {
        Vec::new()
    };
    subprocess.wait_check_future().await?;

    Ok(stdout)
}

async fn read_all_from(input_stream: InputStream) -> Result<Vec<u8>, GError> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let (bytes, bytes_read) = input_stream
            .read_future(buffer, PRIORITY_DEFAULT)
            .await
            .map_err(|(_, error)| error)?;

        output.extend_from_slice(&bytes[..bytes_read]);
        buffer = bytes;

        if bytes_read == 0 {
            input_stream.close_future(PRIORITY_DEFAULT).await?;
            return Ok(output);
        }
    }
}
