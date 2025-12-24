mod all;
mod create_task;
mod edit_task;
mod footer;
mod kanban;
mod log;
mod main;
mod shortcuts;
mod status;
mod task_detail;
mod top_list;
mod welcome;

use self::log::log;
use super::*;
use footer::footer;
use main::main;
use shortcuts::shortcuts;
use status::status;
use top_list::top_list;

pub use all::all as render;
