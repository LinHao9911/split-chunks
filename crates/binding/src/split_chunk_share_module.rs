use rspack_core::{
  incremental::Mutation, ApplyContext, Compilation, CompilationProcessAssets, CompilerOptions,
  Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

/// A plugin that adds a banner to the output `main.js`.
#[derive(Debug)]
#[plugin]
pub struct SplitChunkShareModule {
  need_split_chunk_list: Vec<String>,
}

impl SplitChunkShareModule {
  pub fn new(need_split_chunk_list: Vec<String>) -> Self {
    Self::new_inner(need_split_chunk_list)
  }
}

#[plugin_hook(CompilationProcessAssets for SplitChunkShareModule, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONS, tracing = false)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let need_spilt_chunk_key_value_list = &mut vec![];
  {
    for neet_split_chunk_name in self.need_split_chunk_list.clone() {
      let chunk_key_value = compilation
        .chunk_by_ukey
        .keys()
        .cloned()
        .find(|chunk_ukey| {
          let chunk = compilation.chunk_by_ukey.get(chunk_ukey).unwrap();
          chunk.name().map(|s| s.to_string()) == Some(neet_split_chunk_name.clone())
        });

      if let Some(chunk_ukey) = chunk_key_value {
        need_spilt_chunk_key_value_list.push(chunk_ukey);
      } else {
        // 如果chunk不存在，报错
        return Err(rspack_error::error!(format!(
          "chunk {} not found",
          neet_split_chunk_name
        )));
      }
    }
  }

  let need_split_chunk_module_list = &mut vec![];
  let module_graph = &mut compilation.get_module_graph();

  for chunk_ukey in need_spilt_chunk_key_value_list.iter() {
    let modules = compilation
      .chunk_graph
      .get_chunk_modules(chunk_ukey, module_graph);
    let mut need_split_module_id_list = Vec::new();
    for module in modules {
      // 检查模块是否被其他 Chunk 引用
      let has_module_chunks = compilation
        .chunk_graph
        .get_module_chunks(module.identifier())
        .clone();

      if has_module_chunks.iter().len() > 1 {
        need_split_module_id_list.push(module.identifier());
      }
    }
    if need_split_module_id_list.iter().len() > 0 {
      need_split_chunk_module_list.push((need_split_module_id_list, chunk_ukey));
    }
  }

  let need_split_new_old_chunk_id_list = &mut vec![];

  if need_split_chunk_module_list.len() > 0 {
    for (need_split_module_id_list, need_split_chunk_ukey) in need_split_chunk_module_list.iter() {
      let need_split_chunk = &mut compilation
        .chunk_by_ukey
        .get(need_split_chunk_ukey)
        .unwrap();
      let new_chunk_name = format!(
        "{}_{}",
        need_split_chunk.name().unwrap().to_string(),
        "_share_module"
      );

      let (new_chunk_ukey, created) = {
        Compilation::add_named_chunk(
          new_chunk_name.to_string(),
          &mut compilation.chunk_by_ukey,
          &mut compilation.named_chunks,
        )
      };

      if !created {
        return Err(rspack_error::error!(format!(
          "新建 chunk 的时候存在重名的情况：{}",
          new_chunk_name
        )));
      }

      if let Some(mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkAdd {
          chunk: new_chunk_ukey,
        });
      }

      let new_chunk = compilation.chunk_by_ukey.expect_get_mut(&new_chunk_ukey);
      let new_chunk_reason = new_chunk.chunk_reason_mut();
      if let Some(new_chunk_reason) = new_chunk_reason {
        new_chunk_reason.push(',');
        new_chunk_reason.push_str("split by split_chunk_share_module_plugin");
      }

      compilation.chunk_graph.add_chunk(new_chunk_ukey);

      // 从原 Chunk 移除模块
      for need_split_module_id in need_split_module_id_list.iter() {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(need_split_chunk_ukey, *need_split_module_id);
        // 添加到新 Chunk
        compilation
          .chunk_graph
          .connect_chunk_and_module(new_chunk_ukey, *need_split_module_id);
      }
      need_split_new_old_chunk_id_list.push((new_chunk_ukey, need_split_chunk_ukey));
    }
  }

  for (new_chunk_ukey, need_split_chunk_ukey) in need_split_new_old_chunk_id_list {
    let [Some(new_chunk), Some(need_split_chunk)] = compilation
      .chunk_by_ukey
      .get_many_mut([&new_chunk_ukey, need_split_chunk_ukey])
    else {
      panic!("split_chunk_share_module_plugin split_from_original_chunks failed")
    };

    need_split_chunk.split(new_chunk, &mut compilation.chunk_group_by_ukey);

    if let Some(mutations) = compilation.incremental.mutations_write() {
      mutations.add(Mutation::ChunkSplit {
        from: ***need_split_chunk_ukey,
        to: *new_chunk_ukey,
      });
    }
  }
  Ok(())
}

impl Plugin for SplitChunkShareModule {
  fn name(&self) -> &'static str {
    "SplitChunkShareModule"
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
