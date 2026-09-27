import { invoke } from '@tauri-apps/api/core';
import './style.css';

const state = {
  tasks: [],
  editingTaskId: null,
  deleteTaskId: null,
};

const elements = {
  taskForm: document.querySelector('#task-form'),
  taskTitle: document.querySelector('#task-title'),
  taskDescription: document.querySelector('#task-description'),
  titleCount: document.querySelector('#title-count'),
  descriptionCount: document.querySelector('#description-count'),
  formMode: document.querySelector('#form-mode'),
  formHeading: document.querySelector('#form-heading'),
  submitButton: document.querySelector('#submit-button'),
  cancelEdit: document.querySelector('#cancel-edit'),
  taskList: document.querySelector('#task-list'),
  emptyState: document.querySelector('#empty-state'),
  totalCount: document.querySelector('#total-count'),
  pendingCount: document.querySelector('#pending-count'),
  completedCount: document.querySelector('#completed-count'),
  feedback: document.querySelector('#feedback'),
  deleteDialog: document.querySelector('#delete-dialog'),
  cancelDelete: document.querySelector('#cancel-delete'),
  confirmDelete: document.querySelector('#confirm-delete'),
};

function countCharacters(value) {
  return [...value].length;
}

function validateForm(title, description) {
  const normalizedTitle = title.trim();
  const normalizedDescription = description.trim();
  const titleLength = countCharacters(normalizedTitle);
  const descriptionLength = countCharacters(normalizedDescription);

  if (titleLength === 0) {
    return { valid: false, message: 'O título da tarefa é obrigatório.' };
  }

  if (titleLength > 120) {
    return { valid: false, message: 'O título deve ter no máximo 120 caracteres.' };
  }

  if (descriptionLength > 500) {
    return { valid: false, message: 'A descrição deve ter no máximo 500 caracteres.' };
  }

  return {
    valid: true,
    title: normalizedTitle,
    description: normalizedDescription,
  };
}

function updateCharacterCounters() {
  elements.titleCount.textContent = `${countCharacters(elements.taskTitle.value)}/120`;
  elements.descriptionCount.textContent = `${countCharacters(elements.taskDescription.value)}/500`;
}

function showFeedback(message, type = 'success') {
  elements.feedback.hidden = false;
  elements.feedback.textContent = message;
  elements.feedback.className = `feedback ${type}`;
}

function hideFeedback() {
  elements.feedback.hidden = true;
  elements.feedback.textContent = '';
  elements.feedback.className = 'feedback';
}

function getErrorMessage(error) {
  if (typeof error === 'string' && error.trim() !== '') {
    return error;
  }

  if (error && typeof error.message === 'string' && error.message.trim() !== '') {
    return error.message;
  }

  return 'Não foi possível concluir a operação.';
}

function renderStats() {
  const total = state.tasks.length;
  const completed = state.tasks.filter((task) => task.completed).length;
  const pending = total - completed;

  elements.totalCount.textContent = String(total);
  elements.completedCount.textContent = String(completed);
  elements.pendingCount.textContent = String(pending);
}

function createActionButton(label, onClick, extraClass = '') {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = `action-button ${extraClass}`.trim();
  button.textContent = label;
  button.addEventListener('click', onClick);
  return button;
}

function formatDate(value) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString('pt-BR');
}

function renderTasks() {
  elements.taskList.replaceChildren();
  renderStats();

  if (state.tasks.length === 0) {
    elements.emptyState.hidden = false;
    return;
  }

  elements.emptyState.hidden = true;

  for (const task of state.tasks) {
    const item = document.createElement('li');
    item.className = `task-item${task.completed ? ' completed' : ''}`;

    const card = document.createElement('article');
    card.className = 'task-card';

    const headingRow = document.createElement('div');
    headingRow.className = 'task-heading-row';

    const titleWrap = document.createElement('div');
    titleWrap.className = 'task-title-wrap';

    const title = document.createElement('h3');
    title.className = 'task-title';
    title.textContent = task.title;

    titleWrap.appendChild(title);

    const status = document.createElement('span');
    status.className = 'status-badge';
    status.textContent = task.completed ? 'Concluída' : 'Pendente';

    headingRow.append(titleWrap, status);

    if (task.description) {
      const description = document.createElement('p');
      description.className = 'task-description';
      description.textContent = task.description;
      card.appendChild(headingRow);
      card.appendChild(description);
    } else {
      card.appendChild(headingRow);
    }

    const metadata = document.createElement('p');
    metadata.className = 'task-description';
    metadata.textContent = `Criada em ${formatDate(task.created_at)} · Atualizada em ${formatDate(task.updated_at)}`;
    card.appendChild(metadata);

    const actions = document.createElement('div');
    actions.className = 'task-actions';

    actions.appendChild(
      createActionButton('Editar', () => startEditing(task)),
    );

    actions.appendChild(
      createActionButton(
        task.completed ? 'Reabrir' : 'Concluir',
        () => toggleCompleted(task),
      ),
    );

    actions.appendChild(
      createActionButton('Excluir', () => openDeleteConfirmation(task.id), 'danger'),
    );

    card.appendChild(actions);
    item.appendChild(card);
    elements.taskList.appendChild(item);
  }
}

function resetForm() {
  state.editingTaskId = null;
  elements.taskForm.reset();
  elements.formMode.textContent = 'Nova tarefa';
  elements.formHeading.textContent = 'Criar uma tarefa';
  elements.submitButton.textContent = 'Adicionar tarefa';
  elements.cancelEdit.hidden = true;
  updateCharacterCounters();
}

function startEditing(task) {
  state.editingTaskId = task.id;
  elements.taskTitle.value = task.title;
  elements.taskDescription.value = task.description;
  elements.formMode.textContent = 'Edição';
  elements.formHeading.textContent = 'Editar tarefa';
  elements.submitButton.textContent = 'Salvar alterações';
  elements.cancelEdit.hidden = false;
  updateCharacterCounters();
  hideFeedback();
  elements.taskTitle.focus();
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function openDeleteConfirmation(taskId) {
  state.deleteTaskId = taskId;
  elements.deleteDialog.showModal();
}

function closeDeleteConfirmation() {
  state.deleteTaskId = null;
  elements.deleteDialog.close();
}

async function loadTasks() {
  try {
    state.tasks = await invoke('list_tasks');
    renderTasks();
  } catch (error) {
    showFeedback(getErrorMessage(error), 'error');
  }
}

async function handleSubmit(event) {
  event.preventDefault();

  const validation = validateForm(
    elements.taskTitle.value,
    elements.taskDescription.value,
  );

  if (!validation.valid) {
    showFeedback(validation.message, 'error');
    elements.taskTitle.focus();
    return;
  }

  try {
    if (state.editingTaskId === null) {
      await invoke('create_task', {
        title: validation.title,
        description: validation.description,
      });
      resetForm();
      await loadTasks();
      showFeedback('Tarefa criada com sucesso.');
      return;
    }

    await invoke('update_task', {
      id: state.editingTaskId,
      title: validation.title,
      description: validation.description,
    });
    resetForm();
    await loadTasks();
    showFeedback('Tarefa atualizada com sucesso.');
  } catch (error) {
    showFeedback(getErrorMessage(error), 'error');
  }
}

async function toggleCompleted(task) {
  try {
    if (task.completed) {
      await invoke('reopen_task', { id: task.id });
      await loadTasks();
      showFeedback('Tarefa reaberta com sucesso.');
    } else {
      await invoke('complete_task', { id: task.id });
      await loadTasks();
      showFeedback('Tarefa concluída com sucesso.');
    }
  } catch (error) {
    showFeedback(getErrorMessage(error), 'error');
  }
}

async function confirmDelete() {
  if (state.deleteTaskId === null) {
    return;
  }

  const taskId = state.deleteTaskId;

  try {
    elements.confirmDelete.disabled = true;
    await invoke('delete_task', { id: taskId });
    closeDeleteConfirmation();
    await loadTasks();
    showFeedback('Tarefa excluída com sucesso.');
  } catch (error) {
    showFeedback(getErrorMessage(error), 'error');
  } finally {
    elements.confirmDelete.disabled = false;
  }
}

elements.taskForm.addEventListener('submit', handleSubmit);
elements.cancelEdit.addEventListener('click', () => {
  resetForm();
  hideFeedback();
});
elements.taskTitle.addEventListener('input', updateCharacterCounters);
elements.taskDescription.addEventListener('input', updateCharacterCounters);
elements.cancelDelete.addEventListener('click', closeDeleteConfirmation);
elements.confirmDelete.addEventListener('click', confirmDelete);

elements.deleteDialog.addEventListener('cancel', (event) => {
  event.preventDefault();
  closeDeleteConfirmation();
});

updateCharacterCounters();
loadTasks();
