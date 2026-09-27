use std::{path::Path, sync::{Mutex, MutexGuard}};

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::{models::Task, AppError};

pub trait TaskRepositoryPort: Send + Sync {
    fn insert(
        &self,
        title: &str,
        description: &str,
        created_at: &str,
        updated_at: &str,
    ) -> Result<Task, AppError>;

    fn find_by_id(&self, id: i64) -> Result<Option<Task>, AppError>;

    fn list_all(&self) -> Result<Vec<Task>, AppError>;

    fn update(
        &self,
        id: i64,
        title: &str,
        description: &str,
        updated_at: &str,
    ) -> Result<Option<Task>, AppError>;

    fn delete(&self, id: i64) -> Result<bool, AppError>;

    fn set_completed(
        &self,
        id: i64,
        completed: bool,
        updated_at: &str,
    ) -> Result<Option<Task>, AppError>;
}

pub struct TaskRepository {
    connection: Mutex<Connection>,
}

impl TaskRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let connection = Connection::open(path)?;
        let repository = Self {
            connection: Mutex::new(connection),
        };

        repository.initialize()?;
        Ok(repository)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, AppError> {
        let connection = Connection::open_in_memory()?;
        let repository = Self {
            connection: Mutex::new(connection),
        };

        repository.initialize()?;
        Ok(repository)
    }

    fn initialize(&self) -> Result<(), AppError> {
        let connection = self.lock_connection()?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;

             CREATE TABLE IF NOT EXISTS tasks (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 title TEXT NOT NULL,
                 description TEXT NOT NULL DEFAULT '',
                 completed INTEGER NOT NULL DEFAULT 0,
                 created_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );",
        )?;

        Ok(())
    }

    fn lock_connection(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.connection.lock().map_err(|_| AppError::LockPoisoned)
    }

    fn row_to_task(row: &Row<'_>) -> rusqlite::Result<Task> {
        let completed: i64 = row.get(3)?;

        Ok(Task::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            completed != 0,
            row.get(4)?,
            row.get(5)?,
        ))
    }

    fn query_by_id(connection: &Connection, id: i64) -> rusqlite::Result<Option<Task>> {
        connection
            .query_row(
                "SELECT id, title, description, completed, created_at, updated_at
                 FROM tasks
                 WHERE id = ?1",
                params![id],
                Self::row_to_task,
            )
            .optional()
    }
}

impl TaskRepositoryPort for TaskRepository {
    fn insert(
        &self,
        title: &str,
        description: &str,
        created_at: &str,
        updated_at: &str,
    ) -> Result<Task, AppError> {
        let connection = self.lock_connection()?;
        connection.execute(
            "INSERT INTO tasks (title, description, completed, created_at, updated_at)
             VALUES (?1, ?2, 0, ?3, ?4)",
            params![title, description, created_at, updated_at],
        )?;

        let id = connection.last_insert_rowid();

        Ok(Task::new(
            id,
            title.to_string(),
            description.to_string(),
            false,
            created_at.to_string(),
            updated_at.to_string(),
        ))
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Task>, AppError> {
        let connection = self.lock_connection()?;
        Ok(Self::query_by_id(&connection, id)?)
    }

    fn list_all(&self) -> Result<Vec<Task>, AppError> {
        let connection = self.lock_connection()?;
        let mut statement = connection.prepare(
            "SELECT id, title, description, completed, created_at, updated_at
             FROM tasks
             ORDER BY created_at DESC, id DESC",
        )?;

        let rows = statement.query_map([], Self::row_to_task)?;
        let mut tasks = Vec::new();

        for row in rows {
            tasks.push(row?);
        }

        Ok(tasks)
    }

    fn update(
        &self,
        id: i64,
        title: &str,
        description: &str,
        updated_at: &str,
    ) -> Result<Option<Task>, AppError> {
        let connection = self.lock_connection()?;
        let changed = connection.execute(
            "UPDATE tasks
             SET title = ?1, description = ?2, updated_at = ?3
             WHERE id = ?4",
            params![title, description, updated_at, id],
        )?;

        if changed == 0 {
            return Ok(None);
        }

        Ok(Self::query_by_id(&connection, id)?)
    }

    fn delete(&self, id: i64) -> Result<bool, AppError> {
        let connection = self.lock_connection()?;
        let changed = connection.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;

        Ok(changed > 0)
    }

    fn set_completed(
        &self,
        id: i64,
        completed: bool,
        updated_at: &str,
    ) -> Result<Option<Task>, AppError> {
        let connection = self.lock_connection()?;
        let changed = connection.execute(
            "UPDATE tasks
             SET completed = ?1, updated_at = ?2
             WHERE id = ?3",
            params![if completed { 1_i64 } else { 0_i64 }, updated_at, id],
        )?;

        if changed == 0 {
            return Ok(None);
        }

        Ok(Self::query_by_id(&connection, id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{TaskRepository, TaskRepositoryPort};

    const CREATED_AT: &str = "2026-09-25T18:00:00.000Z";
    const UPDATED_AT: &str = "2026-09-25T18:00:00.000Z";

    fn repository() -> TaskRepository {
        TaskRepository::in_memory().expect("repository de teste deve ser criado")
    }

    fn insert_sample(repository: &TaskRepository) -> i64 {
        let task = repository
            .insert("Estudar Rust", "Revisar structs", CREATED_AT, UPDATED_AT)
            .expect("inserção deve funcionar");

        task.id()
    }

    #[test]
    fn insere_tarefa() {
        let repository = repository();

        let task = repository
            .insert("Estudar Rust", "Revisar structs", CREATED_AT, UPDATED_AT)
            .expect("inserção deve funcionar");

        assert!(task.id() > 0);
        assert_eq!(task.title(), "Estudar Rust");
        assert!(!task.is_completed());
    }

    #[test]
    fn busca_tarefa_por_id() {
        let repository = repository();
        let id = insert_sample(&repository);

        let task = repository
            .find_by_id(id)
            .expect("busca deve funcionar")
            .expect("tarefa deve existir");

        assert_eq!(task.id(), id);
        assert_eq!(task.description(), "Revisar structs");
    }

    #[test]
    fn retorna_none_para_id_inexistente() {
        let repository = repository();

        let task = repository
            .find_by_id(999_999)
            .expect("busca deve funcionar");

        assert!(task.is_none());
    }

    #[test]
    fn atualiza_tarefa() {
        let repository = repository();
        let id = insert_sample(&repository);

        let task = repository
            .update(id, "Estudar Tauri", "Revisar commands", UPDATED_AT)
            .expect("atualização deve funcionar")
            .expect("tarefa deve existir");

        assert_eq!(task.title(), "Estudar Tauri");
        assert_eq!(task.description(), "Revisar commands");
        assert_eq!(task.updated_at(), UPDATED_AT);
    }

    #[test]
    fn atualiza_tarefa_inexistente() {
        let repository = repository();

        let result = repository
            .update(999_999, "Título", "Descrição", UPDATED_AT)
            .expect("consulta deve funcionar");

        assert!(result.is_none());
    }

    #[test]
    fn exclui_tarefa() {
        let repository = repository();
        let id = insert_sample(&repository);

        let deleted = repository.delete(id).expect("exclusão deve funcionar");
        let remaining = repository
            .find_by_id(id)
            .expect("busca deve funcionar");

        assert!(deleted);
        assert!(remaining.is_none());
    }

    #[test]
    fn exclusao_de_id_inexistente_retorna_false() {
        let repository = repository();

        let deleted = repository.delete(999_999).expect("exclusão deve funcionar");

        assert!(!deleted);
    }

    #[test]
    fn altera_status_da_tarefa() {
        let repository = repository();
        let id = insert_sample(&repository);

        let task = repository
            .set_completed(id, true, UPDATED_AT)
            .expect("alteração de status deve funcionar")
            .expect("tarefa deve existir");

        assert!(task.is_completed());
        assert_eq!(task.updated_at(), UPDATED_AT);
    }

    #[test]
    fn lista_todas_as_tarefas() {
        let repository = repository();
        insert_sample(&repository);
        repository
            .insert("Estudar SQLite", "Revisar SQL", "2026-09-25T19:00:00.000Z", "2026-09-25T19:00:00.000Z")
            .expect("segunda inserção deve funcionar");

        let tasks = repository.list_all().expect("listagem deve funcionar");

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].title(), "Estudar SQLite");
    }
}
