use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

use crate::{
    mark_updated, now_utc, storage, validate_property_value, FrumpDoc, FrumpRepo, PropertyKey,
    Task, TaskId, TaskType,
};

#[derive(Clone)]
struct AppState {
    file: Arc<PathBuf>,
}

#[derive(Debug, Serialize)]
struct DocumentDto {
    header: String,
    team: Vec<TeamMemberDto>,
    tasks: Vec<TaskDto>,
}

#[derive(Debug, Serialize)]
struct TeamMemberDto {
    name: String,
    email: String,
    role: Option<String>,
}

#[derive(Debug, Serialize)]
struct TaskDto {
    id: u32,
    task_type: String,
    subject: String,
    body: String,
    properties: Vec<PropertyDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PropertyDto {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TaskInput {
    task_type: String,
    subject: String,
    body: String,
    properties: Vec<PropertyDto>,
}

#[derive(Debug, Deserialize)]
struct NotifyInput {
    recipient: String,
    message: String,
}

#[derive(Debug)]
struct ApiError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
    }
}

type ApiResult<T> = std::result::Result<T, ApiError>;

/// Serve the embedded board on loopback. Document reads occur on each request so external edits
/// are reflected by the browser's periodic refresh without maintaining a second source of truth.
pub async fn serve(file: PathBuf, port: u16) -> Result<()> {
    let state = AppState {
        file: Arc::new(file),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/api/document", get(document))
        .route("/api/tasks", post(create_task))
        .route("/api/tasks/{id}", put(update_task).delete(delete_task))
        .route("/api/tasks/{id}/notify", post(notify_task))
        .with_state(state);
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("Failed to bind local web server at http://{address}"))?;
    println!("Frump board: http://{address}");
    axum::serve(listener, app)
        .await
        .context("Local web server stopped unexpectedly")
}

async fn index() -> Html<&'static str> {
    Html(include_str!("web/index.html"))
}

async fn document(State(state): State<AppState>) -> ApiResult<Json<DocumentDto>> {
    Ok(Json(read_document(&state.file)?))
}

async fn create_task(
    State(state): State<AppState>,
    Json(input): Json<TaskInput>,
) -> ApiResult<Json<TaskDto>> {
    let _lock = acquire_write_lock(&state.file)?;
    let mut doc = read_parsed_document(&state.file)?;
    let id = next_task_id(&doc);
    validate_new_property_values(&input.properties)?;
    let mut task = task_from_input(id, input)?;
    mark_updated(&mut task, &now_utc());
    let response = task_to_dto(&task);
    doc.tasks.add(task);
    write_document(&state.file, &doc)?;
    Ok(Json(response))
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(input): Json<TaskInput>,
) -> ApiResult<Json<TaskDto>> {
    let _lock = acquire_write_lock(&state.file)?;
    let mut doc = read_parsed_document(&state.file)?;
    let task_id = TaskId::new(id)?;
    let new_status = input
        .properties
        .iter()
        .find(|property| property.key == "Status")
        .map(|property| property.value.clone());
    doc.ensure_next_transition(task_id, new_status.as_deref())?;
    let task = doc
        .tasks
        .find_by_id_mut(task_id)
        .ok_or_else(|| ApiError(anyhow::anyhow!("Task {id} not found")))?;
    validate_changed_property_values(task, &input.properties)?;
    let mut replacement = task_from_input(task_id, input)?;
    mark_updated(&mut replacement, &now_utc());
    *task = replacement;
    let response = task_to_dto(task);
    let completed = new_status.as_deref() == Some("done");
    if completed {
        doc.remove_from_next(task_id);
    }
    write_document(&state.file, &doc)?;
    Ok(Json(response))
}

async fn delete_task(State(state): State<AppState>, Path(id): Path<u32>) -> ApiResult<StatusCode> {
    let _lock = acquire_write_lock(&state.file)?;
    let mut doc = read_parsed_document(&state.file)?;
    let task_id = TaskId::new(id)?;
    doc.tasks
        .remove(task_id)
        .ok_or_else(|| ApiError(anyhow::anyhow!("Task {id} not found")))?;
    write_document(&state.file, &doc)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn notify_task(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(input): Json<NotifyInput>,
) -> ApiResult<StatusCode> {
    if input.recipient.trim().is_empty() || input.message.trim().is_empty() {
        return Err(ApiError(anyhow::anyhow!(
            "Recipient and message are required"
        )));
    }
    let doc = read_parsed_document(&state.file)?;
    let task = doc
        .tasks
        .find_by_id(TaskId::new(id)?)
        .ok_or_else(|| ApiError(anyhow::anyhow!("Task {id} not found")))?;
    let text = format!(
        "Frump task #{}: {}\n\n{}",
        task.id,
        task.subject,
        input.message.trim()
    );
    let output = std::process::Command::new("metateam")
        .args([
            "crew",
            "message",
            "--from",
            "frump",
            input.recipient.trim(),
            &text,
        ])
        .output()
        .context("Failed to run Metateam")?;
    if !output.status.success() {
        return Err(ApiError(anyhow::anyhow!(
            "Metateam could not send notification: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn read_document(file: &PathBuf) -> Result<DocumentDto> {
    Ok(document_to_dto(&read_parsed_document(file)?))
}

fn read_parsed_document(file: &PathBuf) -> Result<FrumpDoc> {
    storage::read(file).with_context(|| format!("Failed to read task board {}", file.display()))
}

fn write_document(file: &PathBuf, doc: &FrumpDoc) -> Result<()> {
    storage::write(file, doc)
        .with_context(|| format!("Failed to write task board {}", file.display()))
}

fn acquire_write_lock(file: &PathBuf) -> Result<fs::File> {
    let lock_path = storage::sharded_root(file)
        .map(|root| root.join("general.md"))
        .unwrap_or_else(|| file.clone());
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.lock_exclusive()?;
    Ok(lock)
}

fn next_task_id(doc: &FrumpDoc) -> TaskId {
    if let Ok(repo) = FrumpRepo::open(".") {
        if let Ok(Some(max_historical)) = repo.max_historical_id() {
            return match doc.tasks.max_id() {
                Some(current) if max_historical > current => max_historical.next(),
                Some(current) => current.next(),
                None => max_historical.next(),
            };
        }
    }
    doc.tasks.next_id()
}

fn task_from_input(id: TaskId, input: TaskInput) -> Result<Task> {
    if input.subject.trim().is_empty() {
        anyhow::bail!("Task subject cannot be empty");
    }
    let mut task = Task::new(
        id,
        TaskType::parse(&input.task_type),
        input.subject.trim().to_string(),
    );
    task.set_body(input.body);
    for property in input.properties {
        task.add_property(PropertyKey::new(&property.key)?, property.value);
    }
    Ok(task)
}

fn validate_new_property_values(properties: &[PropertyDto]) -> Result<()> {
    for property in properties {
        validate_property_value(&property.value)
            .map_err(|error| anyhow::anyhow!("Property '{}': {error}", property.key))?;
    }
    Ok(())
}

fn validate_changed_property_values(existing: &Task, properties: &[PropertyDto]) -> Result<()> {
    for property in properties {
        let key = PropertyKey::new(&property.key)?;
        if existing.get_property(&key) != Some(property.value.as_str()) {
            validate_property_value(&property.value)
                .map_err(|error| anyhow::anyhow!("Property '{}': {error}", property.key))?;
        }
    }
    Ok(())
}

fn document_to_dto(doc: &FrumpDoc) -> DocumentDto {
    DocumentDto {
        header: doc.header.clone(),
        team: doc
            .team
            .members()
            .iter()
            .map(|member| TeamMemberDto {
                name: member.name.clone(),
                email: member.email.as_str().to_string(),
                role: member.role.clone(),
            })
            .collect(),
        tasks: doc.tasks.tasks().iter().map(task_to_dto).collect(),
    }
}

fn task_to_dto(task: &Task) -> TaskDto {
    TaskDto {
        id: task.id.value(),
        task_type: task.task_type.as_str().to_string(),
        subject: task.subject.clone(),
        body: task.body.clone(),
        properties: task
            .properties
            .iter()
            .map(|property| PropertyDto {
                key: property.key.as_str().to_string(),
                value: property.value.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskCollection, Team};

    #[test]
    fn task_input_preserves_property_order() {
        let task = task_from_input(
            TaskId::new(1).unwrap(),
            TaskInput {
                task_type: "Task".to_string(),
                subject: "A task".to_string(),
                body: String::new(),
                properties: vec![
                    PropertyDto {
                        key: "Priority".to_string(),
                        value: "high".to_string(),
                    },
                    PropertyDto {
                        key: "Status".to_string(),
                        value: "open".to_string(),
                    },
                ],
            },
        )
        .unwrap();
        assert_eq!(task_to_dto(&task).properties[0].key, "Priority");
        assert_eq!(task_to_dto(&task).properties[1].key, "Status");
    }

    #[test]
    fn document_dto_contains_all_tasks() {
        let task = Task::new(TaskId::new(3).unwrap(), TaskType::Bug, "Fix it".to_string());
        let doc = FrumpDoc::new(
            "# Project\n".to_string(),
            Team::empty(),
            TaskCollection::new(vec![task]),
        );
        assert_eq!(document_to_dto(&doc).tasks[0].id, 3);
    }

    #[test]
    fn legacy_long_property_survives_an_unrelated_web_edit() {
        let mut task = Task::new(
            TaskId::new(1).unwrap(),
            TaskType::Task,
            "A task".to_string(),
        );
        task.add_property(
            PropertyKey::new("Review").unwrap(),
            "A legacy review value deliberately longer than forty bytes".to_string(),
        );
        let unchanged = vec![PropertyDto {
            key: "Review".to_string(),
            value: "A legacy review value deliberately longer than forty bytes".to_string(),
        }];
        assert!(validate_changed_property_values(&task, &unchanged).is_ok());

        let changed = vec![PropertyDto {
            key: "Review".to_string(),
            value: "A changed review value deliberately longer than forty bytes".to_string(),
        }];
        let error = validate_changed_property_values(&task, &changed)
            .unwrap_err()
            .to_string();
        assert!(error.contains("Review"));
        assert!(error.contains("task body"));
    }
}
