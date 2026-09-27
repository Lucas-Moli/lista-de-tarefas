use serde::Serialize;
use thiserror::Error;

pub const MAX_TITLE_LENGTH: usize = 120;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Task {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) completed: bool,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TaskValidationError {
    #[error("O título é obrigatório.")]
    EmptyTitle,
    #[error("O título deve ter no máximo {MAX_TITLE_LENGTH} caracteres.")]
    TitleTooLong,
    #[error("A descrição deve ter no máximo {MAX_DESCRIPTION_LENGTH} caracteres.")]
    DescriptionTooLong,
}

impl Task {
    pub(crate) fn new(
        id: i64,
        title: String,
        description: String,
        completed: bool,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            completed,
            created_at,
            updated_at,
        }
    }

    pub(crate) fn normalize_and_validate(
        title: &str,
        description: &str,
    ) -> Result<(String, String), TaskValidationError> {
        let normalized_title = title.trim().to_string();
        let normalized_description = description.trim().to_string();

        if normalized_title.is_empty() {
            return Err(TaskValidationError::EmptyTitle);
        }

        if normalized_title.chars().count() > MAX_TITLE_LENGTH {
            return Err(TaskValidationError::TitleTooLong);
        }

        if normalized_description.chars().count() > MAX_DESCRIPTION_LENGTH {
            return Err(TaskValidationError::DescriptionTooLong);
        }

        Ok((normalized_title, normalized_description))
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn reopen(&mut self) {
        self.completed = false;
    }

    pub fn validate(&self) -> Result<(), TaskValidationError> {
        Self::normalize_and_validate(&self.title, &self.description).map(|_| ())
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::{Task, TaskValidationError, MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH};

    fn make_task() -> Task {
        Task::new(
            1,
            "Estudar Rust".to_string(),
            "Revisar structs e impl".to_string(),
            false,
            "2026-09-25T18:00:00.000Z".to_string(),
            "2026-09-25T18:00:00.000Z".to_string(),
        )
    }

    #[test]
    fn cria_tarefa_valida() {
        let task = make_task();

        assert_eq!(task.id(), 1);
        assert_eq!(task.title(), "Estudar Rust");
        assert_eq!(task.description(), "Revisar structs e impl");
        assert!(!task.is_completed());
        assert!(task.validate().is_ok());
    }

    #[test]
    fn valida_titulo_obrigatorio() {
        let result = Task::normalize_and_validate("   ", "Descrição");

        assert_eq!(result, Err(TaskValidationError::EmptyTitle));
    }

    #[test]
    fn valida_titulo_com_limite() {
        let title = "a".repeat(MAX_TITLE_LENGTH);
        let result = Task::normalize_and_validate(&title, "Descrição");

        assert!(result.is_ok());
    }

    #[test]
    fn rejeita_titulo_maior_que_o_limite() {
        let title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let result = Task::normalize_and_validate(&title, "Descrição");

        assert_eq!(result, Err(TaskValidationError::TitleTooLong));
    }

    #[test]
    fn valida_descricao_com_limite() {
        let description = "a".repeat(MAX_DESCRIPTION_LENGTH);
        let result = Task::normalize_and_validate("Título", &description);

        assert!(result.is_ok());
    }

    #[test]
    fn rejeita_descricao_maior_que_o_limite() {
        let description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
        let result = Task::normalize_and_validate("Título", &description);

        assert_eq!(result, Err(TaskValidationError::DescriptionTooLong));
    }

    #[test]
    fn conclui_tarefa() {
        let mut task = make_task();

        task.complete();

        assert!(task.is_completed());
    }

    #[test]
    fn reabre_tarefa_concluida() {
        let mut task = make_task();
        task.complete();

        task.reopen();

        assert!(!task.is_completed());
    }
}
