//! Material component directory and article routes.

use tessera_components::{lazy_list::LazyListController, scrollable::ScrollableController};
use tessera_shard::{RouterController, shard};
use tessera_ui::{CallbackWith, Dp, Model, ModelInit, Px, State};

use crate::app::AppLiveSessions;

use super::material_spec::{component_article, component_directory};

struct DirectoryModel {
    scroll: State<ScrollableController>,
    list: State<LazyListController>,
    width: State<Dp>,
    height: State<Px>,
}

impl ModelInit for DirectoryModel {
    fn init() -> Self {
        Self {
            scroll: State::new(ScrollableController::new()),
            list: State::new(LazyListController::new()),
            width: State::new(Dp(900.0)),
            height: State::new(Px(1)),
        }
    }
}

#[shard]
pub fn material_components_page(
    router: State<RouterController>,
    model: Model<DirectoryModel>,
    sessions: Option<AppLiveSessions>,
) {
    component_directory(
        model.scroll,
        model.list,
        model.width,
        model.height,
        CallbackWith::new(move |index| {
            router
                .with_mut(|router| router.push(MaterialArticlePageDestination { index, sessions }));
        }),
    );
}

#[shard]
pub fn material_article_page(
    router: State<RouterController>,
    index: usize,
    sessions: Option<AppLiveSessions>,
) {
    component_article(index, router).sessions_optional(sessions);
}

pub(crate) fn current_article(router: &RouterController) -> Option<usize> {
    (router.last()? as &dyn std::any::Any)
        .downcast_ref::<MaterialArticlePageDestination>()
        .map(|article| article.index)
}
