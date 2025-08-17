use rspack_core::{
  ApplyContext, Compilation, CompilationProcessAssets, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_sources::{ConcatSource, RawStringSource, SourceExt};

/// A plugin that adds a banner to the output `main.js`.
#[derive(Debug)]
#[plugin]
pub struct AddFooterCommentPlugin {
  comment: String,
}

impl AddFooterCommentPlugin {
  pub fn new(comment: String) -> Self {
    Self::new_inner(comment)
  }
}

#[plugin_hook(CompilationProcessAssets for AddFooterCommentPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONS, tracing = false)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut updates = vec![];
  // 遍历所有chunk文件
  for chunk in compilation.chunk_by_ukey.values() {
    for file in chunk.files() {
      updates.push(file.clone());
    }
  }
  for file in updates {
    let _ = compilation.update_asset(file.as_str(), |old, info| {
      let comment = format!("\n/* {} */", self.comment);

      // 在文件末尾追加注释
      let new_source = ConcatSource::new([
        old.to_owned(),
        RawStringSource::from_static("\n").boxed(),
        RawStringSource::from(comment).boxed(),
      ])
      .boxed();
      Ok((new_source, info))
    });
  }

  Ok(())
}

impl Plugin for AddFooterCommentPlugin {
  fn name(&self) -> &'static str {
    "AddFooterCommentPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
