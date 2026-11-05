//! Viewport-scoped modals requested by article illustrations.
//!
//! ## Usage
//!
//! Keep interactive figure dialogs above the complete article viewport.

use tessera_components::dialog::dialog_provider;
use tessera_ui::{Callback, Dp, RenderSlot, State, tessera};

use crate::res::I18N;

#[derive(Clone, PartialEq)]
struct Request {
    open: State<bool>,
    full_screen: bool,
    accessibility_label: String,
    on_close_request: Callback,
    content: tessera_ui::RenderSlotWith<tessera_components::dialog::DialogScope>,
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct ArticleModalState {
    request: Option<Request>,
}

pub(crate) type ArticleModalController = State<ArticleModalState>;

#[tessera]
pub(crate) fn article_modal_host(active: ArticleModalController, content: RenderSlot) {
    let request = active.get().request;
    let open = request
        .as_ref()
        .is_some_and(|request| request.open.is_alive() && request.open.get());
    dialog_provider()
        .is_open(open)
        .full_screen(request.as_ref().is_some_and(|request| request.full_screen))
        .padding(Dp::ZERO)
        .accessibility_label(
            I18N.text(request.as_ref().map_or("ui-article-dialog", |request| {
                request.accessibility_label.as_str()
            })),
        )
        .on_close_request(move || {
            if let Some(request) = active.get().request
                && request.open.is_alive()
            {
                request.on_close_request.call();
            }
        })
        .main_content(move || {
            content.render();
        })
        .dialog_content(move |dialog_scope| {
            if let Some(request) = active.get().request
                && request.open.is_alive()
            {
                request.content.render(dialog_scope);
            }
        });
}

#[tessera]
pub(crate) fn hosted_dialog(
    open: State<bool>,
    full_screen: Option<bool>,
    #[prop(into)] accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    main_content: Option<RenderSlot>,
    dialog_content: Option<tessera_ui::RenderSlotWith<tessera_components::dialog::DialogScope>>,
    modal_controller: Option<ArticleModalController>,
) {
    let full_screen = full_screen.unwrap_or(false);
    let accessibility_label = accessibility_label.unwrap_or_else(|| "ui-article-dialog".into());
    let on_close_request = on_close_request.unwrap_or_default();
    let main_content = main_content.unwrap_or_default();
    let dialog_content = if let Some(content) = dialog_content {
        content
    } else {
        tessera_ui::RenderSlotWith::new(|_| {})
    };
    if let Some(active) = modal_controller {
        if open.get()
            && active.with(|active| {
                active
                    .request
                    .as_ref()
                    .is_none_or(|request| !request.open.is_alive() || !request.open.get())
            })
        {
            active.set(ArticleModalState {
                request: Some(Request {
                    open,
                    full_screen,
                    accessibility_label,
                    on_close_request,
                    content: dialog_content,
                }),
            });
        }
        main_content.render();
    } else {
        dialog_provider()
            .is_open(open.get())
            .full_screen(full_screen)
            .padding(Dp::ZERO)
            .accessibility_label(I18N.text(accessibility_label))
            .on_close_request_shared(on_close_request)
            .main_content_shared(main_content)
            .dialog_content_shared(dialog_content);
    }
}
