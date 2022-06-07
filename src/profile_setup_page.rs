use crate::profile_page_view::{ProfilePageView, ProfilePageViewState};
use adw::subclass::prelude::BinImpl;
use adw::{ActionRow, Bin, Clamp, EntryRow, Toast, ToastOverlay};
use gio::prelude::InputStreamExtManual;
use gio::traits::{FileExt, InputStreamExt};
use gio::{InputStream, SubprocessFlags, SubprocessLauncher};
use glib::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use glib::subclass::InitializingObject;
use glib::{object_subclass, BoolError, Cast, DateTime, Error as GError, Object, PRIORITY_DEFAULT};
use gtk::prelude::{InitializingWidgetExt, NativeDialogExtManual};
use gtk::subclass::prelude::{
    CompositeTemplateCallbacksClass, CompositeTemplateClass, WidgetClassSubclassExt, WidgetImpl,
};
use gtk::traits::{EditableExt, FileChooserExt, NativeDialogExt, WidgetExt};
use gtk::{
    template_callbacks, Accessible, Buildable, Button, CompositeTemplate, ConstraintTarget,
    FileChooserAction, FileChooserNative, FileFilter, Label, ResponseType, TemplateChild, Widget,
    Window,
};
use serde_json::{Deserializer, Value};
use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

glib::wrapper! {
    pub struct ProfileSetupPage(ObjectSubclass<ProfileSetupPagePrivate>)
    @extends Bin, Widget,
    @implements Accessible, Buildable, ConstraintTarget;
}

impl ProfileSetupPage {
    pub fn new() -> Self {
        Object::new(&[]).unwrap()
    }
}

// ------------------------------------------------------------------------------

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/jms55/WhatTheFn/ui/profile_setup_page.ui")]
pub struct ProfileSetupPagePrivate {
    #[template_child]
    page: TemplateChild<Clamp>,
    #[template_child]
    error_toast: TemplateChild<ToastOverlay>,
    #[template_child]
    cargo_toml_row: TemplateChild<ActionRow>,
    #[template_child]
    cargo_toml_path: TemplateChild<Label>,
    #[template_child]
    start_profiling_button: TemplateChild<Button>,
    #[template_child]
    cargo_build_entry: TemplateChild<EntryRow>,
    #[template_child]
    perf_entry: TemplateChild<EntryRow>,
    #[template_child]
    program_arguments_entry: TemplateChild<EntryRow>,
}

#[template_callbacks]
impl ProfileSetupPagePrivate {
    // When a Cargo.toml is chosen, set the path label, update the page view state, and enable the start profiling button
    #[template_callback]
    async fn select_cargo_toml(&self) {
        let cargo_toml_path = self
            .get_file_from_user("Select a Cargo.toml", "Cargo.toml")
            .await;
        if let Some(mut cargo_toml_path) = cargo_toml_path {
            cargo_toml_path.pop();
            if let Some(path_str) = cargo_toml_path.to_str() {
                self.cargo_toml_row.add_css_class("success-row");
                self.start_profiling_button.add_css_class("green-button");

                let profile_name = cargo_toml_path.file_name().unwrap().to_str().unwrap();
                self.page_view()
                    .set_data(ProfilePageViewState::Setup, profile_name);
                self.cargo_toml_path.set_label(path_str);
                self.start_profiling_button.set_sensitive(true);
            } else {
                self.add_error_toast("Error: Cargo.toml path is not valid UTF-8");
            }
        }
    }

    // Build project, run perf, convert to .perf.json, and then switch the page view to ProfilePage
    #[template_callback]
    async fn start_profiling(&self) {
        let prefix = match get_prefix() {
            Ok(prefix) => prefix,
            Err(error) => {
                self.add_error_toast(&format!("Failed to get current datetime: {error}"));
                return;
            }
        };

        self.page.set_sensitive(false);

        let project_directory = PathBuf::from(&self.cargo_toml_path.label());
        let profile_name = project_directory.file_name().unwrap().to_str().unwrap();
        let perf_file = format!("{profile_name}:{prefix}.perf.data");
        let profile = format!("{profile_name}:{prefix}.perf.json");
        let cargo_build_command = self.cargo_build_entry.text();
        let perf_command = self
            .perf_entry
            .text()
            .replace("${TMP_FILE}", &perf_file)
            .replace("${PROGRAM_ARGUMENTS}", &self.program_arguments_entry.text());
        let perf_convert_command =
            format!("perf data convert --input {perf_file} --to-json {profile}");
        let page_view = self.page_view();

        let profiling_result = build_and_profile(
            &project_directory,
            &cargo_build_command,
            perf_command,
            &perf_convert_command,
            &page_view,
        )
        .await;

        match profiling_result {
            Ok(_) => {
                let mut profile_path = project_directory;
                profile_path.push(profile);
                page_view.switch_to_profile_page(&profile_path);
            }
            Err(error) => {
                let profile_name = project_directory.file_name().unwrap().to_str().unwrap();
                page_view.set_data(ProfilePageViewState::Setup, profile_name);
                self.page.set_sensitive(true);
                self.add_error_toast(&format!("{error}"));
            }
        }
    }

    // When a .perf.json is chosen, switch the page view to ProfilePage
    #[template_callback]
    async fn open_existing_profile(&self) {
        let profile_path = self
            .get_file_from_user("Select a .perf.json", "*.perf.json")
            .await;
        if let Some(profile_path) = profile_path {
            self.page_view().switch_to_profile_page(&profile_path);
        }
    }

    async fn get_file_from_user(
        &self,
        file_chooser_title: &str,
        file_filter_pattern: &str,
    ) -> Option<PathBuf> {
        let parent_window = self
            .instance()
            .root()
            .map(Cast::downcast::<Window>)
            .map(Result::ok)
            .flatten();

        let file_chooser = FileChooserNative::new(
            Some(file_chooser_title),
            parent_window.as_ref(),
            FileChooserAction::Open,
            None,
            None,
        );
        file_chooser.set_modal(true);

        let file_filter = FileFilter::new();
        file_filter.set_name(Some(file_filter_pattern));
        file_filter.add_pattern(file_filter_pattern);
        file_chooser.add_filter(&file_filter);

        let response = file_chooser.run_future().await;
        if response == ResponseType::Accept {
            file_chooser.file().unwrap().path()
        } else {
            None
        }
    }

    fn add_error_toast(&self, error_message: &str) {
        self.error_toast.add_toast(&Toast::new(error_message));
    }

    fn page_view(&self) -> ProfilePageView {
        self.instance()
            .parent()
            .unwrap()
            .downcast::<ProfilePageView>()
            .unwrap()
    }
}

async fn build_and_profile(
    project_directory: &Path,
    cargo_build_command: &str,
    mut perf_command: String,
    perf_convert_command: &str,
    page_view: &ProfilePageView,
) -> Result<(), Box<dyn Error>> {
    // Set page view state to compiling
    let profile_name = project_directory.file_name().unwrap().to_str().unwrap();
    page_view.set_data(ProfilePageViewState::SetupCompilingProgram, profile_name);

    // Run cargo build
    let cargo_build_output = run_command(cargo_build_command, project_directory, true)
        .await
        .map_err(|error| format!("Failed to compile project: {error}"))?;

    // Determine path of binary cargo built by parsing cargo build's output
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
    let program_path = program_path.ok_or_else::<Box<dyn Error>, _>(|| {
        "Failed to find program path in compiler output".into()
    })?;

    // Set page view state to profiling
    let profile_name = program_path.file_name().unwrap().to_str().unwrap();
    page_view.set_data(ProfilePageViewState::SetupProfilingProgram, profile_name);

    // Run perf and then convert profile to json
    run_command(&perf_command, project_directory, false)
        .await
        .map_err(|error| format!("Failed to profile project: {error}"))?;
    run_command(perf_convert_command, project_directory, false)
        .await
        .map_err(|error| format!("Failed to convert profile: {error}"))?;
    // TODO: Delete .perf.data?

    Ok(())
}

// TODO: Handle killing when the window is closed
// TODO: Don't return a buffer of stdout, instead take a callback to process a stream?
async fn run_command(
    command_text: &str,
    current_working_directory: &Path,
    get_stdout: bool,
) -> Result<Vec<u8>, GError> {
    // Create the subprocess
    let subprocess = SubprocessLauncher::new(if get_stdout {
        SubprocessFlags::STDOUT_PIPE
    } else {
        SubprocessFlags::NONE
    });
    subprocess.set_cwd(current_working_directory);

    // Parse command text into environment variables + command + arguments
    let mut command = Vec::new();
    let mut parsing_flags = true;
    for x in command_text.split(' ') {
        match x.split_once("=") {
            Some((var, value)) if parsing_flags => subprocess.setenv(var, value, true),
            _ => {
                command.push(OsStr::new(x));
                parsing_flags = false;
            }
        }
    }

    // Launch the subprocess
    let subprocess = subprocess.spawn(&command)?;

    // Collect the process's stdout if asked to
    let stdout = if get_stdout {
        read_all_from(subprocess.stdout_pipe().unwrap()).await?
    } else {
        Vec::new()
    };

    // Wait for the process to end, and ensure it exited successfully
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

fn get_prefix() -> Result<String, BoolError> {
    Ok(DateTime::now_utc()?.format("%F:%T")?.to_string())
}

#[object_subclass]
impl ObjectSubclass for ProfileSetupPagePrivate {
    const NAME: &'static str = "WtfProfileSetupPage";
    type Type = ProfileSetupPage;
    type ParentType = Bin;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(this: &InitializingObject<Self>) {
        this.init_template();
    }
}

impl ObjectImpl for ProfileSetupPagePrivate {}
impl WidgetImpl for ProfileSetupPagePrivate {}
impl BinImpl for ProfileSetupPagePrivate {}
