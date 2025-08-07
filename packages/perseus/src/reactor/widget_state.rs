#[cfg(any(client, doc))]
use std::sync::Arc;

use super::Reactor;
use crate::{
    error_views::ServerErrorData,
    errors::{ClientError, ClientInvariantError},
    path::*,
    state::{AnyFreeze, MakeRx, MakeUnrx, PssContains, TemplateState, UnreactiveState},
};
use serde::{de::DeserializeOwned, Serialize};
use sycamore::prelude::View;
// use sycamore::{prelude::create_signal, view::View};

#[cfg(any(client, doc))]
use crate::template::PreloadInfo;
#[cfg(any(client, doc))]
use sycamore_futures::spawn_local_scoped;

impl Reactor {
    /// Gets the view for the given widget path. This will perform
    /// asynchronous fetching as needed to fetch state from the server, and
    /// will also handle engine-side state pass-through. This function will
    /// propagate as many errors as it can, though those occurring inside a
    /// `spawn_local_scoped` environment will be resolved to error views.
    ///
    /// This is intended for use with widgets that use reactive state. See
    /// `.get_unreactive_widget_view()` for widgets that use unreactive
    /// state.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)] // Internal function
    pub(crate) fn get_widget_view<S, F, P: Clone + 'static>(
        &self,
        path: PathMaybeWithLocale,
        #[allow(unused_variables)] caller_path: PathMaybeWithLocale,
        #[cfg(any(client, doc))] capsule_name: String,
        template_state: TemplateState, // Empty on the browser-side
        props: P,
        #[cfg(any(client, doc))] preload_info: PreloadInfo,
        view_fn: F,
        #[cfg(any(client, doc))] fallback_fn: &Arc<dyn Fn(P) -> View + Send + Sync>,
    ) -> Result<View, ClientError>
    where
        F: Fn(S::Rx, P) -> View + Send + Sync + 'static,
        S: MakeRx + Serialize + DeserializeOwned + 'static,
        S::Rx: MakeUnrx<Unrx = S> + AnyFreeze + Clone,
    {
        match self.get_widget_state_no_fetch::<S>(&path, template_state)? {
            Some(intermediate_state) => {
                // We can directly use the state without creating a ref
                let view = view_fn(intermediate_state, props);
                Ok(view)
            }
            // We need to asynchronously fetch the state from the server, which doesn't work
            // ergonomically with the rest of the code, so we just break out entirely
            #[cfg(any(client, doc))]
            None => {
                let view = create_signal(View::empty());
                let fallback_fn = fallback_fn.clone();

                // We'll render the fallback view in the meantime (which `PerseusApp`
                // guarantees to be defined for capsules)
                view.set((fallback_fn)(props.clone()));

                // Note: this uses the current scope, meaning the fetch will be aborted if the user
                // goes to another page
                let capsule_name = capsule_name.clone();
                let view_clone = view;
                let self_clone = self.clone(); // Assuming Reactor implements Clone

                spawn_local_scoped(async move {
                    // Any errors that occur in here will be converted into proper error
                    // views using the reactor
                    let final_view = {
                        let path_without_locale =
                            PathWithoutLocale(match preload_info.locale.as_str() {
                                "xx-XX" => path.to_string(),
                                locale => path
                                    .strip_prefix(&format!("{}/", locale))
                                    .unwrap()
                                    .to_string(),
                            });
                        // We can simply use the preload system to perform the fetching
                        match self_clone
                            .state_store
                            .preload(
                                &path_without_locale,
                                &preload_info.locale,
                                &capsule_name,
                                preload_info.was_incremental_match,
                                false, // Don't use the route preloading system
                                true,  // This is a widget
                            )
                            .await
                        {
                            // If that succeeded, we can use the same logic as before, and
                            // we know it can't return `Ok(None)`
                            // this time! We're in the browser, so we can just use an empty
                            // template state, rather than
                            // cloning the one we've been given (which is empty anyway).
                            Ok(()) => match self_clone
                                .get_widget_state_no_fetch::<S>(&path, TemplateState::empty())
                            {
                                Ok(Some(intermediate_state)) => {
                                    // Declare the relationship between the widget and its
                                    // caller
                                    self_clone
                                        .state_store
                                        .declare_dependency(&path, &caller_path);

                                    view_fn(intermediate_state, props)
                                }
                                Ok(None) => unreachable!(),
                                Err(err) => self_clone.error_views.handle_widget(err),
                            },
                            Err(err) => self_clone.error_views.handle_widget(err),
                        }
                    };

                    view_clone.set(final_view);
                });

                Ok(view! { (*view.get()) })
            }
            // On the engine-side, this is impossible (we cannot be instructed to fetch)
            #[cfg(engine)]
            None => unreachable!(),
        }
    }

    /// Gets the view for the given widget path. This will perform
    /// asynchronous fetching as needed to fetch state from the server, and
    /// will also handle engine-side state pass-through. This function will
    /// propagate as many errors as it can, though those occurring inside a
    /// `spawn_local_scoped` environment will be resolved to error views.
    ///
    /// This is intended for use with widgets that use unreactive state. See
    /// `.get_widget_view()` for widgets that use reactive state.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)] // Internal function
    pub(crate) fn get_unreactive_widget_view<F, S, P: Clone + 'static>(
        &self,
        path: PathMaybeWithLocale,
        #[allow(unused_variables)] caller_path: PathMaybeWithLocale,
        #[cfg(any(client, doc))] capsule_name: String,
        template_state: TemplateState, // Empty on the browser-side
        props: P,
        #[cfg(any(client, doc))] preload_info: PreloadInfo,
        view_fn: F,
        #[cfg(any(client, doc))] fallback_fn: &Arc<dyn Fn(P) -> View + Send + Sync>,
    ) -> Result<View, ClientError>
    where
        F: Fn(S, P) -> View + Send + Sync + 'static,
        S: MakeRx + Serialize + DeserializeOwned + UnreactiveState + 'static,
        <S as MakeRx>::Rx: AnyFreeze + Clone + MakeUnrx<Unrx = S>,
    {
        match self.get_widget_state_no_fetch::<S>(&path, template_state)? {
            Some(intermediate_state) => {
                // We go back from the unreactive state type wrapper to the base type (since
                // it's unreactive)
                let view = view_fn(intermediate_state.make_unrx(), props);
                Ok(view)
            }
            // We need to asynchronously fetch the state from the server, which doesn't work
            // ergonomically with the rest of the code, so we just break out entirely
            #[cfg(any(client, doc))]
            None => {
                let view = create_signal(View::empty());
                let fallback_fn = fallback_fn.clone();

                // We'll render the fallback view in the meantime (which `PerseusApp`
                // guarantees to be defined for capsules)
                view.set((fallback_fn)(props.clone()));

                // Note: this uses the current scope, meaning the fetch will be aborted if the user
                // goes to another page
                let capsule_name = capsule_name.clone();
                let view_clone = view;
                let self_clone = self.clone(); // Assuming Reactor implements Clone

                spawn_local_scoped(async move {
                    // Any errors that occur in here will be converted into proper error
                    // views using the reactor
                    let final_view = {
                        let path_without_locale =
                            PathWithoutLocale(match preload_info.locale.as_str() {
                                "xx-XX" => path.to_string(),
                                locale => path
                                    .strip_prefix(&format!("{}/", locale))
                                    .unwrap()
                                    .to_string(),
                            });
                        // We can simply use the preload system to perform the fetching
                        match self_clone
                            .state_store
                            .preload(
                                &path_without_locale,
                                &preload_info.locale,
                                &capsule_name,
                                preload_info.was_incremental_match,
                                false, // Don't use the route preloading system
                                true,  // This is a widget
                            )
                            .await
                        {
                            // If that succeeded, we can use the same logic as before, and
                            // we know it can't return `Ok(None)`
                            // this time! We're in the browser, so we can just use an empty
                            // template state, rather than
                            // cloning the one we've been given (which is empty anyway).
                            Ok(()) => match self_clone
                                .get_widget_state_no_fetch::<S>(&path, TemplateState::empty())
                            {
                                Ok(Some(intermediate_state)) => {
                                    // Declare the relationship between the widget and its
                                    // caller
                                    self_clone
                                        .state_store
                                        .declare_dependency(&path, &caller_path);

                                    view_fn(intermediate_state.make_unrx(), props)
                                }
                                Ok(None) => unreachable!(),
                                Err(err) => self_clone.error_views.handle_widget(err),
                            },
                            Err(err) => self_clone.error_views.handle_widget(err),
                        }
                    };

                    view_clone.set(final_view);
                });

                Ok(view! { (*view.get()) })
            }
            // On the engine-side, this is impossible (we cannot be instructed to fetch)
            #[cfg(engine)]
            None => unreachable!(),
        }
    }

    /// Gets the state for the given widget. This will return `Ok(None)`, if the
    /// state needs to be fetched from the server.
    ///
    /// This will check against the active and frozen states, but it will
    /// extract state from the preload system on an initial load (as this is
    /// how widget states are loaded in). Note that this also acts as a
    /// general interface with the preload system for widgets, the role
    /// of which is fulfilled for pages by the subsequent load system.
    ///
    /// On the engine-side, this will use the given template state (which will
    /// be passed through, unlike on the browser-side, where it will always
    /// be empty).
    pub(crate) fn get_widget_state_no_fetch<S>(
        &self,
        url: &PathMaybeWithLocale,
        server_state: TemplateState,
    ) -> Result<Option<S::Rx>, ClientError>
    where
        S: MakeRx + Serialize + DeserializeOwned + 'static,
        S::Rx: MakeUnrx<Unrx = S> + AnyFreeze + Clone,
    {
        if let Some(held_state) = self.get_held_state::<S>(url, true)? {
            Ok(Some(held_state))
        } else if cfg!(client) {
            // On the browser-side, the given server state is empty, and we need to check
            // the preload
            match self.state_store.contains(url) {
                // This implies either user preloading, or initial load automatic preloading
                // from `__PERSEUS_INITIAL_WIDGET_STATES`
                PssContains::Preloaded => {
                    let page_data = self.state_store.get_preloaded(url).unwrap();
                    // Register an empty head
                    self.state_store.add_head(url, String::new(), true);
                    // And reactivize the state for registration
                    let typed_state = TemplateState::from_value(page_data.state)
                        .change_type::<Result<S, ServerErrorData>>();
                    // This attempts a deserialization from a `Value`, which could fail
                    let unrx_res = typed_state
                        .into_concrete()
                        .map_err(|err| ClientInvariantError::InvalidState { source: err })?;
                    match unrx_res {
                        Ok(unrx) => {
                            let rx = unrx.make_rx();
                            // Add that to the state store as the new active state
                            self.state_store.add_state(url, rx.clone(), false)?;

                            Ok(Some(rx))
                        }
                        // This would occur if there were an error in the widget that were
                        // transmitted to us
                        Err(ServerErrorData { status, msg }) => Err(ClientError::ServerError {
                            status,
                            message: msg,
                        }),
                    }
                }
                // We need to fetch the state from the server, which will require
                // asynchronicity, so bail out of this function, which is
                // not equipped for that
                PssContains::None => Ok(None),
                // Widgets have no heads, and must always be registered with a state
                PssContains::Head | PssContains::HeadNoState => {
                    Err(ClientInvariantError::InvalidWidgetPssEntry.into())
                }
                // These would have been caught by `get_held_state()` above
                PssContains::All | PssContains::State => unreachable!(),
            }
        }
        // On the engine-side, the given server state is correct, and `get_held_state()`
        // will definitionally return `Ok(None)`
        else if server_state.is_empty() {
            // This would be quite concerning...
            Err(ClientInvariantError::NoState.into())
        } else {
            // Fall back to the state we were given, first
            // giving it a type (this just sets a phantom type parameter)
            let typed_state = server_state.change_type::<S>();
            // This attempts a deserialization from a `Value`, which could fail
            let unrx = typed_state
                .into_concrete()
                .map_err(|err| ClientInvariantError::InvalidState { source: err })?;
            let rx = unrx.make_rx();
            // Add that to the state store as the new active state
            self.state_store.add_state(url, rx.clone(), false)?;

            Ok(Some(rx))
        }
    }
}
