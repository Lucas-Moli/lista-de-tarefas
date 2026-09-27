use tauri::State;

use crate::{models::Task, services::TaskService};

#[tauri::command]
pub fn list_tasks(state: State<'_, TaskService>) -> Result<Vec<Task>, String> {
    state.list_tasks().map_err(|error| error.user_message())
}

#[tauri::command]
pub fn create_task(
    state: State<'_, TaskService>,
    title: String,
    description: String,
) -> Result<Task, String> {
    state
        .create_task(title, description)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn update_task(
    state: State<'_, TaskService>,
    id: i64,
    title: String,
    description: String,
) -> Result<Task, String> {
    state
        .update_task(id, title, description)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn delete_task(state: State<'_, TaskService>, id: i64) -> Result<(), String> {
    state
        .delete_task(id)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn complete_task(state: State<'_, TaskService>, id: i64) -> Result<Task, String> {
    state
        .complete_task(id)
        .map_err(|error| error.user_message())
}

#[tauri::command]
pub fn reopen_task(state: State<'_, TaskService>, id: i64) -> Result<Task, String> {
    state
        .reopen_task(id)
        .map_err(|error| error.user_message())
}
