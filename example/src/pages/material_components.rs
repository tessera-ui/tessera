//! Material component directory and article routes.

use tessera_components::{lazy_list::LazyListController, scrollable::ScrollableController};
use tessera_shard::{RouterController, shard};
use tessera_ui::{CallbackWith, Dp, Px, State, StateOwner, remember};

use crate::app::AppLiveSessions;

use super::material_spec::{component_article, component_directory};

struct DirectoryState {
    scroll: State<ScrollableController>,
    list: State<LazyListController>,
    width: State<Dp>,
    height: State<Px>,
    owner: StateOwner,
}

impl Default for DirectoryState {
    fn default() -> Self {
        let owner = StateOwner::new();
        Self {
            scroll: owner.state_with_key("scroll", ScrollableController::new),
            list: owner.state_with_key("list", LazyListController::new),
            width: owner.state_with_key("width", || Dp(900.0)),
            height: owner.state_with_key("height", || Px(1)),
            owner,
        }
    }
}

#[shard(state = DirectoryState)]
pub fn material_components_page(
    #[router] router: State<RouterController>,
    sessions: Option<AppLiveSessions>,
) {
    // Navigation can remove the route during input dispatch, before its nodes detach.
    remember(|| state.with(|state| state.owner.clone()));
    let (scroll, list, width, height) =
        state.with(|state| (state.scroll, state.list, state.width, state.height));
    component_directory(
        scroll,
        list,
        width,
        height,
        CallbackWith::new(move |index| {
            router
                .with_mut(|router| router.push(MaterialArticlePageDestination { index, sessions }));
        }),
    );
}

#[shard]
pub fn material_article_page(
    #[router] router: State<RouterController>,
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
