use crate::{errors::*, reactor::Reactor};
#[cfg(engine)]
use crate::{i18n::Translator, reactor::RenderMode, state::TemplateState};
use fmterr::fmt_err;
use serde::{Deserialize, Serialize};
#[cfg(any(client, doc))]
use std::sync::Arc;
#[cfg(any(client, doc))]
use sycamore::prelude::try_use_context;
use sycamore::{prelude::view, utils::hydrate::with_no_hydration_context, View};

/// The error handling system of an app. In Perseus, errors come in several
/// forms, all of which must be handled. This system provides a way to do this
/// automatically, maximizing your app's error tolerance, including against
/// panics.
pub struct ErrorViews {
    /// The central function that parses the error provided and returns a tuple
    /// of views to deal with it: the first view is the document metadata,
    /// and the second the body of the error.
    #[allow(clippy::type_complexity)]
    handler: Box<dyn Fn(ClientError, ErrorContext, ErrorPosition) -> (View, View) + Send + Sync>,
    /// A function for determining if a subsequent load error should occupy the
    /// entire page or not.
    subsequent_load_determinant: Box<dyn Fn(&ClientError) -> bool + Send + Sync>,
    /// A verbatim copy of the user's handler, intended for panics.
    #[cfg(any(client, doc))]
    #[allow(clippy::type_complexity)]
    panic_handler: Arc<
        dyn Fn(ClientError, ErrorContext, ErrorPosition) -> (View<SsrNode>, View) + Send + Sync,
    >,
}

impl std::fmt::Debug for ErrorViews {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorViews").finish_non_exhaustive()
    }
}

impl ErrorViews {
    /// Creates an error handling system for your app with the given handler function.
    pub fn new(
        handler: impl Fn(ClientError, ErrorContext, ErrorPosition) -> (View, View)
            + Send
            + Sync
            + Clone
            + 'static,
    ) -> Self {
        #[allow(clippy::redundant_clone)]
        Self {
            handler: Box::new(handler.clone()),
            subsequent_load_determinant: Box::new(|err| match err {
                ClientError::ServerError { .. } => true,
                _ => false,
            }),
            #[cfg(any(client, doc))]
            panic_handler: Arc::new(handler),
        }
    }

    pub fn subsequent_load_determinant_fn(
        &mut self,
        val: impl Fn(&ClientError) -> bool + Send + Sync + 'static,
    ) -> &mut Self {
        self.subsequent_load_determinant = Box::new(val);
        self
    }

    #[cfg(any(client, doc))]
    pub(crate) fn subsequent_err_should_be_popup(&self, err: &ClientError) -> bool {
        !(self.subsequent_load_determinant)(err)
    }

    pub fn unlocalized_development_default() -> Self {
        Self::new(|err, _, pos| match err {
            ClientError::ServerError { status, .. } if status == 404 => (
                view! {
                    title { "Page not found" }
                },
                view! {
                    div(style = "display: flex; justify-content: center; align-items: center; height: 95vh; width: 100%;") {
                        main(style = "display: flex; flex-direction: column; border: 1px solid black; border-radius: 0.5rem; max-width: 36rem; margin: 1rem;") {
                            h3(style = "font-size: 1.5rem; line-height: 2rem; font-weight: 700; width: 100%; padding-bottom: 1rem; border-bottom: 1px solid black; margin-top: 1rem; margin-bottom: 1rem;") {
                                span(style = "padding-left: 1rem;") { "Page not found!" }
                            }
                            div(style = "padding: 1rem; padding-top: 0; margin-top: 1rem; margin-bottom: 1rem;") {
                                span {
                                    "Uh-oh, that page doesn't seem to exist! Perhaps you forgot to add it to your "
                                    code { "PerseusApp" }
                                    "?"
                                }
                            }
                        }
                    }
                },
            ),
            ClientError::Panic(panic_msg) => (
                View::empty(),
                view! {
                    div(style = "position: fixed; bottom: 0; right: 0; background-color: #f87171; color: white; margin: 1rem; border-radius: 0.5rem; max-width: 30rem;") {
                        h2(style = "font-size: 1.5rem; line-height: 2rem; font-weight: 700; width: 100%; padding-bottom: 1rem; border-bottom: 1px solid white; margin-top: 1rem; margin-bottom: 1rem;") {
                            span(style = "padding-left: 1rem;") { "Critical error!" }
                        }
                        div(style = "padding: 1rem; padding-top: 0; margin-top: 1rem;") {
                            p { "Your app has panicked! You can see the panic message below." }
                            pre(style = "background-color: #f59e0b; padding: 1rem; margin-top: 1rem; border-radius: 0.5rem; white-space: pre-wrap; word-wrap: break-word;") {
                                (panic_msg)
                            }
                            (if panic_msg.contains("cannot modify the panic hook from a panicking thread") {
                                view! {
                                    p {
                                        i { "It looks like the error is about the panicking hook itself, which means the original panic has been overridden, possibly by hot state reloading in development mode. Try reloading the page." }
                                    }
                                }
                            } else {
                                View::empty()
                            })
                        }
                    }
                },
            ),
            err => {
                let err_msg = fmt_err(&err);
                let inner_view = view! {
                    div(style = "background-color: #f87171; color: white; margin: 1rem; border-radius: 0.5rem; max-width: 30rem;") {
                        h2(style = "font-size: 1.5rem; line-height: 2rem; font-weight: 700; width: 100%; padding-bottom: 1rem; border-bottom: 1px solid white; margin-top: 1rem; margin-bottom: 1rem;") {
                            span(style = "padding-left: 1rem;") { "Error!" }
                        }
                        div(style = "padding: 1rem; padding-top: 0; margin-top: 1rem;") {
                            p { "Your app encountered an error, you can see the details below." }
                            pre(style = "background-color: #f59e0b; padding: 1rem; margin-top: 1rem; border-radius: 0.5rem; white-space: pre-wrap; word-break: break-word;") {
                                (err_msg)
                            }
                        }
                    }
                };

                (
                    view! { title { "Error" } },
                    match pos {
                        ErrorPosition::Page => view! {
                            div(style = "display: flex; flex-direction: column; justify-content: center; align-items: center; height: 95vh; width: 100%;") {
                                (inner_view)
                            }
                        },
                        ErrorPosition::Popup => view! {
                            div(style = "position: fixed; bottom: 0; right: 0; display: flex; justify-content: center; align-items: center;") {
                                (inner_view)
                            }
                        },
                        ErrorPosition::Widget => view! {
                            div(style = "display: flex; flex-direction: column;") {
                                (inner_view)
                            }
                        },
                    },
                )
            }
        })
    }
}

#[cfg(any(client, doc))]
impl ErrorViews {
    pub(crate) fn handle(&self, err: ClientError, pos: ErrorPosition) -> (String, View) {
        let reactor = try_use_context::<Reactor>();
        let info = match reactor {
            Some(reactor) => match reactor.try_get_translator() {
                Some(_) => ErrorContext::Full,
                None => ErrorContext::WithReactor,
            },
            None => ErrorContext::Static,
        };

        let (head_view, body_view) = (self.handler)(err, info, pos);
        let head_str = sycamore::render_to_string(|| with_no_hydration_context(|| head_view));

        (head_str, body_view)
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn take_panic_handler(
        &mut self,
    ) -> Arc<dyn Fn(ClientError, ErrorContext, ErrorPosition) -> (View<SsrNode>, View) + Send + Sync>
    {
        std::mem::replace(&mut self.panic_handler, Arc::new(|_, _, _| unreachable!()))
    }
}

#[cfg(engine)]
impl ErrorViews {
    pub(crate) fn render_to_string(
        &self,
        err: ServerErrorData,
        translator: Option<&Translator>,
    ) -> (String, String) {
        let reactor = Reactor::engine(TemplateState::empty(), RenderMode::Error, translator);

        let err_cx = match translator {
            Some(_) => ErrorContext::FullNoGlobal,
            None => ErrorContext::WithReactor,
        };

        reactor.add_self_to_cx();

        let (head_view, body_view) = (self.handler)(
            ClientError::ServerError {
                status: err.status,
                message: err.msg,
            },
            err_cx,
            ErrorPosition::Page,
        );

        let head_str = sycamore::render_to_string(|| with_no_hydration_context(|| head_view));
        let body_str = sycamore::render_to_string(|| body_view);

        (head_str, body_str)
    }
}

impl ErrorViews {
    pub(crate) fn handle_widget(&self, err: ClientError) -> View {
        let (_head, body) = (self.handler)(err, ErrorContext::Full, ErrorPosition::Widget);
        body
    }
}

// Rest of the enums remain the same...
#[derive(Debug, Clone, Copy)]
pub enum ErrorContext {
    Static,
    WithReactor,
    FullNoGlobal,
    Full,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorPosition {
    Page,
    Widget,
    Popup,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerErrorData {
    pub(crate) status: u16,
    pub(crate) msg: String,
}

#[cfg(debug_assertions)]
impl Default for ErrorViews {
    fn default() -> Self {
        Self::unlocalized_development_default()
    }
}
