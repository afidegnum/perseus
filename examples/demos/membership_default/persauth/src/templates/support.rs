use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

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

// Simple view with basic HTML - no complex reactivity for initial implementation
fn support_management_page() -> View {
    view! {
        // Simple navigation header
        nav(class = "navbar", style = "background: white; box-shadow: 0 2px 10px rgba(0,0,0,0.08); padding: 0.75rem 2rem; display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;") {
            div(class = "navbar-brand", style = "display: flex; align-items: center; gap: 10px;") {
                a(href = "/", style = "font-size: 1.25rem; font-weight: 700; color: #333; text-decoration: none;") {
                    span(style = "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent;") { "Perseus" }
                    span(style = "color: #666; font-weight: 400;") { "Membership" }
                }
            }
            div(class = "navbar-nav", style = "display: flex; gap: 0.5rem; align-items: center;") {
                a(href = "/admin", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Dashboard" }
                a(href = "/admin/users", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Users" }
                a(href = "/admin/roles", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Roles" }
                a(href = "/admin/support", style = "display: inline-block; padding: 0.5rem 1rem; color: #667eea; background: rgba(102,126,234,0.1); text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Support" }
            }
        }

        div(class = "page-body") {
            div(class = "container-fluid") {
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

                        // Filters
                        ul(class = "nav-tabs", style = "margin-bottom: 1rem;") {
                            li(class = "nav-item") {
                                a(href = "#all", class = "nav-link active", style = "cursor: pointer;") { "All" }
                            }
                            li(class = "nav-item") {
                                a(href = "#new", class = "nav-link", style = "cursor: pointer;") { "New" }
                            }
                            li(class = "nav-item") {
                                a(href = "#read", class = "nav-link", style = "cursor: pointer;") { "Read" }
                            }
                            li(class = "nav-item") {
                                a(href = "#replied", class = "nav-link", style = "cursor: pointer;") { "Replied" }
                            }
                            li(class = "nav-item") {
                                a(href = "#closed", class = "nav-link", style = "cursor: pointer;") { "Closed" }
                            }
                        }

                        div(class = "row") {
                            // Message List
                            div(class = "col-md-12") {
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
                            }
                        }
                    }
                }

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
