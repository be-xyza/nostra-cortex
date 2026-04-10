use crate::v2::api::{
    build_comment_query, build_kip_find_by_space_and_type, create_agent, create_comment,
    create_question, fetch_kip_entities,
};
use crate::v2::types::KipEntity;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct QuestionsPageProps {
    pub on_back: EventHandler<()>,
}

#[component]
pub fn QuestionsPage(props: QuestionsPageProps) -> Element {
    let mut space_id = use_signal(|| "space_default".to_string());
    let questions = use_signal(|| Vec::<KipEntity>::new());
    let comments = use_signal(|| Vec::<KipEntity>::new());
    let mut selected_question_id = use_signal(|| None::<String>);
    let mut question_title = use_signal(|| String::new());
    let mut question_body = use_signal(|| String::new());
    let mut comment_body = use_signal(|| String::new());
    let loading_questions = use_signal(|| false);
    let loading_comments = use_signal(|| false);
    let mut submitting_question = use_signal(|| false);
    let mut submitting_comment = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);
    let initial_space_id = "space_default".to_string();

    use_future(move || {
        load_questions(
            initial_space_id.clone(),
            questions,
            comments,
            selected_question_id,
            loading_questions,
            loading_comments,
            error_msg,
        )
    });

    let mut handle_create_question = move || {
        let current_title = question_title.read().trim().to_string();
        let current_body = question_body.read().trim().to_string();
        let current_space_id = space_id.read().trim().to_string();

        if current_title.is_empty() {
            error_msg.set(Some("Question title is required.".to_string()));
            return;
        }
        if current_space_id.is_empty() {
            error_msg.set(Some("Space ID is required.".to_string()));
            return;
        }

        submitting_question.set(true);
        spawn(async move {
            let agent = create_agent().await;

            match create_question(&agent, &current_space_id, &current_title, &current_body).await {
                Ok(_) => {
                    success_msg.set(Some("Question created.".to_string()));
                    error_msg.set(None);
                    question_title.set(String::new());
                    question_body.set(String::new());
                    spawn(load_questions(
                        current_space_id,
                        questions,
                        comments,
                        selected_question_id,
                        loading_questions,
                        loading_comments,
                        error_msg,
                    ));
                }
                Err(err) => {
                    error_msg.set(Some(format!("Failed to create question: {}", err)));
                }
            }
            submitting_question.set(false);
        });
    };

    let mut handle_create_comment = move || {
        let Some(question_id) = selected_question_id.read().clone() else {
            error_msg.set(Some(
                "Select a question before adding a comment.".to_string(),
            ));
            return;
        };
        let Some(question) = questions
            .read()
            .iter()
            .find(|item| item.id == question_id)
            .cloned()
        else {
            error_msg.set(Some(
                "Selected question is no longer available.".to_string(),
            ));
            return;
        };

        let current_comment_body = comment_body.read().trim().to_string();
        let current_space_id = space_id.read().trim().to_string();
        if current_comment_body.is_empty() {
            error_msg.set(Some("Comment body is required.".to_string()));
            return;
        }
        if current_space_id.is_empty() {
            error_msg.set(Some("Space ID is required.".to_string()));
            return;
        }

        submitting_comment.set(true);
        let current_question_id = question.id.clone();
        let current_question_title = question.display_title().to_string();
        spawn(async move {
            let agent = create_agent().await;

            match create_comment(
                &agent,
                &current_space_id,
                &current_question_id,
                &current_question_title,
                &current_comment_body,
            )
            .await
            {
                Ok(_) => {
                    success_msg.set(Some("Comment added.".to_string()));
                    error_msg.set(None);
                    comment_body.set(String::new());
                    spawn(load_comments(
                        current_space_id,
                        current_question_id,
                        comments,
                        loading_comments,
                        error_msg,
                    ));
                }
                Err(err) => {
                    error_msg.set(Some(format!("Failed to add comment: {}", err)));
                }
            }
            submitting_comment.set(false);
        });
    };

    let questions_snapshot = questions.read().clone();
    let comments_snapshot = comments.read().clone();
    let selected_question = selected_question_id
        .read()
        .as_ref()
        .and_then(|question_id| {
            questions_snapshot
                .iter()
                .find(|item| &item.id == question_id)
                .cloned()
        });

    let question_cards = questions_snapshot
        .iter()
        .cloned()
        .map(|question| {
            let is_selected = selected_question_id
                .read()
                .as_ref()
                .map(|value| value == &question.id)
                .unwrap_or(false);
            let question_id = question.id.clone();
            let question_title = question.display_title().to_string();

            rsx! {
                button {
                    key: "{question_id}",
                    class: format!(
                        "w-full text-left rounded-xl border p-4 transition-colors {}",
                        if is_selected {
                            "border-zinc-100 bg-zinc-100/5"
                        } else {
                            "border-zinc-800 bg-zinc-900/40 hover:border-zinc-600"
                        }
                    ),
                    onclick: move |_| {
                        selected_question_id.set(Some(question_id.clone()));
                        spawn(load_comments(
                            space_id.read().clone(),
                            question_id.clone(),
                            comments,
                            loading_comments,
                            error_msg,
                        ));
                    },
                    div { class: "flex items-start justify-between gap-3",
                        div {
                            div { class: "text-xs uppercase tracking-wide text-zinc-500", "Question" }
                            div { class: "mt-1 text-lg font-semibold text-zinc-50", "{question_title}" }
                            p { class: "mt-2 text-sm text-zinc-400 line-clamp-2", "{question.description}" }
                        }
                        span { class: "rounded-full border border-zinc-700 px-2 py-0.5 text-[11px] text-zinc-300", "{question.attributes.len()} attrs" }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let comment_cards = comments_snapshot
        .iter()
        .cloned()
        .map(|comment| {
            let comment_title = comment.display_title().to_string();
            rsx! {
                article {
                    key: "{comment.id}",
                    class: "rounded-lg border border-zinc-800 bg-zinc-900/80 p-3",
                    div { class: "text-xs text-zinc-500", "{comment_title}" }
                    p { class: "mt-2 text-sm text-zinc-200 whitespace-pre-wrap", "{comment.description}" }
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        section {
            class: "space-y-6 max-w-6xl mx-auto",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight", "Questions" }
                    p { class: "text-sm text-zinc-400", "Capture questions and thread comments against them." }
                }
                button {
                    class: "px-4 py-2 rounded-md border border-zinc-700 text-zinc-200 text-sm",
                    onclick: move |_| props.on_back.call(()),
                    "Back"
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "rounded-lg border border-rose-700/60 bg-rose-950/40 px-4 py-3 text-rose-200", "{err}" }
            }
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "rounded-lg border border-emerald-700/60 bg-emerald-950/40 px-4 py-3 text-emerald-200", "{msg}" }
            }

            div { class: "grid gap-6 lg:grid-cols-[1fr_360px]",
                div { class: "space-y-4",
                    div { class: "rounded-xl border border-zinc-800 bg-zinc-900/70 p-4 space-y-4",
                        div { class: "grid gap-3 md:grid-cols-[180px_1fr_auto]",
                            div { class: "space-y-1",
                                label { class: "text-xs uppercase tracking-wide text-zinc-500", "Space ID" }
                                input {
                                    class: "w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100",
                                    value: "{space_id}",
                                    placeholder: "space_default",
                                    oninput: move |evt| space_id.set(evt.value())
                                }
                            }
                            div { class: "mt-6",
                                button {
                                    class: "px-4 py-2 rounded-md border border-zinc-700 text-sm text-zinc-100",
                                    onclick: move |_| {
                                        spawn(load_questions(
                                            space_id.read().clone(),
                                            questions,
                                            comments,
                                            selected_question_id,
                                            loading_questions,
                                            loading_comments,
                                            error_msg,
                                        ));
                                    },
                                    if loading_questions() { "Refreshing..." } else { "Refresh Questions" }
                                }
                            }
                            div { class: "mt-6 text-xs text-zinc-500 self-center", "{questions.len()} question(s)" }
                        }

                        div { class: "grid gap-3 md:grid-cols-2",
                            div { class: "space-y-1",
                                label { class: "text-xs uppercase tracking-wide text-zinc-500", "Question title" }
                                input {
                                    class: "w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100",
                                    value: "{question_title}",
                                    placeholder: "What should we decide?",
                                    oninput: move |evt| question_title.set(evt.value())
                                }
                            }
                            div { class: "space-y-1",
                                label { class: "text-xs uppercase tracking-wide text-zinc-500", "Body" }
                                textarea {
                                    class: "min-h-[80px] w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100",
                                    value: "{question_body}",
                                    placeholder: "Add supporting context...",
                                    oninput: move |evt| question_body.set(evt.value())
                                }
                            }
                        }

                        div { class: "flex justify-end",
                            button {
                                class: "inline-flex items-center justify-center rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950 disabled:opacity-60",
                                disabled: submitting_question(),
                                onclick: move |_| handle_create_question(),
                                if submitting_question() { "Creating..." } else { "Create Question" }
                            }
                        }
                    }

                    div { class: "space-y-3",
                        if loading_questions() {
                            div { class: "rounded-lg border border-zinc-800 bg-zinc-900/60 p-8 text-sm text-zinc-400", "Loading questions..." }
                        } else if questions.read().is_empty() {
                            div { class: "rounded-lg border border-dashed border-zinc-700 bg-zinc-950/40 p-8 text-sm text-zinc-400",
                                "No questions found for this space."
                            }
                        } else {
                            {question_cards.into_iter()}
                        }
                    }
                }

                div { class: "space-y-4",
                    div { class: "rounded-xl border border-zinc-800 bg-zinc-900/70 p-4 space-y-4",
                        h2 { class: "text-lg font-semibold", "Question Thread" }
                        if let Some(question) = selected_question.clone() {
                            div { class: "space-y-3",
                                div {
                                    div { class: "text-xs uppercase tracking-wide text-zinc-500", "Selected" }
                                    div { class: "text-xl font-semibold text-zinc-50", "{question.display_title()}" }
                                    p { class: "mt-2 text-sm text-zinc-300", "{question.description}" }
                                }

                                div { class: "rounded-lg border border-zinc-800 bg-zinc-950/60 p-3 space-y-2",
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-xs uppercase tracking-wide text-zinc-500", "Comments" }
                                        span { class: "text-xs text-zinc-400", "{comments.len()} total" }
                                    }

                                    if loading_comments() {
                                        div { class: "text-sm text-zinc-400", "Loading comments..." }
                                    } else if comments.read().is_empty() {
                                        div { class: "text-sm text-zinc-400", "No comments yet." }
                                    } else {
                                        div { class: "space-y-3",
                                            {comment_cards.into_iter()}
                                        }
                                    }
                                }
                            }

                            div { class: "rounded-xl border border-zinc-800 bg-zinc-950/50 p-4 space-y-3",
                                div { class: "text-sm font-medium text-zinc-100", "Add Comment" }
                                textarea {
                                    class: "min-h-[120px] w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100",
                                    value: "{comment_body}",
                                    placeholder: "Write a reply to this question...",
                                    oninput: move |evt| comment_body.set(evt.value())
                                }
                                div { class: "flex justify-end",
                                    button {
                                        class: "inline-flex items-center justify-center rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950 disabled:opacity-60",
                                        disabled: submitting_comment(),
                                        onclick: move |_| handle_create_comment(),
                                        if submitting_comment() { "Saving..." } else { "Add Comment" }
                                    }
                                }
                            }
                        } else {
                            div { class: "rounded-lg border border-dashed border-zinc-700 bg-zinc-950/40 p-6 text-sm text-zinc-400",
                                "Select a question to view its thread."
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn load_questions(
    space_id: String,
    mut questions: Signal<Vec<KipEntity>>,
    mut comments: Signal<Vec<KipEntity>>,
    mut selected_question_id: Signal<Option<String>>,
    mut loading_questions: Signal<bool>,
    loading_comments: Signal<bool>,
    mut error_msg: Signal<Option<String>>,
) {
    if space_id.trim().is_empty() {
        error_msg.set(Some("Space ID is required.".to_string()));
        questions.set(Vec::new());
        comments.set(Vec::new());
        loading_questions.set(false);
        return;
    }

    loading_questions.set(true);
    let agent = create_agent().await;
    let query = build_kip_find_by_space_and_type("Question", &space_id);

    match fetch_kip_entities(&agent, query).await {
        Ok(list) => {
            let first_question = list.first().map(|item| item.id.clone());
            questions.set(list);
            error_msg.set(None);

            if selected_question_id.read().is_none() {
                if let Some(question_id) = first_question {
                    selected_question_id.set(Some(question_id.clone()));
                    spawn(load_comments(
                        space_id.clone(),
                        question_id,
                        comments,
                        loading_comments,
                        error_msg,
                    ));
                } else {
                    comments.set(Vec::new());
                }
            } else if let Some(question_id) = selected_question_id.read().clone() {
                spawn(load_comments(
                    space_id.clone(),
                    question_id,
                    comments,
                    loading_comments,
                    error_msg,
                ));
            }
        }
        Err(err) => {
            questions.set(Vec::new());
            comments.set(Vec::new());
            error_msg.set(Some(format!("Failed to load questions: {}", err)));
        }
    }

    loading_questions.set(false);
}

async fn load_comments(
    space_id: String,
    question_id: String,
    mut comments: Signal<Vec<KipEntity>>,
    mut loading_comments: Signal<bool>,
    mut error_msg: Signal<Option<String>>,
) {
    if space_id.trim().is_empty() || question_id.trim().is_empty() {
        comments.set(Vec::new());
        return;
    }

    loading_comments.set(true);
    let agent = create_agent().await;
    let query = build_comment_query(&space_id, &question_id);

    match fetch_kip_entities(&agent, query).await {
        Ok(list) => {
            comments.set(list);
            error_msg.set(None);
        }
        Err(err) => {
            comments.set(Vec::new());
            error_msg.set(Some(format!("Failed to load comments: {}", err)));
        }
    }

    loading_comments.set(false);
}
