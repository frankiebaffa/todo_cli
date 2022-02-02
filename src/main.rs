use {
    clap::{ Parser, Subcommand, },
    crossterm::{
        execute, terminal::{ Clear, ClearType, },
    },
    std::{
        fmt::{ Display, Formatter, Error as FormatError, },
        fs::File,
        io::{ Error as IOError, Read, stdout, },
        path::PathBuf,
        thread::sleep as thread_sleep,
        time::{ Duration, Instant, },
    },
    todo_core::{
        Container, ItemStatus, GetPath, ExitCode, PathExitCondition, ItemAction,
        ItemActor, ItemType, PrintWhich,
    },
};
#[derive(Parser, Clone)]
struct AddArgs {
    #[clap()]
    item_nest_location: Vec<usize>,
    #[clap(short='m', long)]
    item_message: String,
    #[clap(short='t', long, default_value_t = ItemType::Todo)]
    item_type: ItemType,
}
#[derive(Parser, Clone)]
struct CheckArgs {
    #[clap()]
    item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
struct DisableArgs {
    #[clap()]
    item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
struct EditArgs {
    #[clap()]
    item_location: Vec<usize>,
    #[clap(short='m', long)]
    item_message: String,
}
#[derive(Parser, Clone)]
struct HideArgs {
    #[clap()]
    item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
struct MoveArgs {
    #[clap()]
    item_location: Vec<usize>,
    #[clap(short='o', long)]
    output_location: Vec<usize>,
}
#[derive(Parser, Clone)]
struct ShowArgs {
    #[clap(short='p', long, default_value_t = PrintWhich::All)]
    print_which: PrintWhich,
    #[clap(short='s', long)]
    status: bool,
    #[clap(long)]
    plain: bool,
    #[clap(short, long)]
    level: Option<usize>,
    #[clap(long="display-hidden")]
    display_hidden: bool,
}
#[derive(Parser, Clone)]
struct RemoveArgs {
    #[clap()]
    item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
struct UncheckArgs {
    #[clap()]
    item_location: Vec<usize>,
}
#[derive(Parser, Clone)]
struct UnhideArgs {
    #[clap()]
    item_location: Vec<usize>,
}
#[derive(Subcommand, Clone)]
#[clap(about, version, author)]
enum Mode {
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
    Monitor,
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
            Mode::Monitor => fmt.write_str("Monitor"),
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
            &mut Mode::Monitor | &mut Mode::New | &mut Mode::Show(_) => {},
        }
    }
}
struct Ctx {
    args: Args,
    buffer: String,
    path: PathBuf,
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
    fn init() -> Result<Self, ExitCode> {
        let mut args = Args::parse();
        args.reverse_coordinates();
        let buffer = String::new();
        let path = PathBuf::new();
        let mut ctx = Self { args, buffer, path };
        ctx.construct_path();
        Ok(ctx)
    }
    fn check_path(&mut self, condition: PathExitCondition) -> Result<(), ExitCode> {
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
                    return Err(ExitCode::FileDoesNotExist(self.path.clone()));
                } else {
                    return Ok(());
                }
            },
            PathExitCondition::Ignore => return Ok(()),
        };
    }
    fn flush(&mut self, code: &ExitCode) {
        if !self.buffer.is_empty() {
            println!("Exited with code: {}", code);
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
struct Args {
    // Options
    // Make the list_path arg require either a string passed or the TODO_LIST env var
    /// The relative or absolute path to the list (w/o file extension)
    #[clap(short='l', long="list-path", default_value_t = String::new(), parse(try_from_str = safe_get_list))]
    list_path: String,
    // Modes
    /// The program action to take
    #[clap(subcommand)]
    mode: Mode,
}
impl Args {
    fn reverse_coordinates(&mut self) {
        self.mode.reverse_coordinates();
    }
}
fn safe_exit(ctx: &mut Ctx, e: ExitCode) -> ! {
    ctx.flush(&e);
    std::process::exit(e.into());
}
fn sleep_til(start: Instant) {
    if (Instant::now() - start).as_millis().lt(&1000) {
        thread_sleep(Duration::from_millis(
            (1000 - (Instant::now() - start).as_millis()) as u64
        ));
    }
}
fn main() -> Result<(), IOError> {
    let mut ctx = Ctx::init().unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(e.into());
    });
    // create new list
    match ctx.args.mode.clone() {
        Mode::Monitor => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut hash = String::new();
            loop {
                let start = Instant::now();
                {
                    let mut container = Container::load(&mut ctx)
                        .unwrap_or_else(|e| safe_exit(&mut ctx, e));
                    let mut f_contents = String::new();
                    {
                        let mut file = File::open(&container.path).unwrap_or_else(|_| {
                            safe_exit(
                                &mut ctx,
                                ExitCode::FailedToOpen(container.path.clone())
                            )
                        });
                        file.read_to_string(&mut f_contents).unwrap_or_else(|_| {
                            safe_exit(
                                &mut ctx,
                                ExitCode::FailedToRead(container.path.clone())
                            )
                        });
                    }
                    let new_hash = format!("{:x}", md5::compute(&f_contents.as_bytes()));
                    if !hash.eq(&new_hash) {
                        hash = new_hash;
                    } else {
                        sleep_til(start);
                        continue;
                    }
                    execute!(stdout(), Clear(ClearType::All))?;
                    let mut output = String::new();
                    container.print(
                        &mut output, &PrintWhich::All, false, None, false
                    )?;
                    output.push_str("\r\n");
                    println!("{}", output);
                }
                // clear
                sleep_til(start);
            }
        },
        Mode::New => {
            ctx.check_path(PathExitCondition::Exists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::create(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        },
        Mode::Add(mut args) => {
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
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterStatus(ItemStatus::Complete),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        },
        Mode::Disable(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterStatus(ItemStatus::Disabled),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        },
        Mode::Hide(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterHidden(true),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        }
        Mode::Uncheck(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterStatus(ItemStatus::Incomplete),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        },
        Mode::Unhide(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::AlterHidden(false),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        }
        Mode::Edit(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::Edit(args.item_message),
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
        },
        Mode::Remove(mut args) => {
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            container.act_on_item_at(
                &mut args.item_location,
                ItemAction::Remove,
            );
            container.save().unwrap_or_else(|e| safe_exit(&mut ctx, e));
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
        },
        Mode::Show(args) => {
            let print_which = args.print_which;
            ctx.check_path(PathExitCondition::NotExists)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut container = Container::load(&mut ctx)
                .unwrap_or_else(|e| safe_exit(&mut ctx, e));
            let mut output = String::new();
            container.print(
                &mut output, &print_which, args.plain, args.level,
                args.display_hidden,
            )?;
            output.push_str("\r\n");
            println!("{}", output);
        },
    }
    safe_exit(&mut ctx, ExitCode::Success);
}
