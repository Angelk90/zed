use super::{SlashCommand, SlashCommandOutput};
use anyhow::Result;
use fuzzy::PathMatch;
use gpui::{AppContext, Model, RenderOnce, SharedString, Task, WeakView};
use language::LspAdapterDelegate;
use project::{PathMatchCandidateSet, Project};
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};
use ui::{prelude::*, ButtonLike, ElevationIndex};
use workspace::Workspace;

pub(crate) struct FileSlashCommand {
    project: Model<Project>,
}

impl FileSlashCommand {
    pub fn new(project: Model<Project>) -> Self {
        Self { project }
    }

    fn search_paths(
        &self,
        query: String,
        cancellation_flag: Arc<AtomicBool>,
        cx: &mut AppContext,
    ) -> Task<Vec<PathMatch>> {
        let worktrees = self
            .project
            .read(cx)
            .visible_worktrees(cx)
            .collect::<Vec<_>>();
        let candidate_sets = worktrees
            .into_iter()
            .map(|worktree| {
                let worktree = worktree.read(cx);
                PathMatchCandidateSet {
                    snapshot: worktree.snapshot(),
                    include_ignored: worktree
                        .root_entry()
                        .map_or(false, |entry| entry.is_ignored),
                    include_root_name: true,
                    directories_only: false,
                }
            })
            .collect::<Vec<_>>();

        let executor = cx.background_executor().clone();
        cx.foreground_executor().spawn(async move {
            fuzzy::match_path_sets(
                candidate_sets.as_slice(),
                query.as_str(),
                None,
                false,
                100,
                &cancellation_flag,
                executor,
            )
            .await
        })
    }
}

impl SlashCommand for FileSlashCommand {
    fn name(&self) -> String {
        "file".into()
    }

    fn description(&self) -> String {
        "insert a file".into()
    }

    fn tooltip_text(&self) -> String {
        "insert file".into()
    }

    fn requires_argument(&self) -> bool {
        true
    }

    fn complete_argument(
        &self,
        query: String,
        cancellation_flag: Arc<AtomicBool>,
        cx: &mut AppContext,
    ) -> gpui::Task<Result<Vec<String>>> {
        let paths = self.search_paths(query, cancellation_flag, cx);
        cx.background_executor().spawn(async move {
            Ok(paths
                .await
                .into_iter()
                .map(|path_match| {
                    format!(
                        "{}{}",
                        path_match.path_prefix,
                        path_match.path.to_string_lossy()
                    )
                })
                .collect())
        })
    }

    fn run(
        self: Arc<Self>,
        argument: Option<&str>,
        _workspace: WeakView<Workspace>,
        _delegate: Arc<dyn LspAdapterDelegate>,
        cx: &mut WindowContext,
    ) -> Task<Result<SlashCommandOutput>> {
        let project = self.project.read(cx);
        let Some(argument) = argument else {
            return Task::ready(Err(anyhow::anyhow!("missing path")));
        };

        let path = PathBuf::from(argument);
        let abs_path = project.worktrees().find_map(|worktree| {
            let worktree = worktree.read(cx);
            let worktree_root_path = Path::new(worktree.root_name());
            let relative_path = path.strip_prefix(worktree_root_path).ok()?;
            worktree.absolutize(&relative_path).ok()
        });

        let Some(abs_path) = abs_path else {
            return Task::ready(Err(anyhow::anyhow!("missing path")));
        };

        let fs = project.fs().clone();
        let argument = argument.to_string();
        let text = cx.background_executor().spawn(async move {
            let content = fs.load(&abs_path).await?;
            let mut output = String::with_capacity(argument.len() + content.len() + 9);
            output.push_str("```");
            output.push_str(&argument);
            output.push('\n');
            output.push_str(&content);
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```");
            anyhow::Ok(output)
        });
        cx.foreground_executor().spawn(async move {
            let text = text.await?;
            Ok(SlashCommandOutput {
                text,
                render_placeholder: Arc::new(move |id, unfold, _cx| {
                    FilePlaceholder {
                        path: Some(path.clone()),
                        id,
                        unfold,
                    }
                    .into_any_element()
                }),
            })
        })
    }
}

#[derive(IntoElement)]
pub struct FilePlaceholder {
    pub path: Option<PathBuf>,
    pub id: ElementId,
    pub unfold: Arc<dyn Fn(&mut WindowContext)>,
}

impl RenderOnce for FilePlaceholder {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        let unfold = self.unfold;
        let title = if let Some(path) = self.path.as_ref() {
            SharedString::from(path.to_string_lossy().to_string())
        } else {
            SharedString::from("untitled")
        };

        ButtonLike::new(self.id)
            .style(ButtonStyle::Filled)
            .layer(ElevationIndex::ElevatedSurface)
            .child(Icon::new(IconName::File))
            .child(Label::new(title))
            .on_click(move |_, cx| unfold(cx))
    }
}
