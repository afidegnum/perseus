use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub roles: Vec<String>,
}

// Simple view with basic HTML - no complex reactivity for initial implementation
fn roles_management_page() -> View {
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
                a(href = "/admin/roles", style = "display: inline-block; padding: 0.5rem 1rem; color: #667eea; background: rgba(102,126,234,0.1); text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Roles" }
                a(href = "/admin/support", style = "display: inline-block; padding: 0.5rem 1rem; color: #495057; text-decoration: none; border-radius: 4px; font-size: 0.875rem; font-weight: 500;") { "Support" }
            }
        }

        div(class = "page-body") {
            div(class = "container-fluid") {
                div(class = "card") {
                    div(class = "card-header") {
                        h5 { "Role-Based Access Control" }
                        span(class = "d-block m-t-5", style = "color: #6c757d; font-size: 0.875rem;") { "Manage roles, permissions, and user access" }
                    }
                    div(class = "card-body") {
                        div(class = "alert alert-info") {
                            strong { "RBAC Management" }
                            " - Manage roles, permissions, and user access control."
                        }

                        // Tabs
                        ul(class = "nav-tabs", style = "margin-bottom: 1.5rem;") {
                            li(class = "nav-item") {
                                a(href = "#users", class = "nav-link active", style = "cursor: pointer;") { "Users" }
                            }
                            li(class = "nav-item") {
                                a(href = "#roles", class = "nav-link", style = "cursor: pointer;") { "Roles" }
                            }
                            li(class = "nav-item") {
                                a(href = "#permissions", class = "nav-link", style = "cursor: pointer;") { "Permissions" }
                            }
                        }

                        // Users Tab Content
                        div(id = "users", style = "margin-bottom: 2rem;") {
                            h6 { "User Management" }
                            p(class = "text-muted") { "Manage user roles and permissions." }
                            div(class = "table-responsive") {
                                table(class = "table") {
                                    thead {
                                        tr {
                                            th { "ID" }
                                            th { "Email" }
                                            th { "Roles" }
                                            th { "Actions" }
                                        }
                                    }
                                    tbody {
                                        tr {
                                            td { "1" }
                                            td { "admin@example.com" }
                                            td {
                                                span(class = "badge badge-info", style = "margin-right: 0.25rem;") { "admin" }
                                            }
                                            td {
                                                select(class = "form-control", style = "width: auto; display: inline-block;") {
                                                    option(value = "") { "-- Select User --" }
                                                    option(value = "1") { "admin@example.com" }
                                                    option(value = "2") { "user@example.com" }
                                                }
                                                select(class = "form-control", style = "width: auto; display: inline-block; margin-left: 0.5rem;") {
                                                    option(value = "") { "-- Select Role --" }
                                                    option(value = "admin") { "admin" }
                                                    option(value = "editor") { "editor" }
                                                    option(value = "user") { "user" }
                                                }
                                                button(class = "btn btn-primary btn-sm", style = "margin-left: 0.5rem;") { "Assign" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Roles Tab Content
                        div(id = "roles", style = "margin-bottom: 2rem;") {
                            h6 { "Role Management" }
                            div(class = "card", style = "margin-bottom: 1.5rem;") {
                                div(class = "card-header") {
                                    h6 { "Create New Role" }
                                }
                                div(class = "card-body") {
                                    div(class = "row") {
                                        div(class = "col-md-4") {
                                            div(class = "form-group") {
                                                label(r#for = "role_name") { "Role Name" }
                                                input(r#type = "text", id = "role_name", class = "form-control", placeholder = "e.g., moderator")
                                            }
                                        }
                                        div(class = "col-md-6") {
                                            div(class = "form-group") {
                                                label(r#for = "role_description") { "Description" }
                                                input(r#type = "text", id = "role_description", class = "form-control", placeholder = "Role description")
                                            }
                                        }
                                        div(class = "col-md-2") {
                                            div(class = "form-group") {
                                                label { "&nbsp;" }
                                                button(class = "btn btn-primary") { "Create Role" }
                                            }
                                        }
                                    }
                                }
                            }

                            div(class = "table-responsive") {
                                table(class = "table") {
                                    thead {
                                        tr {
                                            th { "ID" }
                                            th { "Name" }
                                            th { "Description" }
                                            th { "Actions" }
                                        }
                                    }
                                    tbody {
                                        tr {
                                            td { "1" }
                                            td { strong { "admin" } }
                                            td { "Full system access" }
                                            td {
                                                button(class = "btn btn-secondary btn-sm") { "Edit" }
                                                button(class = "btn btn-danger btn-sm", style = "margin-left: 0.25rem;") { "Delete" }
                                            }
                                        }
                                        tr {
                                            td { "2" }
                                            td { strong { "editor" } }
                                            td { "Can edit content" }
                                            td {
                                                button(class = "btn btn-secondary btn-sm") { "Edit" }
                                                button(class = "btn btn-danger btn-sm", style = "margin-left: 0.25rem;") { "Delete" }
                                            }
                                        }
                                        tr {
                                            td { "3" }
                                            td { strong { "user" } }
                                            td { "Basic user access" }
                                            td {
                                                button(class = "btn btn-secondary btn-sm") { "Edit" }
                                                button(class = "btn btn-danger btn-sm", style = "margin-left: 0.25rem;") { "Delete" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Permissions Tab Content
                        div(id = "permissions") {
                            h6 { "System Permissions" }
                            div(class = "table-responsive") {
                                table(class = "table") {
                                    thead {
                                        tr {
                                            th { "ID" }
                                            th { "Name" }
                                            th { "Description" }
                                        }
                                    }
                                    tbody {
                                        tr {
                                            td { "1" }
                                            td { code { "create_posts" } }
                                            td { "Can create new posts" }
                                        }
                                        tr {
                                            td { "2" }
                                            td { code { "edit_posts" } }
                                            td { "Can edit any post" }
                                        }
                                        tr {
                                            td { "3" }
                                            td { code { "delete_posts" } }
                                            td { "Can delete any post" }
                                        }
                                        tr {
                                            td { "4" }
                                            td { code { "manage_users" } }
                                            td { "Can manage user accounts" }
                                        }
                                    }
                                }
                            }

                            div(class = "alert alert-info", style = "margin-top: 1.5rem;") {
                                h6 { "About Permissions" }
                                p { "Permissions define what actions users can perform. Each role has a set of permissions. Users inherit permissions from their assigned roles." }
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
                console.log('Roles management page loaded');
            })();
            "#
        }
    }
}

pub fn get_template() -> Template {
    Template::build("admin/roles").view(roles_management_page).build()
}
