use std::{fs, path::PathBuf, sync::Arc};

use thiserror::Error;
use tauri::Manager;

pub mod commands;
pub mod models;
pub mod repositories;
pub mod services;

use repositories::TaskRepository;
use services::TaskService;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Falha no banco de dados: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Falha ao inicializar a aplicação: {0}")]
    Initialization(String),
    #[error("O acesso interno ao banco de dados ficou indisponível.")]
    LockPoisoned,
}

impl AppError {
    pub(crate) fn user_message(&self) -> String {
        match self {
            Self::Validation(message) | Self::NotFound(message) => message.clone(),
            Self::Database(_) => "Não foi possível acessar o banco de dados.".to_string(),
            Self::Initialization(_) => {
                "Não foi possível inicializar a persistência da aplicação.".to_string()
            }
            Self::LockPoisoned => {
                "Não foi possível acessar a persistência neste momento.".to_string()
            }
        }
    }
}

fn database_path(app_data_dir: PathBuf) -> PathBuf {
    app_data_dir.join("todo.sqlite3")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| {
                    AppError::Initialization(format!(
                        "Não foi possível resolver o diretório de dados: {error}"
                    ))
                })?;

            fs::create_dir_all(&app_data_dir).map_err(|error| {
                AppError::Initialization(format!(
                    "Não foi possível criar o diretório de dados: {error}"
                ))
            })?;

            let repository = TaskRepository::open(database_path(app_data_dir))?;
            let service = TaskService::new(Arc::new(repository));
            app.manage(service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::task_commands::list_tasks,
            commands::task_commands::create_task,
            commands::task_commands::update_task,
            commands::task_commands::delete_task,
            commands::task_commands::complete_task,
            commands::task_commands::reopen_task
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("Falha ao executar a aplicação Tauri: {error}");
        });
}
