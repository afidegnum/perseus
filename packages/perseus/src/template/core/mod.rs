// This module contains the primary shared logic in Perseus, and is broken up to
// avoid a 2000-line file.

mod getters;
mod renderers;
mod setters;
mod utils;
// These are broken out because of state-management closure wrapping
mod entity;
mod state_setters;

use std::ops::Deref;

pub(crate) use entity::{Entity, EntityMap, Forever};
pub(crate) use utils::*;

#[cfg(engine)]
use super::fn_types::*;
use super::TemplateFn;
#[cfg(engine)]
use crate::utils::ComputedDuration;
use sycamore::prelude::*;

/// A single template in an app. Each template is comprised of a Sycamore view,
/// a state type, and some functions involved with generating that state. Pages
/// can then be generated from particular states. For instance, a single `docs`
/// template could have a state `struct` that stores a title and some content,
/// which could then render as many pages as desired.
///
/// You can read more about the templates system [here](https://framesurge.sh/perseus/en-US/docs/next/core-principles).
#[derive(Debug)]
pub struct Template {
    /// The inner entity.
    pub(crate) inner: Entity,
}
impl Deref for Template {
    type Target = TemplateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl Template {
    /// Creates a new [`TemplateInner`] (a builder for [`Template`]s). By
    /// default, this has absolutely no associated data, and, if rendered,
    /// it would result in a blank screen. You can call methods like
    /// `.view()` on this, and you should eventually call `.build()` to turn
    /// it into a full template.
    pub fn build(path: &str) -> TemplateInner {
        TemplateInner::new(path)
    }
}

/// The internal representation of a Perseus template, with all the methods
/// involved in creating and managing it. As this `struct` is not `Clone`,
/// it will almost always appear wrapped in a full [`Template`], which allows
/// cloning and passing the template around arbitrarily. As that dereferences
/// to this, you will be able to use any of the methods on this `struct` on
/// [`Template`].
pub struct TemplateInner {
    /// The path to the root of the template. Any build paths will be inserted
    /// under this.
    path: String,
    /// A function that will render your template. This will be provided the
    /// rendered properties, and will be used whenever your template needs
    /// to be prerendered in some way. This should be very similar to the
    /// function that hydrates your template on the client side.
    /// This will be executed inside `sycamore::render_to_string`, and should
    /// return a `View`. This takes an `Option<Props>`
    /// because otherwise efficient typing is almost impossible for templates
    /// without any properties (solutions welcome in PRs!).
    // Public to the crate so capsules can shadow these functions for property support
    pub(crate) view: TemplateFn,
    /// A function that will be used to populate the document's `<head>` with
    /// metadata such as the title. This will be passed state in
    /// the same way as `template`, but will always be rendered to a string,
    /// which will then be interpolated directly into the `<head>`,
    /// so reactivity here will not work!
    #[cfg(engine)]
    pub(crate) head: Option<HeadFn>,
    /// A function to be run when the server returns an HTTP response. This
    /// should return headers for said response, given the template's state.
    /// The most common use-case of this is to add cache control that respects
    /// revalidation. This will only be run on successful responses, and
    /// does have the power to override existing headers. By default, this will
    /// create sensible cache control headers.
    #[cfg(engine)]
    pub(crate) set_headers: Option<SetHeadersFn>,
    /// A function that generates the information to begin building a template.
    /// This is responsible for generating all the paths that will built for
    /// that template at build-time (which may later be extended with
    /// incremental generation), along with the generation of any extra
    /// state that may be collectively shared by other state generating
    /// functions.
    #[cfg(engine)]
    get_build_paths: Option<GetBuildPathsFn>,
    /// Defines whether or not any new paths that match this template will be
    /// prerendered and cached in production. This allows you to
    /// have potentially billions of templates and retain a super-fast build
    /// process. The first user will have an ever-so-slightly slower
    /// experience, and everyone else gets the benefits afterwards. This
    /// requires `get_build_paths`. Note that the template root will NOT
    /// be rendered on demand, and must be explicitly defined if it's wanted. It
    /// can use a different template.
    #[cfg(engine)]
    incremental_generation: bool,
    /// A function that gets the initial state to use to prerender the template
    /// at build time. This will be passed the path of the template, and
    /// will be run for any sub-paths.
    #[cfg(engine)]
    get_build_state: Option<GetBuildStateFn>,
    /// A function that will run on every request to generate a state for that
    /// request. This allows server-side-rendering. This can be used with
    /// `get_build_state`, though custom amalgamation logic must be provided.
    #[cfg(engine)]
    get_request_state: Option<GetRequestStateFn>,
    /// A function to be run on every request to check if a template prerendered
    /// at build-time should be prerendered again. If used with
    /// `revalidate_after`, this function will only be run after that time
    /// period. This function will not be parsed anything specific to the
    /// request that invoked it.
    #[cfg(engine)]
    should_revalidate: Option<ShouldRevalidateFn>,
    /// A length of time after which to prerender the template again. The given
    /// duration will be waited for, and the next request after it will lead
    /// to a revalidation. Note that, if this is used with incremental
    /// generation, the counter will only start after the first render
    /// (meaning if you expect a weekly re-rendering cycle for all pages,
    /// they'd likely all be out of sync, you'd need to manually implement
    /// that with `should_revalidate`).
    #[cfg(engine)]
    revalidate_after: Option<ComputedDuration>,
    /// Custom logic to amalgamate potentially different states generated at
    /// build and request time. This is only necessary if your template uses
    /// both `build_state` and `request_state`. If not specified and both are
    /// generated, request state will be prioritized.
    #[cfg(engine)]
    amalgamate_states: Option<AmalgamateStatesFn>,
    /// Whether or not this template is actually a capsule. This impacts
    /// significant aspects of internal handling.
    ///
    /// There is absolutely no circumstance in which you should ever change
    /// this. Ever. You will break your app. Always.
    pub is_capsule: bool,
    /// Whether or not this template's pages can have their builds rescheduled
    /// from build-time to request-time if they depend on capsules that aren't
    /// ready with state at build-time. This is included as a precaution to
    /// seemingly erroneous performance changes with pages. If rescheduling
    /// is needed and it hasn't been explicitly allowed, an error will be
    /// returned from the build process.
    pub(crate) can_be_rescheduled: bool,
}
impl std::fmt::Debug for TemplateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template")
            .field("path", &self.path)
            .field("is_capsule", &self.is_capsule)
            .finish()
    }
}
impl TemplateInner {
    /// An internal creator for new inner templates. This is wrapped by
    /// `Template::build` and `Capsule::build`.
    fn new(path: impl Into<String> + std::fmt::Display) -> Self {
        Self {
            path: path.to_string(),
            // In Sycamore 0.9.1, we no longer need to return a scope disposer
            view: Box::new(|_, _, _| Ok(View::new())),
            // Unlike `template`, this may not be set at all (especially in very simple apps)
            #[cfg(engine)]
            head: None,
            #[cfg(engine)]
            set_headers: None,
            #[cfg(engine)]
            get_build_paths: None,
            #[cfg(engine)]
            incremental_generation: false,
            #[cfg(engine)]
            get_build_state: None,
            #[cfg(engine)]
            get_request_state: None,
            #[cfg(engine)]
            should_revalidate: None,
            #[cfg(engine)]
            revalidate_after: None,
            #[cfg(engine)]
            amalgamate_states: None,
            // There is no mechanism to set this to `true`, except through the `Capsule` struct
            is_capsule: false,
            can_be_rescheduled: false,
        }
    }
    /// Builds a full [`Template`] from this [`TemplateInner`], consuming it in
    /// the process. Once called, the template cannot be modified anymore,
    /// and it will be placed into a smart pointer, allowing it to be cloned
    /// freely with minimal costs.
    ///
    /// You should call this just before you return your template.
    pub fn build(self) -> Template {
        Template {
            inner: Entity::from(self),
        }
    }
}

// The engine needs to know whether or not to use hydration, this is how we pass
// those feature settings through
/// An alias for `DomNode` or `HydrateNode`, depending on the feature flags
/// enabled.
#[cfg(all(not(feature = "hydrate"), any(client, doc)))]
pub(crate) type BrowserNodeType = sycamore::web::DomNode;
/// An alias for `DomNode` or `HydrateNode`, depending on the feature flags
/// enabled.
#[cfg(all(feature = "hydrate", any(client, doc)))]
pub(crate) type BrowserNodeType = sycamore::web::HydrateNode;
