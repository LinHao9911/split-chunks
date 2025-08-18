mod my_banner_plugin;
mod add_footer_comment_plugin;
mod split_chunk_share_module;

use napi::bindgen_prelude::*;
use rspack_binding_builder_macros::register_plugin;
use rspack_core::BoxPlugin;

#[macro_use]
extern crate napi_derive;
extern crate rspack_binding_builder;

// Export a plugin named `MyBannerPlugin`.
//
// The plugin needs to be wrapped with `require('@rspack/core').experiments.createNativePlugin`
// to be used in the host.
//
// Check out `lib/index.js` for more details.
//
// `register_plugin` is a macro that registers a plugin.
//
// The first argument to `register_plugin` is the name of the plugin.
// The second argument to `register_plugin` is a resolver function that is called with `napi::Env` and the options returned from the resolver function from JS side.
//
// The resolver function should return a `BoxPlugin` instance.
register_plugin!("MyBannerPlugin", |_env: Env, options: Unknown<'_>| {
  let banner = options
    .coerce_to_string()?
    .into_utf8()?
    .as_str()?
    .to_string();
  Ok(Box::new(my_banner_plugin::MyBannerPlugin::new(banner)) as BoxPlugin)
});

register_plugin!("AddFooterCommentPlugin", |_env: Env, options: Unknown<'_>| {
  let comment = options
    .coerce_to_string()?
    .into_utf8()?
    .as_str()?
    .to_string();
  Ok(Box::new(add_footer_comment_plugin::AddFooterCommentPlugin::new(comment)) as BoxPlugin)
});

register_plugin!("SplitChunkShareModule", |_env: Env, options: Unknown<'_>| {
  let options = options.coerce_to_object()?;
  let chunk_name_list = options.get::<Vec<String>>("chunk_name_list")?.unwrap();
  Ok(Box::new(split_chunk_share_module::SplitChunkShareModule::new(chunk_name_list)) as BoxPlugin)
});
