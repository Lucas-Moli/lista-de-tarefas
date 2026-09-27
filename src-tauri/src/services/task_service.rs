use std::sync::Arc;

use chrono::{SecondsFormat, Utc};

use crate::{
    models::{Task, TaskValidationError},
    repositories::TaskRepositoryPort,
    AppError,
};

pub struct TaskService {
    repository: Arc<dyn TaskRepositoryPort>,
}

impl TaskService {
    pub fn new(repository: Arc<dyn TaskRepositoryPort>) -> Self {
        Self { repository }
    }

    pub fn list_tasks(&self) -> Result<Vec<Task>, AppError> {
        self.repository.list_all()
    }

    pub fn create_task(&self, title: String, description: String) -> Result<Task, AppError> {
        let (title, description) = Self::validate_and_normalize(&title, &description)?;
        let timestamp = current_timestamp();

        self.repository
            .insert(&title, &description, &timestamp, &timestamp)
    }

    pub fn update_task(
        &self,
        id: i64,
        title: String,
        description: String,
    ) -> Result<Task, AppError> {
        Self::validate_id(id)?;
        let (title, description) = Self::validate_and_normalize(&title, &description)?;
        let timestamp = current_timestamp();

        let updated = self
            .repository
            .update(id, &title, &description, &timestamp)?;

        updated.ok_or_else(|| AppError::NotFound("A tarefa informada não existe.".to_string()))
    }

    pub fn delete_task(&self, id: i64) -> Result<(), AppError> {
        Self::validate_id(id)?;

        let deleted = self.repository.delete(id)?;

        if !deleted {
            return Err(AppError::NotFound("A tarefa informada não existe.".to_string()));
        }

        Ok(())
    }

    pub fn complete_task(&self, id: i64) -> Result<Task, AppError> {
        self.change_completion(id, true)
    }

    pub fn reopen_task(&self, id: i64) -> Result<Task, AppError> {
        self.change_completion(id, false)
    }

    fn change_completion(&self, id: i64, completed: bool) -> Result<Task, AppError> {
        Self::validate_id(id)?;

        let mut task = self
            .repository
            .find_by_id(id)?
            .ok_or_else(|| AppError::NotFound("A tarefa informada não existe.".to_string()))?;

        if completed {
            task.complete();
        } else {
            task.reopen();
        }

        let timestamp = current_timestamp();
        let updated = self
            .repository
            .set_completed(task.id(), task.is_completed(), &timestamp)?;

        updated.ok_or_else(|| AppError::NotFound("A tarefa informada não existe.".to_string()))
    }

    fn validate_and_normalize(
        title: &str,
        description: &str,
    ) -> Result<(String, String), AppError> {
        Task::normalize_and_validate(title, description).map_err(validation_error_to_app_error)
    }

    fn validate_id(id: i64) -> Result<(), AppError> {
        if id <= 0 {
            return Err(AppError::Validation("O ID da tarefa é inválido.".to_string()));
        }

        Ok(())
    }
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn validation_error_to_app_error(error: TaskValidationError) -> AppError {
    AppError::Validation(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::TaskService;
    use crate::repositories::TaskRepository;

    fn service() -> TaskService {
        let repository = TaskRepository::in_memory().expect("repository de teste deve ser criado");
        TaskService::new(Arc::new(repository))
    }

    #[test]
    fn cria_tarefa_valida() {
        let service = service();

        let task = service
            .create_task("Estudar Rust".to_string(), "Revisar impl e traits".to_string())
            .expect("criação válida deve funcionar");

        assert!(task.id() > 0);
        assert_eq!(task.title(), "Estudar Rust");
        assert_eq!(task.description(), "Revisar impl e traits");
        assert!(!task.is_completed());
    }

    #[test]
    fn rejeita_criacao_com_titulo_vazio() {
        let service = service();

        let result = service.create_task("   ".to_string(), "Descrição".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn rejeita_criacao_com_descricao_muito_longa() {
        let service = service();
        let description = "a".repeat(501);

        let result = service.create_task("Título".to_string(), description);

        assert!(result.is_err());
    }

    #[test]
    fn atualiza_tarefa_existente() {
        let service = service();
        let task = service
            .create_task("Título antigo".to_string(), "Descrição antiga".to_string())
            .expect("criação deve funcionar");

        let updated = service
            .update_task(
                task.id(),
                "Título novo".to_string(),
                "Descrição nova".to_string(),
            )
            .expect("atualização deve funcionar");

        assert_eq!(updated.id(), task.id());
        assert_eq!(updated.title(), "Título novo");
        assert_eq!(updated.description(), "Descrição nova");
    }

    #[test]
    fn falha_ao_atualizar_tarefa_inexistente() {
        let service = service();

        let result = service.update_task(
            999_999,
            "Título".to_string(),
            "Descrição".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn exclui_tarefa_existente() {
        let service = service();
        let task = service
            .create_task("Excluir".to_string(), String::new())
            .expect("criação deve funcionar");

        let result = service.delete_task(task.id());

        assert!(result.is_ok());
        assert!(service.list_tasks().expect("listagem deve funcionar").is_empty());
    }

    #[test]
    fn falha_ao_excluir_tarefa_inexistente() {
        let service = service();

        let result = service.delete_task(999_999);

        assert!(result.is_err());
    }

    #[test]
    fn conclui_tarefa() {
        let service = service();
        let task = service
            .create_task("Concluir".to_string(), String::new())
            .expect("criação deve funcionar");

        let completed = service
            .complete_task(task.id())
            .expect("conclusão deve funcionar");

        assert!(completed.is_completed());
    }

    #[test]
    fn reabre_tarefa_concluida() {
        let service = service();
        let task = service
            .create_task("Reabrir".to_string(), String::new())
            .expect("criação deve funcionar");

        service
            .complete_task(task.id())
            .expect("conclusão deve funcionar");
        let reopened = service
            .reopen_task(task.id())
            .expect("reabertura deve funcionar");

        assert!(!reopened.is_completed());
    }

    #[test]
    fn rejeita_id_invalido() {
        let service = service();

        let result = service.delete_task(0);

        assert!(result.is_err());
    }
}
