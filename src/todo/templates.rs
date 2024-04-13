use super::models;
use askama::Template;

#[derive(Template)]
#[template(path = "todo/index.html")]
pub struct HelloTemplate;

#[derive(Template)]
#[template(path = "todo/stream.html")]
pub struct StreamTemplate;

#[derive(Template)]
#[template(path = "todo/todos.html")]
pub struct Records {
    pub todos: Vec<models::Todo>,
}

#[derive(Template)]
#[template(path = "todo/todo.html")]
pub struct TodoNewTemplate {
    pub todo: models::Todo,
}
