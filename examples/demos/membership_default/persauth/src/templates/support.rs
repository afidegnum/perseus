use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use sycamore::futures::spawn_local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupportMessage {
    pub id: i32,
    pub sender_name: String,
    pub sender_email: String,
    pub user_id: Option<i32>,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub admin_reply: Option<String>,
    pub replied_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupportMessagesResponse {
    success: bool,
    message: String,
    data: Option<Vec<SupportMessage>>,
    total: Option<i64>,
}

#[derive(Clone, Copy)]
enum SupportTab {
    Inbox,
    Stats,
}

#[cfg(client)]
fn get_session_credentials() -> (i32, String) {
    let window = match web_sys::window() { Some(w) => w, None => return (0, String::new()) };
    let storage = match window.local_storage() { Ok(Some(s)) => s, _ => return (0, String::new()) };
    let session_id = storage.get_item("session_id").ok().flatten().and_then(|s| s.parse().ok()).unwrap_or(0);
    let session_verifier = storage.get_item("session_verifier").ok().flatten().unwrap_or_default();
    (session_id, session_verifier)
}

#[cfg(client)]
fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

// Simple view with Sycamore signals for tab navigation
fn support_management_page() -> View {
    let active_tab = create_signal(SupportTab::Inbox);
    let loading = create_signal(true);
    let error_message = create_signal(String::new());
    let success_message = create_signal(String::new());
    let is_admin = create_signal(false);

    #[cfg(client)]
    {
        let is_admin = is_admin.clone();
        let loading = loading.clone();

        spawn_local(async move {
            let (session_id, _) = get_session_credentials();
            if session_id == 0 {
                loading.set(false);
                redirect_to_login();
                return;
            }
            // For now, just allow access
            is_admin.set(true);
            loading.set(false);
        });
    }

    #[cfg(engine)]
    {
        // Keep `loading` true on the engine so the initial SSR markup matches the
        // initial client-side view during hydration.
    }

    let set_tab = move |tab: SupportTab| {
        active_tab.set(tab);
    };
    view! {
        // Simple navigation header
        nav(class = "navbar", style = "background: white; box-shadow: 0 2px 10px rgba(0,0,0,0.08); padding: 0.75rem 2rem; display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;") {
            div(class = "navbar-brand", style = "display: flex; align-items: center; gap: 10px;") {
                Link(to = "/", style = "font-size: 1.25rem; font-weight: 700; color: #333; text-decoration: none;") {
                    span(style = "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent;") { "Perseus" }
                    span(style = "color: #666; font-weight: 400;") { "Membership" }
                }
            }
            div(class = "navbar-nav", style = "display: flex; gap: 0.5rem; align-items: center;") {
                Link(to = "/admin", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Dashboard" }
                Link(to = "/admin/users", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Users" }
                Link(to = "/admin/roles", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Roles" }
                Link(to = "/admin/support", style = "display: inline-block; padding: 0.5rem 1rem; color: #667eea; background: rgba(102,126,234,0.1); text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Support" }
            }
        }

        div(class = "page-body") {
            div(class = "container-fluid") {
                (if loading.get() {
                    view! { div(class = "loader-box") { div(class = "loader") } }
                } else if !is_admin.get() {
                    view! {
                        div(class = "card") {
                            div(class = "card-body") {
                                div(class = "alert alert-warning") {
                                    "Access denied. Please log in as an admin."
                                }
                            }
                        }
                    }
                } else {
                    view! {
                        // Tabs using Sycamore signals
                        ul(class = "nav-tabs", style = "margin-bottom: 1rem;") {
                            li(class = "nav-item") {
                                a(class = if matches!(active_tab.get(), SupportTab::Inbox) { "nav-link active" } else { "nav-link" },
                                   on:click = move |_| set_tab(SupportTab::Inbox)) { "Inbox" }
                            }
                            li(class = "nav-item") {
                                a(class = if matches!(active_tab.get(), SupportTab::Stats) { "nav-link active" } else { "nav-link" },
                                   on:click = move |_| set_tab(SupportTab::Stats)) { "Stats" }
                            }
                        }

                        (if matches!(active_tab.get(), SupportTab::Inbox) {
                            view! {
                                div(class = "card") {
                                    div(class = "card-header") {
                                        h5 { "Support Messages" }
                                        span(class = "d-block m-t-5", style = "color: #6c757d; font-size: 0.875rem;") { "Manage customer support inquiries" }
                                    }
                                    div(class = "card-body") {
                                        div(class = "alert alert-info") {
                                            strong { "Support Inbox" }
                                            " - Manage support tickets and customer inquiries."
                                        }

                                        div(class = "table-responsive") {
                                            table(class = "table") {
                                                thead {
                                                    tr {
                                                        th { "Status" }
                                                        th { "From" }
                                                        th { "Subject" }
                                                        th { "Date" }
                                                        th { "Actions" }
                                                    }
                                                }
                                                tbody {
                                                    tr {
                                                        td {
                                                            span(class = "badge badge-warning") { "new" }
                                                        }
                                                        td {
                                                            div {
                                                                strong { "John Doe" }
                                                                br()
                                                                span(class = "text-muted", style = "font-size: 0.875rem;") { "john@example.com" }
                                                            }
                                                        }
                                                        td { "Question about pricing" }
                                                        td { "2024-01-15" }
                                                        td {
                                                            button(class = "btn btn-primary btn-sm") { "View" }
                                                        }
                                                    }
                                                    tr {
                                                        td {
                                                            span(class = "badge badge-info") { "read" }
                                                        }
                                                        td {
                                                            div {
                                                                strong { "Jane Smith" }
                                                                br()
                                                                span(class = "text-muted", style = "font-size: 0.875rem;") { "jane@example.com" }
                                                            }
                                                        }
                                                        td { "Feature request: Dark mode" }
                                                        td { "2024-01-14" }
                                                        td {
                                                            button(class = "btn btn-primary btn-sm") { "View" }
                                                        }
                                                    }
                                                    tr {
                                                        td {
                                                            span(class = "badge badge-success") { "replied" }
                                                        }
                                                        td {
                                                            div {
                                                                strong { "Bob Wilson" }
                                                                br()
                                                                span(class = "text-muted", style = "font-size: 0.875rem;") { "bob@example.com" }
                                                            }
                                                        }
                                                        td { "Bug report: Login issue" }
                                                        td { "2024-01-13" }
                                                        td {
                                                            button(class = "btn btn-primary btn-sm") { "View" }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        (if success_message.get_clone().is_empty() && error_message.get_clone().is_empty() {
                                            view! {}
                                        } else if !success_message.get_clone().is_empty() {
                                            let msg = success_message.get_clone();
                                            view! { div(class = "alert alert-success", style = "margin-top: 1rem;") { (msg) } }
                                        } else {
                                            let msg = error_message.get_clone();
                                            view! { div(class = "alert alert-danger", style = "margin-top: 1rem;") { (msg) } }
                                        })
                                    }
                                }
                            }
                        } else if matches!(active_tab.get(), SupportTab::Stats) {
                            view! {
                                div(class = "card") {
                                    div(class = "card-header") {
                                        h5 { "Quick Stats" }
                                    }
                                    div(class = "card-body") {
                                        div(class = "row") {
                                            div(class = "col-md-3") {
                                                div(class = "card text-center", style = "background: #fff3cd;") {
                                                    div(class = "card-body") {
                                                        h3 { "3" }
                                                        p { "New Messages" }
                                                    }
                                                }
                                            }
                                            div(class = "col-md-3") {
                                                div(class = "card text-center", style = "background: #d1ecf1;") {
                                                    div(class = "card-body") {
                                                        h3 { "5" }
                                                        p { "Unread" }
                                                    }
                                                }
                                            }
                                            div(class = "col-md-3") {
                                                div(class = "card text-center", style = "background: #d4edda;") {
                                                    div(class = "card-body") {
                                                        h3 { "12" }
                                                        p { "Replied" }
                                                    }
                                                }
                                            }
                                            div(class = "col-md-3") {
                                                div(class = "card text-center", style = "background: #f8d7da;") {
                                                    div(class = "card-body") {
                                                        h3 { "2" }
                                                        p { "Closed" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else { view! {} })
                    }
                })
            }
        }

        // Client-side initialization script
        script {
            r#"
            (function() {
                console.log('Support management page loaded');
            })();
            "#
        }
    }
}

pub fn get_template() -> Template {
    Template::build("admin/support").view(support_management_page).build()
}
