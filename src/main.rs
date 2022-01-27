use {
    clap::{
        Parser,
        Subcommand,
    },
    std::{
        fmt::{
            Display,
            Formatter,
            Error as FormatError,
        },
        path::PathBuf,
    },
    todo_core::{
        Container,
        ItemStatus,
        GetPath,
        ExitCode,
        PathExitCondition,
        get_printable_coords,
        ItemAction,
        ItemActor,
        ItemType,
        PrintWhich,
    },
};
#[derive(Parser, Clone)]
pub struct AddArgs {
    #[clap()]
    pub item_nest_location: Vec<usize>,
    #[clap(short='m', long)]
    pub item_message: String,
    #[clap(short='t', long, default_value_t = ItemType::Todo)]
    pub item_type: ItemType,
}
#[derive(Parser, Clone)]
pub struct CheckArgs {
    #[clap()]
    pub item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
pub struct DisableArgs {
    #[clap()]
    pub item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
pub struct EditArgs {
    #[clap()]
    pub item_location: Vec<usize>,
    #[clap(short='m', long)]
    pub item_message: String,
}
#[derive(Parser, Clone)]
pub struct HideArgs {
    #[clap()]
    pub item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
pub struct MoveArgs {
    #[clap()]
    pub item_location: Vec<usize>,
    #[clap(short='o', long)]
    pub output_location: Vec<usize>,
}
#[derive(Parser, Clone)]
pub struct ShowArgs {
    #[clap(short='p', long, default_value_t = PrintWhich::All)]
    pub print_which: PrintWhich,
    #[clap(short='s', long)]
    pub status: bool,
    #[clap(long)]
    pub plain: bool,
    #[clap(short, long)]
    pub level: Option<usize>,
    #[clap(long="display-hidden")]
    pub display_hidden: bool,
}
#[derive(Parser, Clone)]
pub struct RemoveArgs {
    #[clap()]
    pub item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
pub struct UncheckArgs {
    #[clap()]
    pub item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
pub struct UnhideArgs {
    #[clap()]
    pub item_location: Vec<usize>,
}
#[derive(Subcommand, Clone)]
#[clap(about, version, author)]
pub enum Mode {
    /// Add a new list-item
    Add(AddArgs),
    /// Check-off an existing list-item
    Check(CheckArgs),
    /// Disable an existing list-item
    Disable(DisableArgs),
    /// Edit the item-text of an existing list-item
    Edit(EditArgs),
    /// Hide the list item
    Hide(HideArgs),
    /// Move an existing list-item to a new location
    Move(MoveArgs),
    /// Create a new list
    New,
    /// Show an existing list
    Show(ShowArgs),
    /// Remove an existing list-item
    Remove(RemoveArgs),
    /// Uncheck an existing list-item
    Uncheck(UncheckArgs),
    /// Hide the list item
    Unhide(UnhideArgs),
}
impl Display for Mode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match self {
            Mode::Add(_) => fmt.write_str("Add"),
            Mode::Check(_) => fmt.write_str("Check"),
            Mode::Disable(_) => fmt.write_str("Disable"),
            Mode::Edit(_) => fmt.write_str("Edit"),
            Mode::Hide(_) => fmt.write_str("Hide"),
            Mode::Move(_) => fmt.write_str("Move"),
            Mode::New => fmt.write_str("New"),
            Mode::Show(_) => fmt.write_str("Show"),
            Mode::Remove(_) => fmt.write_str("Remove"),
            Mode::Uncheck(_) => fmt.write_str("Uncheck"),
            Mode::Unhide(_) => fmt.write_str("Unhide"),
        }
    }
}
impl Mode {
    fn reverse_coordinates(&mut self) {
        match self {
            &mut Mode::Add(ref mut mode_args) => {
                mode_args.item_nest_location.reverse();
            },
            &mut Mode::Check(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Disable(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Edit(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Hide(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Move(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Remove(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Uncheck(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::Unhide(ref mut mode_args) => {
                mode_args.item_location.reverse();
            },
            &mut Mode::New | &mut Mode::Show(_) => {},
        }
    }
}
pub struct Ctx {
    pub args: Args,
    pub buffer: String,
    pub path: PathBuf,
}
impl<'ctx> Ctx {
    fn construct_path(&mut self) {
        let tmp_path = PathBuf::from(format!("{}", &self.args.list_path));
        match tmp_path.extension() {
            Some(ext) => {
                if !ext.eq("json") {
                    self.path.push(format!("{}.json", &self.args.list_path));
                } else {
                    self.path.push(format!("{}", &self.args.list_path));
                }
            },
            None => self.path.push(format!("{}.json", &self.args.list_path)),
        }
    }
    pub fn init() -> Result<Self, ExitCode> {
        let mut args = Args::parse();
        args.reverse_coordinates();
        let buffer = String::new();
        let path = PathBuf::new();
        let mut ctx = Self { args, buffer, path, };
        ctx.construct_path();
        Ok(ctx)
    }
    pub fn check_path(&mut self, condition: PathExitCondition) -> Result<(), ExitCode> {
        match condition {
            PathExitCondition::Exists => {
                if self.path.exists() {
                    return Err(ExitCode::FileExists(self.path.clone()));
                } else {
                    return Ok(());
                }
            },
            PathExitCondition::NotExists => {
                if !self.path.exists() {
                    self.v_print(
                        format!(
                            "File at \"{}\" does not exist",
                            &self.path.to_str().unwrap()
                        )
                    );
                    return Err(ExitCode::FileDoesNotExist(self.path.clone()));
                } else {
                    return Ok(());
                }
            },
            PathExitCondition::Ignore => return Ok(()),
        };
    }
    pub fn print(&mut self, msg: impl AsRef<str>) {
        if !self.buffer.is_empty() {
            self.buffer.push_str("\n");
        }
        self.buffer.push_str(&format!("{}", msg.as_ref()));
    }
    pub fn q_print(&mut self, msg: impl AsRef<str>) {
        if !self.args.quiet {
            self.print(msg);
        }
    }
    pub fn v_print(&mut self, msg: impl AsRef<str>) {
        if self.args.verbose {
            self.q_print(msg);
        }
    }
    pub fn flush(&mut self, code: &ExitCode) {
        self.v_print(format!("{}", code));
        if !self.buffer.is_empty() {
            println!("{}", self.buffer);
        }
    }
}
impl GetPath for Ctx {
    fn get_path(&self) -> &PathBuf {
        return &self.path;
    }
    fn get_path_mut(&mut self) -> &mut PathBuf {
        return &mut self.path;
    }
}
fn safe_get_list(arg: &str) -> Result<String, String> {
    if arg.is_empty() {
        match std::env::var("TODO_LIST") {
            Ok(s) => Ok(s),
            Err(_) => return Err(ExitCode::NoEnvVar.to_string()),
        }
    } else {
        Ok(arg.to_string())
    }
}
/// A todo list manager
#[derive(Parser, Clone)]
#[clap(about, version, author)]
pub struct Args {
    // Options
    // Make the list_path arg require either a string passed or the TODO_LIST env var
    /// The relative or absolute path to the list (w/o file extension)
    #[clap(short='l', long="list-path", default_value_t = String::new(), parse(try_from_str = safe_get_list))]
    pub list_path: String,
    // Flags
    /// Silences all messages (overrides verbose flag)
    #[clap(short='q', long)]
    pub quiet: bool,
    /// Prints verbose messages during output
    #[clap(short='v', long)]
    pub verbose: bool,
    // Modes
    /// The program action to take
    #[clap(subcommand)]
    pub mode: Mode,
}
impl Args {
    pub fn reverse_coordinates(&mut self) {
        self.mode.reverse_coordinates();
    }
}
fn safe_exit(ctx: &mut Ctx, e: ExitCode) -> ! {
    ctx.flush(&e);
    std::process::exit(e.into());
}
fn main() {
    let mut ctx = Ctx::init().unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(e.into());
    });
    ctx.v_print("==ARGS==");
    ctx.v_print(&format!("Mode: {}", &ctx.args.mode));
    ctx.v_print(&format!("Path: {}", &ctx.args.list_path));
    ctx.v_print("==/ARGS==");
    ctx.v_print("==RUN==");
    // create new list
    match ctx.args.mode.clone() {
        Mode::New => {
            ctx.v_print(format!("Creating new list \"{}\"", &ctx.args.list_path));
            ctx.check_path(PathExitCondition::Exists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::create(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Created new list \"{}\"", ctx.path.clone().to_str().unwrap()
            ));
        },
        Mode::Add(mut args) => {
            ctx.v_print(
                format!(
                    "Creating new list-item in list \"{}\" with item-message \"{}\"",
                    &ctx.args.list_path,
                    &args.item_message,
                )
            );
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_nest_location,
                ItemAction::Add(args.item_type, args.item_message),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        },
        Mode::Check(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Checking item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location),
                &ctx.args.list_path
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterStatus(ItemStatus::Complete),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Checked from list \"{}\"", &ctx.args.list_path));
        },
        Mode::Disable(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Disabling item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location),
                &ctx.args.list_path
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterStatus(ItemStatus::Disabled),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Checked from list \"{}\"", &ctx.args.list_path));
        },
        Mode::Hide(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Hiding item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location),
                &ctx.args.list_path
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterHidden(true),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Hid from list \"{}\"", &ctx.args.list_path));
        }
        Mode::Uncheck(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Unchecking item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location), ctx.args.list_path
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterStatus(ItemStatus::Incomplete),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Unchecked from list \"{}\"", ctx.args.list_path));
        },
        Mode::Unhide(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Unhiding item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location),
                &ctx.args.list_path
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterHidden(false),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Unhid from list \"{}\"", &ctx.args.list_path));
        }
        Mode::Edit(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Editing item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location), &ctx.args.list_path
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::Edit(args.item_message),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Edited item from list \"{}\"", ctx.args.list_path));
        },
        Mode::Remove(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!(
                "Removing item \"{}\" from list \"{}\"",
                get_printable_coords(&args.item_location),
                &ctx.args.list_path,
            ));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::Remove,
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Removed item from list \"{}\"", &ctx.args.list_path));
        },
        Mode::Move(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let item = container.act_on_item_at(
                &mut args.item_location,
                ItemAction::Remove,
            ).unwrap();
            container.act_on_item_at(
                &mut args.output_location,
                ItemAction::Put(item),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print(format!("Moved item"));
        },
        Mode::Show(args) => {
            let print_which = args.print_which;
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            ctx.v_print("==FILE==");
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut content = String::new();
            container.print(
                &mut content, &print_which, args.plain, args.level,
                args.display_hidden,
            );
            ctx.print(content);
            ctx.v_print("==/FILE==");
            if args.status {
                ctx.check_path(PathExitCondition::NotExists)
                    .unwrap_or_else(|e| safe_exit(&mut ctx, e));
                ctx.v_print("==STATUS==");
                let mut content = String::new();
                let mut container = Container::load(&mut ctx)
                    .unwrap_or_else(|e| safe_exit(&mut ctx, e));
                container.status(&mut content, &print_which);
                ctx.print(content);
                ctx.v_print("==/STATUS==");
            }
        },
    }
    safe_exit(&mut ctx, ExitCode::Success);
}
